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

             // Construct JSON Request
             let req = format!("{{ \"method\": \"{}\", \"args\": [\"{}\"] }}", method, arg);
             
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
                         // Parse Response: { "ok": true, "value": "..." }
                         if let Ok(resp) = serde_json::from_str::<Value>(&line) {
                             if resp["ok"].as_bool() == Some(true) {
                                 return resp["value"].as_str().unwrap_or("").to_string();
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
                    println!("Regs: {:?}", &self.registers[0..16]);
                    println!("Heap Size: {}", self.heap.len());
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
                /*
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
                */
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
