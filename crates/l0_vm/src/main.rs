use l0_core::*;
use std::env;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};
use serde_json::Value;

// ============================================================================
// L-0 VM v0.2
// ============================================================================

use serde::{Serialize, Deserialize};

// === Tool Registry ===

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PluginConfig {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    binary: Option<String>,
    method: Option<String>,
}

struct ToolRegistry {
    plugins: HashMap<u16, PluginConfig>,
    env_dir: String,
}

impl ToolRegistry {
    fn new() -> Self {
        ToolRegistry {
            plugins: HashMap::new(),
            env_dir: env::current_dir().unwrap().to_str().unwrap().to_string(),
        }
    }

    fn load(&mut self) {
        // Try local tools.json
        let paths = ["tools.json", "env/tools.json"];
        for p in &paths {
            if Path::new(p).exists() {
                 if let Ok(content) = fs::read_to_string(p) {
                     if let Ok(json) = serde_json::from_str::<Value>(&content) {
                         for section in ["plugins", "builtins"] {
                             if let Some(plugins) = json.get(section) {
                                 if let Some(obj) = plugins.as_object() {
                                     for (k, v) in obj {
                                         // Parse Hex Key "0x1234"
                                         let id = if k.starts_with("0x") {
                                             u16::from_str_radix(&k[2..], 16).unwrap_or(0)
                                         } else {
                                             k.parse().unwrap_or(0)
                                         };
                                         
                                         if let Ok(cfg) = serde_json::from_value::<PluginConfig>(v.clone()) {
                                             self.plugins.insert(id, cfg);
                                         } else {
                                             eprintln!("[Registry] Failed to parse config for {}", k);
                                         }
                                     }
                                 }
                             }
                         }
                         eprintln!("[Registry] Loaded {} tools from {}", self.plugins.len(), p);
                     } else {
                         eprintln!("[Registry] Failed to parse JSON from {}", p);
                     }
                 }
                 break; // Found one
            }
        }
    }

    fn get(&self, id: u16) -> Option<&PluginConfig> {
        self.plugins.get(&id)
    }
}

// === Linear Heap ===
// v2.0: Linear memory model with proper pointer arithmetic support

const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16MB default heap

#[derive(Clone)]
struct Allocation {
    offset: usize,  // Start offset in buffer
    size: usize,    // Allocated size
    used: bool,     // Is this allocation active?
}

struct LinearHeap {
    buffer: Vec<u8>,                      // Linear memory buffer
    allocations: HashMap<i64, Allocation>, // ptr_id -> allocation info
    next_ptr: i64,                         // Next pointer ID
    bump: usize,                           // Bump pointer for fast allocation
    free_list: Vec<(usize, usize)>,       // (offset, size) of freed regions
}

impl LinearHeap {
    fn new() -> Self {
        LinearHeap {
            buffer: vec![0u8; HEAP_SIZE],
            allocations: HashMap::new(),
            next_ptr: 1,  // Start from 1 (0 reserved for null)
            bump: 0,
            free_list: Vec::new(),
        }
    }

    /// Allocate memory and return pointer ID
    fn alloc(&mut self, data: Vec<u8>) -> i64 {
        let size = data.len();
        if size == 0 {
            return 0; // Null for empty allocation
        }

        // Try to reuse from free list (first-fit)
        let offset = if let Some(idx) = self.free_list.iter().position(|(_, s)| *s >= size) {
            let (off, free_size) = self.free_list.remove(idx);
            // If leftover space, put back in free list
            if free_size > size + 16 { // Min 16 bytes to avoid fragmentation
                self.free_list.push((off + size, free_size - size));
            }
            off
        } else {
            // Bump allocate
            if self.bump + size > self.buffer.len() {
                // Grow buffer
                let new_size = (self.bump + size).max(self.buffer.len() * 2);
                self.buffer.resize(new_size, 0);
            }
            let off = self.bump;
            self.bump += size;
            off
        };

        // Copy data to buffer
        self.buffer[offset..offset + size].copy_from_slice(&data);

        // Record allocation
        let ptr = self.next_ptr;
        self.next_ptr += 1;
        self.allocations.insert(ptr, Allocation { offset, size, used: true });

        ptr
    }

    /// Free an allocation
    fn free(&mut self, ptr: i64) {
        if let Some(alloc) = self.allocations.get_mut(&ptr) {
            if alloc.used {
                alloc.used = false;
                self.free_list.push((alloc.offset, alloc.size));
                // TODO: Coalesce adjacent free regions
            }
        }
    }

    /// Get immutable slice for pointer
    fn get(&self, ptr: i64) -> Option<&[u8]> {
        self.allocations.get(&ptr).and_then(|a| {
            if a.used {
                Some(&self.buffer[a.offset..a.offset + a.size])
            } else {
                None
            }
        })
    }

    /// Get mutable slice for pointer
    fn get_mut(&mut self, ptr: i64) -> Option<&mut [u8]> {
        if let Some(alloc) = self.allocations.get(&ptr) {
            if alloc.used {
                let offset = alloc.offset;
                let size = alloc.size;
                return Some(&mut self.buffer[offset..offset + size]);
            }
        }
        None
    }

    /// Get allocation length
    fn len(&self, ptr: i64) -> Option<usize> {
        self.allocations.get(&ptr).and_then(|a| {
            if a.used { Some(a.size) } else { None }
        })
    }

    /// Read byte at offset (supports pointer arithmetic)
    fn read_byte(&self, ptr: i64, offset: usize) -> Option<u8> {
        self.allocations.get(&ptr).and_then(|a| {
            if a.used && offset < a.size {
                Some(self.buffer[a.offset + offset])
            } else {
                None
            }
        })
    }

    /// Write byte at offset
    fn write_byte(&mut self, ptr: i64, offset: usize, val: u8) -> bool {
        if let Some(alloc) = self.allocations.get(&ptr) {
            if alloc.used && offset < alloc.size {
                self.buffer[alloc.offset + offset] = val;
                return true;
            }
        }
        false
    }

    /// Get raw offset for cross-allocation operations
    fn get_raw_offset(&self, ptr: i64) -> Option<usize> {
        self.allocations.get(&ptr).and_then(|a| {
            if a.used { Some(a.offset) } else { None }
        })
    }
}

// === VM ===

struct VM {
    // 256 Registers (i64 Unified)
    // Reserved: R240=target_vec, R241=state_vec, R242=threshold, R243=last_sim
    registers: [i64; 256],

    // Linear Heap Memory (v2.0)
    heap: LinearHeap,

    // Flags
    flag_eq: bool,
    flag_gt: bool,
    flag_lt: bool,

    // Instruction Pointer
    pc: usize,

    // State
    halted: bool,
    trapped: bool,      // Governance: VM is trapped, awaiting supervisor
    trap_code: u8,      // Governance: trap reason code

    // Tools
    registry: ToolRegistry,
}

impl VM {
    fn new() -> Self {
        let mut registry = ToolRegistry::new();
        registry.load();

        VM {
            registers: [0; 256],
            heap: LinearHeap::new(),
            flag_eq: false,
            flag_gt: false,
            flag_lt: false,
            pc: 0,
            halted: false,
            trapped: false,
            trap_code: 0,
            registry,
        }
    }

    // Helper: Allocate on linear heap
    fn heap_alloc(&mut self, data: Vec<u8>) -> i64 {
        self.heap.alloc(data)
    }
    
    // Execute External Plugin
    fn call_plugin(&self, plugin: &PluginConfig, arg: &str) -> String {
        if let (Some(binary), Some(method)) = (&plugin.binary, &plugin.method) {
             let bin_path = if Path::new(binary).exists() {
                 binary.to_string()
             } else {
                 format!("{}/{}", self.registry.env_dir, binary)
             };

             // Escape special characters in argument for JSON
             let escaped_arg = arg
                 .replace('\\', "\\\\")
                 .replace('"', "\\\"")
                 .replace('\n', "\\n")
                 .replace('\r', "\\r")
                 .replace('\t', "\\t");

             // Construct JSON Request (with newline for line-based plugins)
             let req = format!("{{ \"method\": \"{}\", \"args\": [\"{}\"] }}\n", method, escaped_arg);
             
             // Spawn Process
             let mut child = Command::new(&bin_path)
                 .stdin(Stdio::piped())
                 .stdout(Stdio::piped())
                 .spawn();
                 
             if let Ok(mut child) = child {
                 if let Some(mut stdin) = child.stdin.take() {
                     let _ = stdin.write_all(req.as_bytes());
                 }
                 
                 if let Some(stdout) = child.stdout.take() {
                     let mut reader = BufReader::new(stdout);
                     let mut line = String::new();
                     if reader.read_line(&mut line).is_ok() {
                         // Parse Response: { "ok": true, "value": ... }
                         if let Ok(resp) = serde_json::from_str::<Value>(&line) {
                             if resp["ok"].as_bool() == Some(true) {
                                 // Handle different value types
                                 let value = &resp["value"];
                                 if let Some(s) = value.as_str() {
                                     return s.to_string();
                                 } else {
                                     // Return JSON representation for non-string values
                                     return value.to_string();
                                 }
                             } else {
                                 return format!("Error: {}", resp["error"]);
                             }
                         }
                     }
                 }
             }
             return "[Plugin Exec Failed]".to_string();
        }
        "[Invalid Plugin Config]".to_string()
    }

    fn execute(&mut self, program: &[Instruction], debug: bool) {
        self.pc = 0;
        self.halted = false;

        while self.pc < program.len() && !self.halted {
            let instr = &program[self.pc];
            if debug {
                eprintln!("[PC:{:03}] {:?}", self.pc, instr);
            }
            
            self.pc += 1;

            match instr {
                // ... (Previous Instructions match logic is same) ...
                
                Instruction::TEXEC { tool, arg, dest } => {
                    let arg_ptr = self.registers[*arg as usize];
                    let arg_str = self.heap.get(arg_ptr).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    
                    let tool_name = if let Some(plugin) = self.registry.get(*tool) {
                        plugin.name.as_str()
                    } else {
                        "UNKNOWN"
                    };

                    let result_str = if let Some(plugin) = self.registry.get(*tool) {
                        if plugin.kind == "plugin" {
                             self.call_plugin(plugin, &arg_str)
                        } else {
                             // It's a builtin check by name below
                             match tool_name {
                                "PRINT" => {
                                    println!("{}", arg_str);
                                    "".to_string()
                                },
                                "PRINTN" => {
                                    use std::io::Write;
                                    print!("{}", arg_str);
                                    std::io::stdout().flush().unwrap_or(());
                                    "".to_string()
                                },
                                "INPUT" => {
                                    let mut input = String::new();
                                    if std::io::stdin().read_line(&mut input).is_ok() {
                                        input.trim().to_string()
                                    } else {
                                        "".to_string()
                                    }
                                },
                                "POW" => {
                                    let parts: Vec<&str> = arg_str.split(',').map(|s| s.trim()).collect();
                                    if parts.len() == 2 {
                                        if let (Ok(a), Ok(b)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                                            a.pow(b as u32).to_string()
                                        } else { "Error: Invalid Args".to_string() }
                                    } else {
                                        "Error: Need 2 Args".to_string()
                                    }
                                },
                                "RAND" => {
                                    // RAND: arg = "max" -> returns random 0..max
                                    use std::time::{SystemTime, UNIX_EPOCH};
                                    let max: i64 = arg_str.trim().parse().unwrap_or(100);
                                    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
                                    let rand_val = (seed % (max as u64)) as i64;
                                    rand_val.to_string()
                                },
                                "STRLEN" => {
                                    // STRLEN: returns length of string
                                    arg_str.len().to_string()
                                },
                                "ABS" => {
                                    // ABS: absolute value
                                    let val: i64 = arg_str.trim().parse().unwrap_or(0);
                                    val.abs().to_string()
                                },
                                "MIN" => {
                                    // MIN: "a,b" -> min(a,b)
                                    let parts: Vec<&str> = arg_str.split(',').map(|s| s.trim()).collect();
                                    if parts.len() == 2 {
                                        if let (Ok(a), Ok(b)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                                            std::cmp::min(a, b).to_string()
                                        } else { "Error: Invalid Args".to_string() }
                                    } else { "Error: Need 2 Args".to_string() }
                                },
                                "MAX" => {
                                    // MAX: "a,b" -> max(a,b)
                                    let parts: Vec<&str> = arg_str.split(',').map(|s| s.trim()).collect();
                                    if parts.len() == 2 {
                                        if let (Ok(a), Ok(b)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                                            std::cmp::max(a, b).to_string()
                                        } else { "Error: Invalid Args".to_string() }
                                    } else { "Error: Need 2 Args".to_string() }
                                },
                                "TIME" => {
                                    // TIME: returns current unix timestamp in seconds
                                    use std::time::{SystemTime, UNIX_EPOCH};
                                    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                                    ts.to_string()
                                },
                                "CONCAT" => {
                                    // CONCAT: "str1,str2" -> "str1str2"
                                    let parts: Vec<&str> = arg_str.splitn(2, ',').collect();
                                    if parts.len() == 2 {
                                        format!("{}{}", parts[0], parts[1])
                                    } else { arg_str.to_string() }
                                },
                                "SUBSTR" => {
                                    // SUBSTR: "str,start,len" -> substring
                                    let parts: Vec<&str> = arg_str.splitn(3, ',').collect();
                                    if parts.len() == 3 {
                                        let s = parts[0];
                                        let start: usize = parts[1].trim().parse().unwrap_or(0);
                                        let len: usize = parts[2].trim().parse().unwrap_or(s.len());
                                        if start < s.len() {
                                            let end = std::cmp::min(start + len, s.len());
                                            s[start..end].to_string()
                                        } else { "".to_string() }
                                    } else { "Error: Need str,start,len".to_string() }
                                },
                                "SPLIT" => {
                                    // SPLIT: "str,delim,index" -> returns element at index after split
                                    let parts: Vec<&str> = arg_str.splitn(3, ',').collect();
                                    if parts.len() == 3 {
                                        let s = parts[0];
                                        let delim = parts[1];
                                        let idx: usize = parts[2].trim().parse().unwrap_or(0);
                                        let items: Vec<&str> = s.split(delim).collect();
                                        if idx < items.len() {
                                            items[idx].to_string()
                                        } else { "".to_string() }
                                    } else { "Error: Need str,delim,index".to_string() }
                                },
                                "UPPER" => {
                                    // UPPER: convert to uppercase
                                    arg_str.to_uppercase()
                                },
                                "LOWER" => {
                                    // LOWER: convert to lowercase
                                    arg_str.to_lowercase()
                                },
                                "TRIM" => {
                                    // TRIM: remove whitespace
                                    arg_str.trim().to_string()
                                },
                                _ => { format!("[Tool {} called with {}]", tool_name, arg_str) }
                             }
                        }
                    } else {
                        format!("[Tool ID {} Not Found]", tool)
                    };
                    
                    let res_ptr = self.heap_alloc(result_str.as_bytes().to_vec());
                    self.registers[*dest as usize] = res_ptr;
                },
                
                // Pass through others to avoid compile error (I will implement full match below)
                _ => self.execute_rest(instr), 
            }
        }
    }
    
    // Helper to reduce match size in replace_file_content (hacky but needed for partial replacement)
    fn execute_rest(&mut self, instr: &Instruction) {
         match instr {
                Instruction::NOP => {},
                Instruction::HALT => self.halted = true,
                Instruction::PANIC(msg) => { eprintln!("PANIC: {}", msg); std::process::exit(1); },
                Instruction::DUMP => {
                    println!("--- VM DUMP ---");
                    println!("Regs[0-15]: {:?}", &self.registers[0..16]);
                    println!("Regs[10-25]: {:?}", &self.registers[10..26]);
                    println!("Linear Heap: {} allocations, {} bytes used",
                             self.heap.allocations.len(), self.heap.bump);
                    for (ptr, alloc) in &self.heap.allocations {
                        if alloc.used {
                            let data = &self.heap.buffer[alloc.offset..alloc.offset + alloc.size];
                            let preview = String::from_utf8_lossy(data);
                            let truncated = if preview.len() > 40 {
                                format!("{}...", &preview[..40])
                            } else {
                                preview.to_string()
                            };
                            println!("  [ptr={}]: \"{}\" ({} bytes @ offset {})",
                                     ptr, truncated, alloc.size, alloc.offset);
                        }
                    }
                },
                Instruction::GAS(reg) => { self.registers[*reg as usize] = 1000; },
                Instruction::ASSERT(reg) => {
                    if self.registers[*reg as usize] == 0 {
                         eprintln!("ASSERTION FAILED"); std::process::exit(1);
                    }
                },
                Instruction::SET { reg, val } => { self.registers[*reg as usize] = *val; },
                Instruction::SETS { reg, val } => {
                     let ptr = self.heap_alloc(val.as_bytes().to_vec());
                     self.registers[*reg as usize] = ptr;
                },
                Instruction::MOV { dest, src } => { self.registers[*dest as usize] = self.registers[*src as usize]; },
                Instruction::SWAP { r1, r2 } => {
                    let tmp = self.registers[*r1 as usize];
                    self.registers[*r1 as usize] = self.registers[*r2 as usize];
                    self.registers[*r2 as usize] = tmp;
                },
                Instruction::ADD { dest, s1, s2 } => { self.registers[*dest as usize] = self.registers[*s1 as usize] + self.registers[*s2 as usize]; },
                Instruction::SUB { dest, s1, s2 } => { self.registers[*dest as usize] = self.registers[*s1 as usize] - self.registers[*s2 as usize]; },
                Instruction::MUL { dest, s1, s2 } => { self.registers[*dest as usize] = self.registers[*s1 as usize] * self.registers[*s2 as usize]; },
                Instruction::DIV { dest, s1, s2 } => { 
                    let v2 = self.registers[*s2 as usize];
                    if v2 != 0 { self.registers[*dest as usize] = self.registers[*s1 as usize] / v2; }
                },
                Instruction::MOD { dest, s1, s2 } => { 
                    let v2 = self.registers[*s2 as usize];
                    if v2 != 0 { self.registers[*dest as usize] = self.registers[*s1 as usize] % v2; }
                },
                Instruction::AND { dest, s1, s2 } => { self.registers[*dest as usize] = self.registers[*s1 as usize] & self.registers[*s2 as usize]; },
                Instruction::OR  { dest, s1, s2 } => { self.registers[*dest as usize] = self.registers[*s1 as usize] | self.registers[*s2 as usize]; },
                Instruction::XOR { dest, s1, s2 } => { self.registers[*dest as usize] = self.registers[*s1 as usize] ^ self.registers[*s2 as usize]; },
                Instruction::NOT { dest, src }    => { self.registers[*dest as usize] = !self.registers[*src as usize]; },
                Instruction::CMP { r1, r2 } => {
                    let v1 = self.registers[*r1 as usize];
                    let v2 = self.registers[*r2 as usize];
                    self.flag_eq = v1 == v2;
                    self.flag_gt = v1 > v2;
                    self.flag_lt = v1 < v2;
                },
                Instruction::JMP { target } => { self.pc = *target; },
                Instruction::BEQ { target } => { if self.flag_eq { self.pc = *target; } },
                Instruction::BGT { target } => { if self.flag_gt { self.pc = *target; } },
                Instruction::BLT { target } => { if self.flag_lt { self.pc = *target; } },
                Instruction::NEW { dest, size } => {
                    let ptr = self.heap_alloc(vec![0u8; *size]);
                     self.registers[*dest as usize] = ptr;
                },
                Instruction::FREE { ptr } => {
                    let p = self.registers[*ptr as usize];
                    self.heap.free(p);
                },
                Instruction::READ { dest, ptr, offset } => {
                    let p = self.registers[*ptr as usize];
                    if let Some(v) = self.heap.read_byte(p, *offset) {
                        self.registers[*dest as usize] = v as i64;
                    }
                },
                Instruction::WRITE { ptr, offset, val } => {
                    let p = self.registers[*ptr as usize];
                    let v = self.registers[*val as usize] as u8;
                    self.heap.write_byte(p, *offset, v);
                },
                Instruction::REGEX { dest, pat, text } => {
                    let pat_ptr = self.registers[*pat as usize];
                    let txt_ptr = self.registers[*text as usize];
                    let pat_str = self.heap.get(pat_ptr).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    let txt_str = self.heap.get(txt_ptr).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    match regex::Regex::new(&pat_str) {
                         Ok(re) => { self.registers[*dest as usize] = if re.is_match(&txt_str) { 1 } else { 0 }; },
                         Err(_) => { self.registers[*dest as usize] = 0; }
                    }
                },
                Instruction::ITOA { dest, src } => {
                    let val = self.registers[*src as usize];
                    let s = val.to_string();
                    let ptr = self.heap_alloc(s.as_bytes().to_vec());
                    self.registers[*dest as usize] = ptr;
                },
                Instruction::ATOI { dest, src } => {
                    let str_ptr = self.registers[*src as usize];
                    let s = self.heap.get(str_ptr).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_else(|| "0".to_string());
                    self.registers[*dest as usize] = s.trim().parse::<i64>().unwrap_or(0);
                },
                Instruction::READR { dest, ptr, off } => {
                    let p = self.registers[*ptr as usize];
                    let offset = self.registers[*off as usize] as usize;
                    if let Some(v) = self.heap.read_byte(p, offset) {
                        self.registers[*dest as usize] = v as i64;
                    }
                },
                Instruction::WRITER { ptr, off, val } => {
                    let p = self.registers[*ptr as usize];
                    let offset = self.registers[*off as usize] as usize;
                    let v = self.registers[*val as usize] as u8;
                    self.heap.write_byte(p, offset, v);
                },
                Instruction::SCAT { dest, s1, s2 } => {
                    let ptr1 = self.registers[*s1 as usize];
                    let ptr2 = self.registers[*s2 as usize];
                    let str1 = self.heap.get(ptr1).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    let str2 = self.heap.get(ptr2).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();
                    let result = format!("{}{}", str1, str2);
                    let ptr = self.heap_alloc(result.as_bytes().to_vec());
                    self.registers[*dest as usize] = ptr;
                },
                Instruction::STORE64 { ptr, off, val } => {
                    let p = self.registers[*ptr as usize];
                    let offset = (self.registers[*off as usize] as usize) * 8;
                    let v = self.registers[*val as usize];
                    let bytes = v.to_le_bytes();
                    if let Some(data) = self.heap.get_mut(p) {
                        if offset + 8 <= data.len() {
                            data[offset..offset+8].copy_from_slice(&bytes);
                        }
                    }
                },
                Instruction::LOAD64 { dest, ptr, off } => {
                    let p = self.registers[*ptr as usize];
                    let offset = (self.registers[*off as usize] as usize) * 8;
                    if let Some(data) = self.heap.get(p) {
                        if offset + 8 <= data.len() {
                            let bytes: [u8; 8] = data[offset..offset+8].try_into().unwrap_or([0u8; 8]);
                            self.registers[*dest as usize] = i64::from_le_bytes(bytes);
                        }
                    }
                },
                // === Batch Memory Operations ===
                Instruction::MEMCPY { dst, doff, src, soff, len } => {
                    let src_ptr = self.registers[*src as usize];
                    let src_off = self.registers[*soff as usize] as usize;
                    let dst_ptr = self.registers[*dst as usize];
                    let dst_off = self.registers[*doff as usize] as usize;
                    let length = self.registers[*len as usize] as usize;

                    // Read source bytes
                    let bytes: Vec<u8> = if let Some(data) = self.heap.get(src_ptr) {
                        if src_off + length <= data.len() {
                            data[src_off..src_off + length].to_vec()
                        } else { vec![] }
                    } else { vec![] };

                    // Write to destination
                    if !bytes.is_empty() {
                        if let Some(data) = self.heap.get_mut(dst_ptr) {
                            if dst_off + length <= data.len() {
                                data[dst_off..dst_off + length].copy_from_slice(&bytes);
                            }
                        }
                    }
                },
                Instruction::HLEN { dest, ptr } => {
                    let p = self.registers[*ptr as usize];
                    let len = self.heap.len(p).unwrap_or(0) as i64;
                    self.registers[*dest as usize] = len;
                },
                Instruction::SLICE { dest, ptr, off, len } => {
                    let p = self.registers[*ptr as usize];
                    let offset = self.registers[*off as usize] as usize;
                    let length = self.registers[*len as usize] as usize;

                    let slice: Vec<u8> = if let Some(data) = self.heap.get(p) {
                        if offset + length <= data.len() {
                            data[offset..offset + length].to_vec()
                        } else if offset < data.len() {
                            data[offset..].to_vec()
                        } else { vec![] }
                    } else { vec![] };

                    let new_ptr = self.heap_alloc(slice);
                    self.registers[*dest as usize] = new_ptr;
                },
                Instruction::MEMSET { ptr, off, len, val } => {
                    let p = self.registers[*ptr as usize];
                    let offset = self.registers[*off as usize] as usize;
                    let length = self.registers[*len as usize] as usize;
                    let value = self.registers[*val as usize] as u8;

                    if let Some(data) = self.heap.get_mut(p) {
                        let end = std::cmp::min(offset + length, data.len());
                        for i in offset..end {
                            data[i] = value;
                        }
                    }
                },

                // === Vector Operations (Semantic Computing) ===
                // Vector format: [dims:i64 (8 bytes)][f64][f64]...
                // Results scaled by 1_000_000 for integer precision

                Instruction::VNEW { dest, dims } => {
                    let num_dims = self.registers[*dims as usize] as usize;
                    // 8 bytes for dimension count + 8 bytes per dimension
                    let size = 8 + num_dims * 8;
                    let mut data = vec![0u8; size];
                    // Store dimension count at offset 0
                    data[0..8].copy_from_slice(&(num_dims as i64).to_le_bytes());
                    let ptr = self.heap_alloc(data);
                    self.registers[*dest as usize] = ptr;
                },

                Instruction::VSET { vec, idx, val } => {
                    let p = self.registers[*vec as usize];
                    let index = self.registers[*idx as usize] as usize;
                    let value_bits = self.registers[*val as usize] as u64;

                    if let Some(data) = self.heap.get_mut(p) {
                        let offset = 8 + index * 8; // Skip dimension header
                        if offset + 8 <= data.len() {
                            data[offset..offset+8].copy_from_slice(&value_bits.to_le_bytes());
                        }
                    }
                },

                Instruction::VGET { dest, vec, idx } => {
                    let p = self.registers[*vec as usize];
                    let index = self.registers[*idx as usize] as usize;

                    if let Some(data) = self.heap.get(p) {
                        let offset = 8 + index * 8;
                        if offset + 8 <= data.len() {
                            let bytes: [u8; 8] = data[offset..offset+8].try_into().unwrap_or([0u8; 8]);
                            self.registers[*dest as usize] = i64::from_le_bytes(bytes);
                        }
                    }
                },

                Instruction::VDOT { dest, v1, v2 } => {
                    let p1 = self.registers[*v1 as usize];
                    let p2 = self.registers[*v2 as usize];

                    let mut dot = 0.0f64;

                    if let (Some(d1), Some(d2)) = (self.heap.get(p1), self.heap.get(p2)) {
                        let dims1 = i64::from_le_bytes(d1[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        let dims2 = i64::from_le_bytes(d2[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        let dims = std::cmp::min(dims1, dims2);

                        for i in 0..dims {
                            let off = 8 + i * 8;
                            let a = f64::from_le_bytes(d1[off..off+8].try_into().unwrap_or([0u8; 8]));
                            let b = f64::from_le_bytes(d2[off..off+8].try_into().unwrap_or([0u8; 8]));
                            dot += a * b;
                        }
                    }

                    // Scale by 1_000_000 for integer precision
                    self.registers[*dest as usize] = (dot * 1_000_000.0) as i64;
                },

                Instruction::VSIM { dest, v1, v2 } => {
                    // Cosine similarity = dot(v1,v2) / (|v1| * |v2|)
                    let p1 = self.registers[*v1 as usize];
                    let p2 = self.registers[*v2 as usize];

                    let mut dot = 0.0f64;
                    let mut mag1 = 0.0f64;
                    let mut mag2 = 0.0f64;

                    if let (Some(d1), Some(d2)) = (self.heap.get(p1), self.heap.get(p2)) {
                        let dims1 = i64::from_le_bytes(d1[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        let dims2 = i64::from_le_bytes(d2[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        let dims = std::cmp::min(dims1, dims2);

                        for i in 0..dims {
                            let off = 8 + i * 8;
                            let a = f64::from_le_bytes(d1[off..off+8].try_into().unwrap_or([0u8; 8]));
                            let b = f64::from_le_bytes(d2[off..off+8].try_into().unwrap_or([0u8; 8]));
                            dot += a * b;
                            mag1 += a * a;
                            mag2 += b * b;
                        }
                    }

                    let similarity = if mag1 > 0.0 && mag2 > 0.0 {
                        dot / (mag1.sqrt() * mag2.sqrt())
                    } else { 0.0 };

                    // Scale by 1_000_000 (range: -1M to +1M)
                    self.registers[*dest as usize] = (similarity * 1_000_000.0) as i64;
                },

                Instruction::VMAG { dest, vec } => {
                    let p = self.registers[*vec as usize];
                    let mut mag = 0.0f64;

                    if let Some(data) = self.heap.get(p) {
                        let dims = i64::from_le_bytes(data[0..8].try_into().unwrap_or([0u8; 8])) as usize;

                        for i in 0..dims {
                            let off = 8 + i * 8;
                            if off + 8 <= data.len() {
                                let v = f64::from_le_bytes(data[off..off+8].try_into().unwrap_or([0u8; 8]));
                                mag += v * v;
                            }
                        }
                    }

                    self.registers[*dest as usize] = (mag.sqrt() * 1_000_000.0) as i64;
                },

                Instruction::VNORM { vec } => {
                    let p = self.registers[*vec as usize];

                    // First calculate magnitude
                    let mut mag = 0.0f64;
                    let dims = if let Some(data) = self.heap.get(p) {
                        let d = i64::from_le_bytes(data[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        for i in 0..d {
                            let off = 8 + i * 8;
                            if off + 8 <= data.len() {
                                let v = f64::from_le_bytes(data[off..off+8].try_into().unwrap_or([0u8; 8]));
                                mag += v * v;
                            }
                        }
                        d
                    } else { 0 };

                    mag = mag.sqrt();

                    // Normalize in-place
                    if mag > 0.0 {
                        if let Some(data) = self.heap.get_mut(p) {
                            for i in 0..dims {
                                let off = 8 + i * 8;
                                if off + 8 <= data.len() {
                                    let v = f64::from_le_bytes(data[off..off+8].try_into().unwrap_or([0u8; 8]));
                                    let normalized = v / mag;
                                    data[off..off+8].copy_from_slice(&normalized.to_le_bytes());
                                }
                            }
                        }
                    }
                },

                // === Governance (Semantic Drift Control) ===
                // Reserved registers: R240=target_vec, R241=state_vec, R242=threshold, R243=last_sim

                Instruction::LATCH { target, threshold } => {
                    // Latch target vector pointer to R240, threshold to R242
                    self.registers[240] = self.registers[*target as usize];
                    self.registers[242] = self.registers[*threshold as usize];
                    eprintln!("[GOVERNANCE] Latched target_vec=R240, threshold={}", self.registers[242]);
                },

                Instruction::GUARD { state } => {
                    // Compute VSIM between R240 (target) and R[state]
                    // If similarity < R242 (threshold), trigger TRAP
                    let p1 = self.registers[240];  // target vector
                    let p2 = self.registers[*state as usize];  // current state vector

                    let mut dot = 0.0f64;
                    let mut mag1 = 0.0f64;
                    let mut mag2 = 0.0f64;

                    if let (Some(d1), Some(d2)) = (self.heap.get(p1), self.heap.get(p2)) {
                        let dims1 = i64::from_le_bytes(d1[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        let dims2 = i64::from_le_bytes(d2[0..8].try_into().unwrap_or([0u8; 8])) as usize;
                        let dims = std::cmp::min(dims1, dims2);

                        for i in 0..dims {
                            let off = 8 + i * 8;
                            let a = f64::from_le_bytes(d1[off..off+8].try_into().unwrap_or([0u8; 8]));
                            let b = f64::from_le_bytes(d2[off..off+8].try_into().unwrap_or([0u8; 8]));
                            dot += a * b;
                            mag1 += a * a;
                            mag2 += b * b;
                        }
                    }

                    let similarity = if mag1 > 0.0 && mag2 > 0.0 {
                        (dot / (mag1.sqrt() * mag2.sqrt()) * 1_000_000.0) as i64
                    } else { 0 };

                    self.registers[243] = similarity;  // Store last similarity in R243
                    let threshold = self.registers[242];

                    eprintln!("[GOVERNANCE] GUARD: similarity={}, threshold={}", similarity, threshold);

                    if similarity < threshold {
                        eprintln!("[GOVERNANCE] DRIFT DETECTED! Triggering TRAP...");
                        self.trapped = true;
                        self.trap_code = 1;  // Drift trap

                        // Emit trap context to stdout for supervisor
                        let context = serde_json::json!({
                            "trap": "DRIFT",
                            "code": 1,
                            "pc": self.pc,
                            "similarity": similarity,
                            "threshold": threshold,
                            "registers": {
                                "R240_target": self.registers[240],
                                "R241_state": self.registers[*state as usize],
                                "R242_threshold": self.registers[242],
                                "R243_last_sim": self.registers[243]
                            }
                        });
                        println!("{}", serde_json::to_string(&context).unwrap());

                        // Wait for supervisor input
                        let mut input = String::new();
                        eprintln!("[GOVERNANCE] Awaiting supervisor response (JSON)...");
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            if let Ok(resp) = serde_json::from_str::<Value>(&input) {
                                if resp["action"].as_str() == Some("continue") {
                                    eprintln!("[GOVERNANCE] Supervisor: continue");
                                    self.trapped = false;
                                } else if resp["action"].as_str() == Some("abort") {
                                    eprintln!("[GOVERNANCE] Supervisor: abort");
                                    self.halted = true;
                                } else if let Some(new_pc) = resp["jump"].as_u64() {
                                    eprintln!("[GOVERNANCE] Supervisor: jump to {}", new_pc);
                                    self.pc = new_pc as usize;
                                    self.trapped = false;
                                }
                            }
                        }
                    }
                },

                Instruction::TRAP { code } => {
                    // Manual trap - emit context and suspend
                    self.trapped = true;
                    self.trap_code = *code;

                    let context = serde_json::json!({
                        "trap": "MANUAL",
                        "code": *code,
                        "pc": self.pc,
                        "registers_0_15": &self.registers[0..16],
                        "last_similarity": self.registers[243]
                    });
                    println!("{}", serde_json::to_string(&context).unwrap());

                    // Wait for supervisor
                    let mut input = String::new();
                    eprintln!("[GOVERNANCE] TRAP {}: Awaiting supervisor...", code);
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        if let Ok(resp) = serde_json::from_str::<Value>(&input) {
                            if resp["action"].as_str() == Some("continue") {
                                self.trapped = false;
                            } else if resp["action"].as_str() == Some("abort") {
                                self.halted = true;
                            } else if let Some(new_pc) = resp["jump"].as_u64() {
                                self.pc = new_pc as usize;
                                self.trapped = false;
                            }
                            // Allow setting registers from supervisor
                            if let Some(regs) = resp["set_registers"].as_object() {
                                for (k, v) in regs {
                                    if let (Ok(idx), Some(val)) = (k.parse::<usize>(), v.as_i64()) {
                                        if idx < 256 {
                                            self.registers[idx] = val;
                                        }
                                    }
                                }
                            }
                        }
                    }
                },

                Instruction::YIELD { query, dest } => {
                    // Yield to supervisor with a query, receive response
                    let query_ptr = self.registers[*query as usize];
                    let query_str = self.heap.get(query_ptr).map(|d| String::from_utf8_lossy(d).to_string()).unwrap_or_default();

                    let request = serde_json::json!({
                        "yield": true,
                        "query": query_str,
                        "pc": self.pc,
                        "last_similarity": self.registers[243]
                    });
                    println!("{}", serde_json::to_string(&request).unwrap());

                    // Wait for response
                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        let response = input.trim().to_string();
                        let ptr = self.heap_alloc(response.as_bytes().to_vec());
                        self.registers[*dest as usize] = ptr;
                    }
                },

                _ => {}
         }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("L-0 Virtual Machine v1.0");
        eprintln!("Usage: l0vm <program.l0> [--debug] [--dev]");
        eprintln!("");
        eprintln!("Standard workflow:");
        eprintln!("  l0asm source.asm > program.l0   # Compile ASM to bytecode");
        eprintln!("  l0vm program.l0                 # Execute bytecode");
        eprintln!("");
        eprintln!("Flags:");
        eprintln!("  --debug   Trace execution (print each instruction)");
        eprintln!("  --dev     Allow .json source files (for VM development only)");
        eprintln!("  --info    Print ISA and tool registry");
        std::process::exit(1);
    }

    // Check for --info flag first
    if args.contains(&"--info".to_string()) {
        let mut registry = ToolRegistry::new();
        registry.load();

        let mut tools_map = serde_json::Map::new();
        for (id, cfg) in &registry.plugins {
            tools_map.insert(format!("0x{:04X}", id), serde_json::to_value(cfg).unwrap());
        }

        let info = serde_json::json!({
            "version": "1.0.0",
            "usage": "l0vm <program.l0> [--debug]",
            "description": "L-Zero Virtual Machine",
            "source_format": "ASM (.asm) - compile with l0asm",
            "binary_format": "Bincode (.l0)",
            "architecture": {
                "registers": 256,
                "heap": "Linear memory model",
                "endianness": "Little Endian"
            },
            "isa": {
                "primitives": Instruction::introspection()
            },
            "tools": tools_map
        });

        println!("{}", serde_json::to_string_pretty(&info).unwrap());
        std::process::exit(0);
    }

    let debug = args.iter().any(|a| a == "--debug");
    let dev_mode = args.iter().any(|a| a == "--dev");

    // Find the first argument that is not a flag
    let path = args.iter().skip(1).find(|a| !a.starts_with("--")).expect("No program file specified");

    // Load program - prefer binary, JSON only in dev mode
    let program: Vec<Instruction> = if path.ends_with(".json") {
        if !dev_mode {
            eprintln!("Error: JSON source files are deprecated for production.");
            eprintln!("Use the assembler workflow:");
            eprintln!("  1. Write source.asm (with labels)");
            eprintln!("  2. l0asm source.asm > program.l0");
            eprintln!("  3. l0vm program.l0");
            eprintln!("");
            eprintln!("To load JSON anyway (for VM development), use --dev flag:");
            eprintln!("  l0vm program.json --dev");
            std::process::exit(1);
        }
        eprintln!("[DEV] Loading JSON source (deprecated format)");
        let content = fs::read_to_string(path).expect("Failed to read JSON file");
        serde_json::from_str(&content).expect("Failed to parse L-Zero JSON program")
    } else {
        // Binary format (.l0) - preferred
        let content = fs::read(path).expect("Failed to read binary file");
        bincode::deserialize(&content).expect("Failed to parse L-Zero Binary Bytecode")
    };

    let mut vm = VM::new();
    vm.execute(&program, debug);
}
