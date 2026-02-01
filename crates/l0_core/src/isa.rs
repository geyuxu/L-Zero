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
    /// Compare two integers: sets flags based on R[r1] vs R[r2]
    CMP { r1: u8, r2: u8 },
    /// Compare two strings: sets flags based on strcmp(heap[R[s1]], heap[R[s2]])
    SCMP { s1: u8, s2: u8 },
    /// Unconditional Jump (Absolute Line)
    JMP { target: usize }, 
    BEQ { target: usize },
    BGT { target: usize },
    BLT { target: usize },

    // === 6. Memory (Safe Heap) ===
    /// Allocate fixed-size memory: R[dest] = heap_alloc(size bytes)
    NEW { dest: u8, size: usize },
    /// Allocate dynamic-size memory: R[dest] = heap_alloc(R[size_reg] bytes)
    NEWR { dest: u8, size_reg: u8 },
    FREE { ptr: u8 },
    READ { dest: u8, ptr: u8, offset: usize },
    WRITE { ptr: u8, offset: usize, val: u8 },

    // === 7. Extensions ===

    /// Regex match: R[dest] = 1 if R[text] matches R[pat], else 0
    REGEX { dest: u8, pat: u8, text: u8 },

    /// Execute Tool (ID, ArgReg, DestReg)
    TEXEC { tool: u16, arg: u8, dest: u8 },

    /// Integer to String (dest = str(src))
    ITOA { dest: u8, src: u8 },

    /// String to Integer (dest = int(src)), 0 on error
    ATOI { dest: u8, src: u8 },

    /// Read with register offset: R[dest] = heap[R[ptr]][R[off]]
    READR { dest: u8, ptr: u8, off: u8 },

    /// Write with register offset: heap[R[ptr]][R[off]] = R[val]
    WRITER { ptr: u8, off: u8, val: u8 },

    /// String concatenation: R[dest] = R[s1] + R[s2]
    SCAT { dest: u8, s1: u8, s2: u8 },

    /// Store i64 register to heap (8 bytes): heap[R[ptr]][off*8..off*8+8] = R[val]
    STORE64 { ptr: u8, off: u8, val: u8 },

    /// Load i64 from heap to register: R[dest] = heap[R[ptr]][off*8..off*8+8]
    LOAD64 { dest: u8, ptr: u8, off: u8 },

    // === 8. Batch Memory Operations ===

    /// Memory copy: copy R[len] bytes from heap[R[src]][R[soff]] to heap[R[dst]][R[doff]]
    MEMCPY { dst: u8, doff: u8, src: u8, soff: u8, len: u8 },

    /// Get heap allocation length: R[dest] = len(heap[R[ptr]])
    HLEN { dest: u8, ptr: u8 },

    /// Bulk read N bytes: R[dest] = new heap containing heap[R[ptr]][R[off]..R[off]+R[len]]
    SLICE { dest: u8, ptr: u8, off: u8, len: u8 },

    /// Memory set: fill heap[R[ptr]][R[off]..R[off]+R[len]] with byte R[val]
    MEMSET { ptr: u8, off: u8, len: u8, val: u8 },

    // === 9. Vector Operations (Semantic Computing) ===
    // Vectors stored as: [dims:i64][f64 as bits][f64 as bits]...
    // Results scaled by 1_000_000 for integer precision

    /// Allocate vector: R[dest] = new vector of R[dims] dimensions (initialized to 0)
    VNEW { dest: u8, dims: u8 },

    /// Set vector element: vec[R[idx]] = f64::from_bits(R[val])
    VSET { vec: u8, idx: u8, val: u8 },

    /// Get vector element: R[dest] = vec[R[idx]] as i64 bits
    VGET { dest: u8, vec: u8, idx: u8 },

    /// Vector dot product: R[dest] = dot(v1, v2) * 1_000_000
    VDOT { dest: u8, v1: u8, v2: u8 },

    /// Cosine similarity: R[dest] = cos_sim(v1, v2) * 1_000_000 (range: -1M to +1M)
    VSIM { dest: u8, v1: u8, v2: u8 },

    /// Vector magnitude: R[dest] = |v| * 1_000_000
    VMAG { dest: u8, vec: u8 },

    /// Normalize vector in-place: v = v / |v|
    VNORM { vec: u8 },

    // === 10. Governance (Semantic Drift Control) ===
    // Enables autonomous AI agents to detect and respond to semantic drift
    // Reserved registers: R240=target_vec, R241=state_vec, R242=threshold, R243=last_sim

    /// Latch target vector: R240 = R[target], R242 = R[threshold] (threshold * 1M)
    LATCH { target: u8, threshold: u8 },

    /// Check drift: compute VSIM(R240, R[state]), if < R242 then TRAP
    GUARD { state: u8 },

    /// Trap to supervisor: suspend VM, emit context JSON to stdout, wait for correction
    TRAP { code: u8 },

    /// Yield to supervisor: emit query, receive response into R[dest]
    YIELD { query: u8, dest: u8 },

    // === 11. Memory Watermark (Arena-style Reset) ===

    /// Mark current heap position: R[dest] = current_heap_watermark
    MARK { dest: u8 },

    /// Reset heap to watermark: free all allocations after R[limit]
    RESET { limit: u8 }
}
