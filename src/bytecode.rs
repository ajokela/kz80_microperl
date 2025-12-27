//! Bytecode instruction set for MicroPerl Z80 VM
//!
//! The MicroPerl VM is a stack-based virtual machine optimized for Z80.
//! All values are 16-bit (matching Z80's register pairs).
//! Strings and arrays are heap-allocated with 16-bit pointers.

/// Bytecode opcodes (1 byte each)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Op {
    // Stack operations
    Nop = 0x00,
    Push = 0x01,        // Push 16-bit immediate: PUSH lo hi
    PushByte = 0x02,    // Push 8-bit immediate (sign-extended): PUSHB val
    Pop = 0x03,         // Pop and discard top of stack
    Dup = 0x04,         // Duplicate top of stack
    Swap = 0x05,        // Swap top two stack values
    Over = 0x06,        // Copy second item to top

    // Local variables (indexed from frame pointer)
    LoadLocal = 0x10,   // Load local variable: LDLOC idx
    StoreLocal = 0x11,  // Store to local variable: STLOC idx

    // Global variables (indexed from global table)
    LoadGlobal = 0x12,  // Load global variable: LDGLOB idx_lo idx_hi
    StoreGlobal = 0x13, // Store to global variable: STGLOB idx_lo idx_hi

    // String operations
    PushStr = 0x18,     // Push string constant: PUSHSTR idx_lo idx_hi
    StrLen = 0x19,      // Get string length
    StrCat = 0x1A,      // Concatenate two strings
    StrIdx = 0x1B,      // Get character at index
    StrCmp = 0x1C,      // Compare strings (-1, 0, 1)
    Substr = 0x1D,      // Substring: substr(str, start, len)

    // Array operations
    NewArray = 0x20,    // Create new array: NEWARR size
    ArrLen = 0x21,      // Get array length
    ArrGet = 0x22,      // Get array element: arr[idx]
    ArrSet = 0x23,      // Set array element: arr[idx] = val
    ArrPush = 0x24,     // Push onto array end
    ArrPop = 0x25,      // Pop from array end

    // Hash operations
    NewHash = 0x28,     // Create new hash
    HashGet = 0x29,     // Get hash value: hash{key}
    HashSet = 0x2A,     // Set hash value: hash{key} = val
    HashDel = 0x2B,     // Delete hash key
    HashKeys = 0x2C,    // Get array of keys

    // Arithmetic (operate on top two stack values)
    Add = 0x30,         // a + b
    Sub = 0x31,         // a - b
    Mul = 0x32,         // a * b
    Div = 0x33,         // a / b
    Mod = 0x34,         // a % b
    Neg = 0x35,         // -a
    Inc = 0x36,         // a + 1
    Dec = 0x37,         // a - 1

    // Bitwise
    BitAnd = 0x38,      // a & b
    BitOr = 0x39,       // a | b
    BitXor = 0x3A,      // a ^ b
    BitNot = 0x3B,      // ~a
    Shl = 0x3C,         // a << b
    Shr = 0x3D,         // a >> b

    // Comparison (push 1 for true, 0 for false)
    CmpEq = 0x40,       // a == b
    CmpNe = 0x41,       // a != b
    CmpLt = 0x42,       // a < b
    CmpGt = 0x43,       // a > b
    CmpLe = 0x44,       // a <= b
    CmpGe = 0x45,       // a >= b
    Cmp = 0x46,         // a <=> b (-1, 0, 1)

    // String comparison
    StrEq = 0x48,       // eq
    StrNe = 0x49,       // ne
    StrLt = 0x4A,       // lt
    StrGt = 0x4B,       // gt
    StrLe = 0x4C,       // le
    StrGe = 0x4D,       // ge

    // Logical
    Not = 0x50,         // !a
    And = 0x51,         // a && b (short-circuit)
    Or = 0x52,          // a || b (short-circuit)

    // Control flow
    Jump = 0x60,        // Unconditional jump: JMP addr_lo addr_hi
    JumpIf = 0x61,      // Jump if true: JIF addr_lo addr_hi
    JumpIfNot = 0x62,   // Jump if false: JIFN addr_lo addr_hi
    JumpIfDef = 0x63,   // Jump if defined

    // Subroutine calls
    Call = 0x68,        // Call subroutine: CALL addr_lo addr_hi
    CallNative = 0x69,  // Call native function: CALLNAT idx
    Return = 0x6A,      // Return from subroutine
    ReturnVal = 0x6B,   // Return with value

    // Frame management
    EnterFrame = 0x70,  // Set up new stack frame: ENTER num_locals
    LeaveFrame = 0x71,  // Tear down stack frame

    // I/O
    Print = 0x78,       // Print top of stack (auto-detect type)
    PrintStr = 0x79,    // Print as string
    PrintNum = 0x7A,    // Print as number
    PrintChar = 0x7B,   // Print as character
    PrintLn = 0x7C,     // Print newline
    Input = 0x7D,       // Read line of input
    InputChar = 0x7E,   // Read single character

    // Type operations
    ToNum = 0x80,       // Convert to number
    ToStr = 0x81,       // Convert to string
    TypeOf = 0x82,      // Get type (0=undef, 1=num, 2=str, 3=arr, 4=hash)
    IsDef = 0x83,       // Check if defined

    // Regex (simplified)
    Match = 0x88,       // Match string against pattern
    Subst = 0x89,       // Substitute pattern

    // Special
    Halt = 0xF0,        // Stop execution
    Debug = 0xFE,       // Debug breakpoint
    Invalid = 0xFF,     // Invalid opcode
}

impl Op {
    /// Get the size of the instruction including operands
    pub fn size(&self) -> usize {
        match self {
            // No operands
            Op::Nop | Op::Pop | Op::Dup | Op::Swap | Op::Over |
            Op::StrLen | Op::StrCat | Op::StrIdx | Op::StrCmp | Op::Substr |
            Op::ArrLen | Op::ArrGet | Op::ArrSet | Op::ArrPush | Op::ArrPop |
            Op::NewHash | Op::HashGet | Op::HashSet | Op::HashDel | Op::HashKeys |
            Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod | Op::Neg | Op::Inc | Op::Dec |
            Op::BitAnd | Op::BitOr | Op::BitXor | Op::BitNot | Op::Shl | Op::Shr |
            Op::CmpEq | Op::CmpNe | Op::CmpLt | Op::CmpGt | Op::CmpLe | Op::CmpGe | Op::Cmp |
            Op::StrEq | Op::StrNe | Op::StrLt | Op::StrGt | Op::StrLe | Op::StrGe |
            Op::Not | Op::And | Op::Or |
            Op::Return | Op::ReturnVal | Op::LeaveFrame |
            Op::Print | Op::PrintStr | Op::PrintNum | Op::PrintChar | Op::PrintLn |
            Op::Input | Op::InputChar |
            Op::ToNum | Op::ToStr | Op::TypeOf | Op::IsDef |
            Op::Match | Op::Subst |
            Op::Halt | Op::Debug | Op::Invalid => 1,

            // 1-byte operand
            Op::PushByte | Op::LoadLocal | Op::StoreLocal |
            Op::NewArray | Op::CallNative | Op::EnterFrame => 2,

            // 2-byte operand
            Op::Push | Op::LoadGlobal | Op::StoreGlobal | Op::PushStr |
            Op::Jump | Op::JumpIf | Op::JumpIfNot | Op::JumpIfDef | Op::Call => 3,
        }
    }

    /// Convert from byte
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x00 => Op::Nop,
            0x01 => Op::Push,
            0x02 => Op::PushByte,
            0x03 => Op::Pop,
            0x04 => Op::Dup,
            0x05 => Op::Swap,
            0x06 => Op::Over,
            0x10 => Op::LoadLocal,
            0x11 => Op::StoreLocal,
            0x12 => Op::LoadGlobal,
            0x13 => Op::StoreGlobal,
            0x18 => Op::PushStr,
            0x19 => Op::StrLen,
            0x1A => Op::StrCat,
            0x1B => Op::StrIdx,
            0x1C => Op::StrCmp,
            0x1D => Op::Substr,
            0x20 => Op::NewArray,
            0x21 => Op::ArrLen,
            0x22 => Op::ArrGet,
            0x23 => Op::ArrSet,
            0x24 => Op::ArrPush,
            0x25 => Op::ArrPop,
            0x28 => Op::NewHash,
            0x29 => Op::HashGet,
            0x2A => Op::HashSet,
            0x2B => Op::HashDel,
            0x2C => Op::HashKeys,
            0x30 => Op::Add,
            0x31 => Op::Sub,
            0x32 => Op::Mul,
            0x33 => Op::Div,
            0x34 => Op::Mod,
            0x35 => Op::Neg,
            0x36 => Op::Inc,
            0x37 => Op::Dec,
            0x38 => Op::BitAnd,
            0x39 => Op::BitOr,
            0x3A => Op::BitXor,
            0x3B => Op::BitNot,
            0x3C => Op::Shl,
            0x3D => Op::Shr,
            0x40 => Op::CmpEq,
            0x41 => Op::CmpNe,
            0x42 => Op::CmpLt,
            0x43 => Op::CmpGt,
            0x44 => Op::CmpLe,
            0x45 => Op::CmpGe,
            0x46 => Op::Cmp,
            0x48 => Op::StrEq,
            0x49 => Op::StrNe,
            0x4A => Op::StrLt,
            0x4B => Op::StrGt,
            0x4C => Op::StrLe,
            0x4D => Op::StrGe,
            0x50 => Op::Not,
            0x51 => Op::And,
            0x52 => Op::Or,
            0x60 => Op::Jump,
            0x61 => Op::JumpIf,
            0x62 => Op::JumpIfNot,
            0x63 => Op::JumpIfDef,
            0x68 => Op::Call,
            0x69 => Op::CallNative,
            0x6A => Op::Return,
            0x6B => Op::ReturnVal,
            0x70 => Op::EnterFrame,
            0x71 => Op::LeaveFrame,
            0x78 => Op::Print,
            0x79 => Op::PrintStr,
            0x7A => Op::PrintNum,
            0x7B => Op::PrintChar,
            0x7C => Op::PrintLn,
            0x7D => Op::Input,
            0x7E => Op::InputChar,
            0x80 => Op::ToNum,
            0x81 => Op::ToStr,
            0x82 => Op::TypeOf,
            0x83 => Op::IsDef,
            0x88 => Op::Match,
            0x89 => Op::Subst,
            0xF0 => Op::Halt,
            0xFE => Op::Debug,
            _ => Op::Invalid,
        }
    }
}

/// Native function IDs for built-in functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NativeFunc {
    // String functions
    Length = 0,
    Substr = 1,
    Index = 2,
    Rindex = 3,
    Lc = 4,
    Uc = 5,
    Chr = 6,
    Ord = 7,
    Sprintf = 8,

    // Array functions
    Push = 16,
    Pop = 17,
    Shift = 18,
    Unshift = 19,
    Reverse = 20,
    Sort = 21,
    Join = 22,
    Split = 23,

    // Hash functions
    Keys = 32,
    Values = 33,
    Exists = 34,
    Delete = 35,

    // Math functions
    Abs = 48,
    Int = 49,
    Rand = 50,
    Srand = 51,

    // I/O functions
    Open = 64,
    Close = 65,
    Read = 66,
    Write = 67,
    Eof = 68,

    // Misc
    Defined = 80,
    Ref = 81,
    Die = 82,
    Exit = 83,
    Sleep = 84,
    Time = 85,
}

/// Compiled bytecode module
#[derive(Debug, Clone)]
pub struct Module {
    /// String constant pool
    pub strings: Vec<String>,

    /// Global variable names (for debugging)
    pub globals: Vec<String>,

    /// Subroutine table: (name, address, num_params)
    pub subs: Vec<(String, u16, u8)>,

    /// Bytecode
    pub code: Vec<u8>,

    /// Entry point address
    pub entry: u16,
}

impl Module {
    pub fn new() -> Self {
        Module {
            strings: Vec::new(),
            globals: Vec::new(),
            subs: Vec::new(),
            code: Vec::new(),
            entry: 0,
        }
    }

    /// Add a string to the constant pool, return its index
    pub fn add_string(&mut self, s: &str) -> u16 {
        if let Some(idx) = self.strings.iter().position(|x| x == s) {
            return idx as u16;
        }
        let idx = self.strings.len() as u16;
        self.strings.push(s.to_string());
        idx
    }

    /// Emit an opcode
    pub fn emit(&mut self, op: Op) {
        self.code.push(op as u8);
    }

    /// Emit an opcode with 1-byte operand
    pub fn emit_byte(&mut self, op: Op, b: u8) {
        self.code.push(op as u8);
        self.code.push(b);
    }

    /// Emit an opcode with 2-byte operand (little-endian)
    pub fn emit_word(&mut self, op: Op, w: u16) {
        self.code.push(op as u8);
        self.code.push(w as u8);
        self.code.push((w >> 8) as u8);
    }

    /// Current code position
    pub fn pos(&self) -> u16 {
        self.code.len() as u16
    }

    /// Patch a 16-bit address at the given position
    pub fn patch_addr(&mut self, pos: usize, addr: u16) {
        self.code[pos] = addr as u8;
        self.code[pos + 1] = (addr >> 8) as u8;
    }
}
