use l0_core::Instruction;
use std::env;
use std::fs;
use std::io::{Read, BufRead, BufReader};
use std::collections::HashMap;

// Intermediate Instruction Representation
#[derive(Debug)]
struct RawInstr {
    line_num: usize,
    op: String,
    args: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: l0asm <file.l0>");
        std::process::exit(1);
    }

    let path = &args[1];
    let file = fs::File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);

    // --- Pass 1: Parse & Collect Labels ---
    let mut instructions: Vec<RawInstr> = Vec::new();
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut current_index = 0;

    for (line_idx, line_r) in reader.lines().enumerate() {
        let line = line_r.expect("Failed to read line");
        let trim = line.trim();
        
        // Skip comments and empty lines
        if trim.is_empty() || trim.starts_with('#') || trim.starts_with("//") {
            continue;
        }

        // Handle Inline Comments "OP ... # comment"
        let code_part = trim.split('#').next().unwrap().split("//").next().unwrap().trim();
        if code_part.is_empty() { continue; }

        // Check for Label "Label:"
        if code_part.ends_with(':') {
            let label_name = &code_part[..code_part.len()-1];
            if labels.contains_key(label_name) {
                eprintln!("Error: Duplicate Label '{}' at line {}", label_name, line_idx + 1);
                std::process::exit(1);
            }
            labels.insert(label_name.to_string(), current_index);
            continue;
        }

        // Parse Instruction "OP arg1, arg2"
        let parts: Vec<&str> = code_part.splitn(2, |c: char| c.is_whitespace()).collect();
        let op = parts[0].to_uppercase();
        
        // Parse Args (comma separated)
        let args_vec: Vec<String> = if parts.len() > 1 {
            // Split by comma, but be careful with strings? 
            // Minimal: Split by comma. String args like "Hello, World" might break.
            // Requirement for strings: "SETS".
            // Minimal parser: naive split.
            // To support "SETS 1, "Hello, World"", we need slightly better parsing.
            // But let's stick to "Simple". If needed, users can avoid commas in strings or we patch.
            // Actually, L-0 strings often don't have commas. Or we can just grab the rest for SETS/PANIC.
            
            // Special handling for SETS/PANIC who take String?
            // "SETS 1, Hello World" -> args: ["1", "Hello World"]
            // "PANIC Error" -> args: ["Error"]
            
            // State machine parser for proper handling of commas in strings
            let rest = parts[1];
            parse_args(rest)
        } else {
            Vec::new()
        };

        instructions.push(RawInstr {
            line_num: line_idx + 1,
            op,
            args: args_vec,
        });
        current_index += 1;
    }

    // --- Pass 2: Resolve & Build ---
    let mut program: Vec<Instruction> = Vec::new();

    for raw in instructions {
        let op = raw.op.as_str();
        let args = &raw.args;

        let instr = match op {
            // System
            "NOP" => Instruction::NOP,
            "HALT" => Instruction::HALT,
            "DUMP" => Instruction::DUMP,
            "PANIC" => {
                // Tuple variant PANIC(String)
                // If arg contains quotes, strip them? Minimal: take as is.
                let msg = args.join(", "); // Re-join if split by accident
                 Instruction::PANIC(msg.replace("\"", "")) 
            },
            "GAS" => Instruction::GAS(parse_u8(args, 0)),
            "ASSERT" => Instruction::ASSERT(parse_u8(args, 0)),

            // Registers
            "SET" => Instruction::SET { reg: parse_u8(args, 0), val: parse_i64(args, 1) },
            "SETS" => Instruction::SETS { reg: parse_u8(args, 0), val: parse_str(args, 1) }, 
            "MOV" => Instruction::MOV { dest: parse_u8(args, 0), src: parse_u8(args, 1) },
            "SWAP" => Instruction::SWAP { r1: parse_u8(args, 0), r2: parse_u8(args, 1) },

            // Math
            "ADD" => Instruction::ADD { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "SUB" => Instruction::SUB { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "MUL" => Instruction::MUL { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "DIV" => Instruction::DIV { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "MOD" => Instruction::MOD { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },

            // Logic
            "AND" => Instruction::AND { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "OR" =>  Instruction::OR  { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "XOR" => Instruction::XOR { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "NOT" => Instruction::NOT { dest: parse_u8(args, 0), src: parse_u8(args, 1) },

            // Control
            "CMP" => Instruction::CMP { r1: parse_u8(args, 0), r2: parse_u8(args, 1) },
            "SCMP" => Instruction::SCMP { s1: parse_u8(args, 0), s2: parse_u8(args, 1) },

            // Jumps with Label Resolution
            "JMP" => Instruction::JMP { target: resolve_label(&args[0], &labels) },
            "BEQ" => Instruction::BEQ { target: resolve_label(&args[0], &labels) },
            "BGT" => Instruction::BGT { target: resolve_label(&args[0], &labels) },
            "BLT" => Instruction::BLT { target: resolve_label(&args[0], &labels) },

            // Memory
            "NEW" => Instruction::NEW { dest: parse_u8(args, 0), size: parse_usize(args, 1) },
            "NEWR" => Instruction::NEWR { dest: parse_u8(args, 0), size_reg: parse_u8(args, 1) },
            "FREE" => Instruction::FREE { ptr: parse_u8(args, 0) },
            "READ" => Instruction::READ { dest: parse_u8(args, 0), ptr: parse_u8(args, 1), offset: parse_usize(args, 2) },
            "WRITE" => Instruction::WRITE { ptr: parse_u8(args, 0), offset: parse_usize(args, 1), val: parse_u8(args, 2) },
            
            // Extension
            "REGEX" => Instruction::REGEX { dest: parse_u8(args, 0), pat: parse_u8(args, 1), text: parse_u8(args, 2) },
            "TEXEC" => Instruction::TEXEC { tool: parse_u16_hex(args, 0), arg: parse_u8(args, 1), dest: parse_u8(args, 2) },
            "ITOA" => Instruction::ITOA { dest: parse_u8(args, 0), src: parse_u8(args, 1) },
            "ATOI" => Instruction::ATOI { dest: parse_u8(args, 0), src: parse_u8(args, 1) },
            "READR" => Instruction::READR { dest: parse_u8(args, 0), ptr: parse_u8(args, 1), off: parse_u8(args, 2) },
            "WRITER" => Instruction::WRITER { ptr: parse_u8(args, 0), off: parse_u8(args, 1), val: parse_u8(args, 2) },
            "SCAT" => Instruction::SCAT { dest: parse_u8(args, 0), s1: parse_u8(args, 1), s2: parse_u8(args, 2) },
            "STORE64" => Instruction::STORE64 { ptr: parse_u8(args, 0), off: parse_u8(args, 1), val: parse_u8(args, 2) },
            "LOAD64" => Instruction::LOAD64 { dest: parse_u8(args, 0), ptr: parse_u8(args, 1), off: parse_u8(args, 2) },

            // Batch Memory Operations
            "MEMCPY" => Instruction::MEMCPY { dst: parse_u8(args, 0), doff: parse_u8(args, 1), src: parse_u8(args, 2), soff: parse_u8(args, 3), len: parse_u8(args, 4) },
            "HLEN" => Instruction::HLEN { dest: parse_u8(args, 0), ptr: parse_u8(args, 1) },
            "SLICE" => Instruction::SLICE { dest: parse_u8(args, 0), ptr: parse_u8(args, 1), off: parse_u8(args, 2), len: parse_u8(args, 3) },
            "MEMSET" => Instruction::MEMSET { ptr: parse_u8(args, 0), off: parse_u8(args, 1), len: parse_u8(args, 2), val: parse_u8(args, 3) },

            // Vector Operations (Semantic Computing)
            "VNEW" => Instruction::VNEW { dest: parse_u8(args, 0), dims: parse_u8(args, 1) },
            "VSET" => Instruction::VSET { vec: parse_u8(args, 0), idx: parse_u8(args, 1), val: parse_u8(args, 2) },
            "VGET" => Instruction::VGET { dest: parse_u8(args, 0), vec: parse_u8(args, 1), idx: parse_u8(args, 2) },
            "VDOT" => Instruction::VDOT { dest: parse_u8(args, 0), v1: parse_u8(args, 1), v2: parse_u8(args, 2) },
            "VSIM" => Instruction::VSIM { dest: parse_u8(args, 0), v1: parse_u8(args, 1), v2: parse_u8(args, 2) },
            "VMAG" => Instruction::VMAG { dest: parse_u8(args, 0), vec: parse_u8(args, 1) },
            "VNORM" => Instruction::VNORM { vec: parse_u8(args, 0) },

            // Governance (Semantic Drift Control)
            "LATCH" => Instruction::LATCH { target: parse_u8(args, 0), threshold: parse_u8(args, 1) },
            "GUARD" => Instruction::GUARD { state: parse_u8(args, 0) },
            "TRAP" => Instruction::TRAP { code: parse_u8(args, 0) },
            "YIELD" => Instruction::YIELD { query: parse_u8(args, 0), dest: parse_u8(args, 1) },

            // Memory Watermark (Arena-style Reset)
            "MARK" => Instruction::MARK { dest: parse_u8(args, 0) },
            "RESET" => Instruction::RESET { limit: parse_u8(args, 0) },

            _ => {
                eprintln!("Error: Unknown Opcode '{}' at line {}", op, raw.line_num);
                std::process::exit(1);
            }
        };
        program.push(instr);
    }
    
    // Output Binary (Bincode)
    let encoded: Vec<u8> = bincode::serialize(&program).expect("Failed to serialize to binary");
    
    // Write to stdout (for piping) or file
    // If output is piped, use write_all to stdout. Note: Don't use println! for binary.
    use std::io::Write;
    std::io::stdout().write_all(&encoded).expect("Failed to write binary to stdout");
}

// --- Helpers ---

/// State machine parser for argument parsing
/// Handles commas inside quoted strings: `"Hello, World"` -> preserves comma
/// Supports escape sequences: `\"` inside strings
fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            // Handle escape sequences
            match c {
                'n' => current.push('\n'),
                'r' => current.push('\r'),
                't' => current.push('\t'),
                _ => current.push(c), // \", \\, etc.
            }
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => {
                escape_next = true;
            }
            '"' => {
                in_string = !in_string;
                // Don't add quotes to output - they're delimiters
            }
            ',' if !in_string => {
                // End of argument
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    args.push(trimmed);
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Don't forget the last argument
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        args.push(trimmed);
    }

    args
}

fn parse_u8(args: &[String], idx: usize) -> u8 {
    args.get(idx).expect("Missing Argument").parse().expect("Invalid u8")
}

fn parse_u16_hex(args: &[String], idx: usize) -> u16 {
    let s = args.get(idx).expect("Missing Argument");
    if s.starts_with("0x") {
        u16::from_str_radix(&s[2..], 16).expect("Invalid Hex u16")
    } else {
        s.parse().expect("Invalid u16")
    }
}

fn parse_i64(args: &[String], idx: usize) -> i64 {
    args.get(idx).expect("Missing Argument").parse().expect("Invalid i64")
}

fn parse_usize(args: &[String], idx: usize) -> usize {
    args.get(idx).expect("Missing Argument").parse().expect("Invalid usize")
}

fn parse_str(args: &[String], idx: usize) -> String {
    // Get string argument - quotes already stripped by parse_args
    // For SETS, rejoin remaining args if there are multiple (handles unquoted strings with commas)
    if idx < args.len() {
        args[idx..].join(", ")
    } else {
        panic!("Missing string argument at index {}", idx);
    }
}

fn resolve_label(target: &str, labels: &HashMap<String, usize>) -> usize {
    // Try to parse as explicit integer first
    if let Ok(idx) = target.parse::<usize>() {
        return idx;
    }
    // Lookup label
    *labels.get(target).unwrap_or_else(|| {
        eprintln!("Error: Undefined Label '{}'", target);
        std::process::exit(1);
    })
}
