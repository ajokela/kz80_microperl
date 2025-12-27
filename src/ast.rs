//! Abstract Syntax Tree types for MicroPerl

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Integer(i32),
    Float(f64),
    String(String),

    // Variables
    ScalarVar(String),
    ArrayVar(String),
    HashVar(String),

    // Array/Hash access
    ArrayIndex(Box<Expr>, Box<Expr>),   // $arr[idx]
    HashIndex(Box<Expr>, Box<Expr>),    // $hash{key}

    // Binary operations
    BinOp(Box<Expr>, BinOp, Box<Expr>),

    // Unary operations
    UnaryOp(UnaryOp, Box<Expr>),

    // Pre/Post increment/decrement
    PreIncrement(Box<Expr>),
    PreDecrement(Box<Expr>),
    PostIncrement(Box<Expr>),
    PostDecrement(Box<Expr>),

    // Assignment
    Assign(Box<Expr>, Box<Expr>),
    OpAssign(Box<Expr>, BinOp, Box<Expr>),  // +=, -=, etc.

    // Function call
    Call(String, Vec<Expr>),

    // Method call
    MethodCall(Box<Expr>, String, Vec<Expr>),

    // List/Array constructor
    List(Vec<Expr>),

    // Hash constructor
    Hash(Vec<(Expr, Expr)>),

    // Range
    Range(Box<Expr>, Box<Expr>),

    // Ternary
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),

    // Regex match
    Match(Box<Expr>, String, String),       // expr =~ /pattern/flags
    NotMatch(Box<Expr>, String, String),    // expr !~ /pattern/flags

    // Reference
    Ref(Box<Expr>),

    // Dereference
    Deref(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // String
    Concat,

    // Numeric comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Cmp,

    // String comparison
    StrEq,
    StrNe,
    StrLt,
    StrGt,
    StrLe,
    StrGe,
    StrCmp,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Ref,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // Expression statement
    Expr(Expr),

    // Variable declaration
    My(Vec<String>, Option<Expr>),      // my ($x, $y) = ...
    Our(Vec<String>, Option<Expr>),     // our ($x, $y) = ...

    // Control flow
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        elsif_blocks: Vec<(Expr, Vec<Stmt>)>,
        else_block: Option<Vec<Stmt>>,
    },
    Unless {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Until {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    Foreach {
        var: String,
        list: Expr,
        body: Vec<Stmt>,
    },

    // Loop control
    Last,
    Next,
    Return(Option<Expr>),

    // Subroutine definition
    Sub {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    // Print statements
    Print(Vec<Expr>),
    Say(Vec<Expr>),

    // Block
    Block(Vec<Stmt>),

    // Use/Package (minimal support)
    Use(String),
    Package(String),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

impl Program {
    pub fn new() -> Self {
        Program { statements: Vec::new() }
    }
}
