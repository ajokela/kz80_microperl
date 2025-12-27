//! Token types for MicroPerl lexer

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i32),
    Float(f64),
    String(String),
    Regex(String, String), // pattern, flags

    // Identifiers and variables
    ScalarVar(String),  // $name
    ArrayVar(String),   // @name
    HashVar(String),    // %name
    Ident(String),      // bareword/function name

    // Keywords
    My,
    Our,
    Sub,
    If,
    Elsif,
    Else,
    Unless,
    While,
    Until,
    For,
    Foreach,
    Last,
    Next,
    Return,
    Print,
    Say,
    Use,
    Package,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DoubleStar,     // **
    Dot,            // . (concatenation)
    DotEquals,      // .=

    // Comparison
    Eq,             // ==
    Ne,             // !=
    Lt,             // <
    Gt,             // >
    Le,             // <=
    Ge,             // >=
    Cmp,            // <=>
    StrEq,          // eq
    StrNe,          // ne
    StrLt,          // lt
    StrGt,          // gt
    StrLe,          // le
    StrGe,          // ge
    StrCmp,         // cmp
    Match,          // =~
    NotMatch,       // !~

    // Logical
    And,            // &&
    Or,             // ||
    Not,            // !
    AndWord,        // and
    OrWord,         // or
    NotWord,        // not

    // Bitwise
    BitAnd,         // &
    BitOr,          // |
    BitXor,         // ^
    BitNot,         // ~
    ShiftLeft,      // <<
    ShiftRight,     // >>

    // Assignment
    Assign,         // =
    PlusEquals,     // +=
    MinusEquals,    // -=
    StarEquals,     // *=
    SlashEquals,    // /=
    PercentEquals,  // %=
    AndEquals,      // &&=
    OrEquals,       // ||=

    // Increment/Decrement
    Increment,      // ++
    Decrement,      // --

    // Delimiters
    LParen,         // (
    RParen,         // )
    LBracket,       // [
    RBracket,       // ]
    LBrace,         // {
    RBrace,         // }
    Semicolon,      // ;
    Comma,          // ,
    Arrow,          // ->
    FatArrow,       // =>
    Colon,          // :
    DoubleColon,    // ::
    Range,          // ..
    Ellipsis,       // ...

    // Special
    Backslash,      // \ (reference)
    At,             // @ (used in interpolation)
    Dollar,         // $ (used in interpolation)
    Question,       // ?

    // End of input
    Eof,
}

impl Token {
    pub fn is_keyword(s: &str) -> Option<Token> {
        match s {
            "my" => Some(Token::My),
            "our" => Some(Token::Our),
            "sub" => Some(Token::Sub),
            "if" => Some(Token::If),
            "elsif" => Some(Token::Elsif),
            "else" => Some(Token::Else),
            "unless" => Some(Token::Unless),
            "while" => Some(Token::While),
            "until" => Some(Token::Until),
            "for" => Some(Token::For),
            "foreach" => Some(Token::Foreach),
            "last" => Some(Token::Last),
            "next" => Some(Token::Next),
            "return" => Some(Token::Return),
            "print" => Some(Token::Print),
            "say" => Some(Token::Say),
            "use" => Some(Token::Use),
            "package" => Some(Token::Package),
            "eq" => Some(Token::StrEq),
            "ne" => Some(Token::StrNe),
            "lt" => Some(Token::StrLt),
            "gt" => Some(Token::StrGt),
            "le" => Some(Token::StrLe),
            "ge" => Some(Token::StrGe),
            "cmp" => Some(Token::StrCmp),
            "and" => Some(Token::AndWord),
            "or" => Some(Token::OrWord),
            "not" => Some(Token::NotWord),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}
