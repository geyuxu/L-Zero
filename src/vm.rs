mod lib;
use lib::*;
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

// === VM ===

struct VM {
    // 256 Registers (i64 Unified)
    // Reserved: R240=target_vec, R241=state_vec, R242=threshold, R243=last_sim
    registers: [i64; 256],

    // Heap Memory: Index -> Byte Vector
    heap: Vec<Option<Vec<u8>>>,

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
            heap: Vec::new(),
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

    // Helper: Allocate on heap
    fn heap_alloc(&mut self, data: Vec<u8>) -> i64 {
        if let Some(pos) = self.heap.iter().position(|x| x.is_none()) {
            self.heap[pos] = Some(data);
            return pos as i64;
        }
        let idx = self.heap.len();
        self.heap.push(Some(data));
        idx as i64
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
                    let arg_ptr = self.registers[*arg as usize] as usize;
                    let arg_str = if let Some(Some(d)) = self.heap.get(arg_ptr) {
                         String::from_utf8_lossy(d).to_string()
                    } else { "".to_string() };
                    
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
                    println!("Heap ({} slots):", self.heap.len());
                    for (i, slot) in self.heap.iter().enumerate() {
                        if let Some(data) = slot {
                            let preview = String::from_utf8_lossy(data);
                            let truncated = if preview.len() > 40 {
                                format!("{}...", &preview[..40])
                            } else {
                                preview.to_string()
                            };
                            println!("  [{}]: \"{}\" ({} bytes)", i, truncated, data.len());
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
                    let idx = self.registers[*ptr as usize] as usize;
                    if idx < self.heap.len() { self.heap[idx] = None; }
                },
                Instruction::READ { dest, ptr, offset } => {
                    let hp = self.registers[*ptr as usize] as usize;
                     if let Some(Some(data)) = self.heap.get(hp) {
                        if *offset < data.len() { self.registers[*dest as usize] = data[*offset] as i64; }
                     }
                },
                Instruction::WRITE { ptr, offset, val } => {
                     let hp = self.registers[*ptr as usize] as usize;
                     let v = self.registers[*val as usize] as u8;
                     if hp < self.heap.len() {
                         if let Some(data) = &mut self.heap[hp] {
                             if *offset < data.len() { data[*offset] = v; }
                         }
                     }
                },
                Instruction::REGEX { dest, pat, text } => {
                    let pat_ptr = self.registers[*pat as usize] as usize;
                    let txt_ptr = self.registers[*text as usize] as usize;
                    let pat_str = if let Some(Some(d)) = self.heap.get(pat_ptr) { String::from_utf8_lossy(d).to_string() } else { "".to_string() };
                    let txt_str = if let Some(Some(d)) = self.heap.get(txt_ptr) { String::from_utf8_lossy(d).to_string() } else { "".to_string() };
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
                    let str_ptr = self.registers[*src as usize] as usize;
                    let s = if let Some(Some(d)) = self.heap.get(str_ptr) {
                        String::from_utf8_lossy(d).to_string()
                    } else { "0".to_string() };
                    self.registers[*dest as usize] = s.trim().parse::<i64>().unwrap_or(0);
                },
                Instruction::READR { dest, ptr, off } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let offset = self.registers[*off as usize] as usize;
                    if let Some(Some(data)) = self.heap.get(hp) {
                        if offset < data.len() {
                            self.registers[*dest as usize] = data[offset] as i64;
                        }
                    }
                },
                Instruction::WRITER { ptr, off, val } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let offset = self.registers[*off as usize] as usize;
                    let v = self.registers[*val as usize] as u8;
                    if hp < self.heap.len() {
                        if let Some(data) = &mut self.heap[hp] {
                            if offset < data.len() { data[offset] = v; }
                        }
                    }
                },
                Instruction::SCAT { dest, s1, s2 } => {
                    let ptr1 = self.registers[*s1 as usize] as usize;
                    let ptr2 = self.registers[*s2 as usize] as usize;
                    let str1 = if let Some(Some(d)) = self.heap.get(ptr1) {
                        String::from_utf8_lossy(d).to_string()
                    } else { "".to_string() };
                    let str2 = if let Some(Some(d)) = self.heap.get(ptr2) {
                        String::from_utf8_lossy(d).to_string()
                    } else { "".to_string() };
                    let result = format!("{}{}", str1, str2);
                    let ptr = self.heap_alloc(result.as_bytes().to_vec());
                    self.registers[*dest as usize] = ptr;
                },
                Instruction::STORE64 { ptr, off, val } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let offset = (self.registers[*off as usize] as usize) * 8;
                    let v = self.registers[*val as usize];
                    let bytes = v.to_le_bytes();
                    if hp < self.heap.len() {
                        if let Some(data) = &mut self.heap[hp] {
                            if offset + 8 <= data.len() {
                                data[offset..offset+8].copy_from_slice(&bytes);
                            }
                        }
                    }
                },
                Instruction::LOAD64 { dest, ptr, off } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let offset = (self.registers[*off as usize] as usize) * 8;
                    if let Some(Some(data)) = self.heap.get(hp) {
                        if offset + 8 <= data.len() {
                            let bytes: [u8; 8] = data[offset..offset+8].try_into().unwrap_or([0u8; 8]);
                            self.registers[*dest as usize] = i64::from_le_bytes(bytes);
                        }
                    }
                },
                // === Batch Memory Operations ===
                Instruction::MEMCPY { dst, doff, src, soff, len } => {
                    let src_hp = self.registers[*src as usize] as usize;
                    let src_off = self.registers[*soff as usize] as usize;
                    let dst_hp = self.registers[*dst as usize] as usize;
                    let dst_off = self.registers[*doff as usize] as usize;
                    let length = self.registers[*len as usize] as usize;

                    // Read source bytes
                    let bytes: Vec<u8> = if let Some(Some(data)) = self.heap.get(src_hp) {
                        if src_off + length <= data.len() {
                            data[src_off..src_off + length].to_vec()
                        } else { vec![] }
                    } else { vec![] };

                    // Write to destination
                    if !bytes.is_empty() && dst_hp < self.heap.len() {
                        if let Some(data) = &mut self.heap[dst_hp] {
                            if dst_off + length <= data.len() {
                                data[dst_off..dst_off + length].copy_from_slice(&bytes);
                            }
                        }
                    }
                },
                Instruction::HLEN { dest, ptr } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let len = if let Some(Some(data)) = self.heap.get(hp) {
                        data.len() as i64
                    } else { 0 };
                    self.registers[*dest as usize] = len;
                },
                Instruction::SLICE { dest, ptr, off, len } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let offset = self.registers[*off as usize] as usize;
                    let length = self.registers[*len as usize] as usize;

                    let slice: Vec<u8> = if let Some(Some(data)) = self.heap.get(hp) {
                        if offset + length <= data.len() {
                            data[offset..offset + length].to_vec()
                        } else if offset < data.len() {
                            data[offset..].to_vec()
                        } else { vec![] }
                    } else { vec![] };

                    let ptr = self.heap_alloc(slice);
                    self.registers[*dest as usize] = ptr;
                },
                Instruction::MEMSET { ptr, off, len, val } => {
                    let hp = self.registers[*ptr as usize] as usize;
                    let offset = self.registers[*off as usize] as usize;
                    let length = self.registers[*len as usize] as usize;
                    let value = self.registers[*val as usize] as u8;

                    if hp < self.heap.len() {
                        if let Some(data) = &mut self.heap[hp] {
                            let end = std::cmp::min(offset + length, data.len());
                            for i in offset..end {
                                data[i] = value;
                            }
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
                    let hp = self.registers[*vec as usize] as usize;
                    let index = self.registers[*idx as usize] as usize;
                    let value_bits = self.registers[*val as usize] as u64;

                    if hp < self.heap.len() {
                        if let Some(data) = &mut self.heap[hp] {
                            let offset = 8 + index * 8; // Skip dimension header
                            if offset + 8 <= data.len() {
                                data[offset..offset+8].copy_from_slice(&value_bits.to_le_bytes());
                            }
                        }
                    }
                },

                Instruction::VGET { dest, vec, idx } => {
                    let hp = self.registers[*vec as usize] as usize;
                    let index = self.registers[*idx as usize] as usize;

                    if let Some(Some(data)) = self.heap.get(hp) {
                        let offset = 8 + index * 8;
                        if offset + 8 <= data.len() {
                            let bytes: [u8; 8] = data[offset..offset+8].try_into().unwrap_or([0u8; 8]);
                            self.registers[*dest as usize] = i64::from_le_bytes(bytes);
                        }
                    }
                },

                Instruction::VDOT { dest, v1, v2 } => {
                    let hp1 = self.registers[*v1 as usize] as usize;
                    let hp2 = self.registers[*v2 as usize] as usize;

                    let mut dot = 0.0f64;

                    if let (Some(Some(d1)), Some(Some(d2))) = (self.heap.get(hp1), self.heap.get(hp2)) {
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
                    let hp1 = self.registers[*v1 as usize] as usize;
                    let hp2 = self.registers[*v2 as usize] as usize;

                    let mut dot = 0.0f64;
                    let mut mag1 = 0.0f64;
                    let mut mag2 = 0.0f64;

                    if let (Some(Some(d1)), Some(Some(d2))) = (self.heap.get(hp1), self.heap.get(hp2)) {
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
                    let hp = self.registers[*vec as usize] as usize;
                    let mut mag = 0.0f64;

                    if let Some(Some(data)) = self.heap.get(hp) {
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
                    let hp = self.registers[*vec as usize] as usize;

                    // First calculate magnitude
                    let mut mag = 0.0f64;
                    let dims = if let Some(Some(data)) = self.heap.get(hp) {
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
                    if mag > 0.0 && hp < self.heap.len() {
                        if let Some(data) = &mut self.heap[hp] {
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
                    let hp1 = self.registers[240] as usize;  // target vector
                    let hp2 = self.registers[*state as usize] as usize;  // current state vector

                    let mut dot = 0.0f64;
                    let mut mag1 = 0.0f64;
                    let mut mag2 = 0.0f64;

                    if let (Some(Some(d1)), Some(Some(d2))) = (self.heap.get(hp1), self.heap.get(hp2)) {
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
                    let query_ptr = self.registers[*query as usize] as usize;
                    let query_str = if let Some(Some(d)) = self.heap.get(query_ptr) {
                        String::from_utf8_lossy(d).to_string()
                    } else { "".to_string() };

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
        eprintln!("Usage: l0vm <program.json> [--debug]");
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
            "version": "0.3.1",
            "usage": "l0vm <program.l0> [--debug]",
            "description": "L-Zero Virtual Machine (Binary Bytecode)",
            "architecture": {
                "format": "Bincode (Rust Binary)",
                "endianness": "Little Endian (Standard)",
                "isa_definition": "Enum (Untagged)"
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
    // Find the first argument that is not a flag
    let path = args.iter().skip(1).find(|a| !a.starts_with("--")).expect("No program file specified");

    // Detect format by extension
    let program: Vec<Instruction> = if path.ends_with(".json") {
        // JSON format
        let content = fs::read_to_string(path).expect("Failed to read JSON file");
        serde_json::from_str(&content).expect("Failed to parse L-Zero JSON program")
    } else {
        // Binary format (.l0)
        let content = fs::read(path).expect("Failed to read binary file");
        bincode::deserialize(&content).expect("Failed to parse L-Zero Binary Bytecode")
    };

    let mut vm = VM::new();
    vm.execute(&program, debug);
}
