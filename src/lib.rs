// L-0 Language Core Library
use serde::{Deserialize, Serialize};

// Macro to define the ISA and generate reflection data automatically
macro_rules! define_isa {
    (
        $(
            $(#[doc = $doc:expr])* 
            $name:ident $( ( $($t_arg:ty),* ) )? $( { $($s_arg:ident : $s_type:ty),* } )?
        ),* $(,)?
    ) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub enum Instruction {
            $(
                $(#[doc = $doc])*
                $name $( ( $($t_arg),* ) )? $( { $($s_arg : $s_type),* } )?
            ),*
        }

        #[derive(Serialize, Debug)]
        pub struct ArgInfo {
            pub name: String,
            pub type_name: String,
        }

        #[derive(Serialize, Debug)]
        pub struct OpInfo {
            pub name: String,
            pub ordinal: usize,
            pub format: String,
            pub args: Vec<ArgInfo>,
            pub description: String,
        }

        impl Instruction {
            pub fn introspection() -> Vec<OpInfo> {
                let mut ops = Vec::new();
                let mut _idx = 0;
                $(
                    {
                        #[allow(unused_mut)]
                        let mut args_info = Vec::new();
                        let mut fmt = "Unit".to_string();

                        // Tuple Variants
                        $(
                            fmt = "Tuple".to_string();
                            let types = vec![ $( stringify!($t_arg) ),* ];
                            for (i, t) in types.iter().enumerate() {
                                args_info.push(ArgInfo {
                                    name: format!("arg{}", i),
                                    type_name: t.to_string(),
                                });
                            }
                        )?

                        // Struct Variants
                        $(
                            fmt = "Struct".to_string();
                            $(
                                args_info.push(ArgInfo {
                                    name: stringify!($s_arg).to_string(),
                                    type_name: stringify!($s_type).to_string(),
                                });
                            )*
                        )?

                        let doc_str = concat!($( $doc, "\n", )*).trim().to_string();

                        ops.push(OpInfo {
                            name: stringify!($name).to_string(),
                            ordinal: _idx,
                            format: fmt,
                            args: args_info,
                            description: doc_str,
                        });
                        _idx += 1;
                    }
                )*
                ops
            }
        }
    };
}

define_isa! {
    // === 1. System ===
    /// No Operation
    NOP,
    /// Stop Execution
    HALT,
    /// Panic with message
    PANIC(String), 
    /// Dump VM State (Regs/Heap)
    DUMP,
    /// Get current Gas limit into reg
    GAS(u8),       
    /// Assert register is not zero
    ASSERT(u8),    

    // === 2. Registers ===
    /// Set register to immediate value
    SET { reg: u8, val: i64 },
    /// Set register to string (allocates)
    SETS { reg: u8, val: String }, 
    /// Copy register
    MOV { dest: u8, src: u8 },
    /// Swap two registers
    SWAP { r1: u8, r2: u8 },

    // === 3. Arithmetic (Integer Only) ===
    ADD { dest: u8, s1: u8, s2: u8 },
    SUB { dest: u8, s1: u8, s2: u8 },
    MUL { dest: u8, s1: u8, s2: u8 },
    DIV { dest: u8, s1: u8, s2: u8 },
    MOD { dest: u8, s1: u8, s2: u8 },

    // === 4. Logic ===
    AND { dest: u8, s1: u8, s2: u8 },
    OR  { dest: u8, s1: u8, s2: u8 },
    XOR { dest: u8, s1: u8, s2: u8 },
    NOT { dest: u8, src: u8 },

    // === 5. Control Flow ===
    CMP { r1: u8, r2: u8 },
    /// Unconditional Jump (Absolute Line)
    JMP { target: usize }, 
    BEQ { target: usize },
    BGT { target: usize },
    BLT { target: usize },

    // === 6. Memory (Safe Heap) ===
    NEW { dest: u8, size: usize },
    FREE { ptr: u8 },
    READ { dest: u8, ptr: u8, offset: usize },
    WRITE { ptr: u8, offset: usize, val: u8 },

    // === 7. Extensions ===
    // (REGEX is temporarily disabled)
    // REGEX { dest: u8, pat: u8, text: u8 }, 
    
    /// Execute Tool (ID, ArgReg, DestReg)
    TEXEC { tool: u16, arg: u8, dest: u8 },

    /// Integer to String (dest = str(src))
    ITOA { dest: u8, src: u8 },

    /// String to Integer (dest = int(src)), 0 on error
    ATOI { dest: u8, src: u8 },

    /// Read with register offset: R[dest] = heap[R[ptr]][R[off]]
    READR { dest: u8, ptr: u8, off: u8 },

    /// Write with register offset: heap[R[ptr]][R[off]] = R[val]
    WRITER { ptr: u8, off: u8, val: u8 }
}
