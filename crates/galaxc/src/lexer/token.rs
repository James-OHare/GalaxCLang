// Token definitions for GalaxC.
// Every lexeme produced by the scanner is tagged with a TokenKind
// and carries its source span for diagnostic reporting.

use crate::diagnostics::Span;

/// A single token with its kind, span, and optional literal value.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// The raw source text of this token, stored for identifiers and literals.
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, lexeme: impl Into<String>) -> Self {
        Token {
            kind,
            span,
            lexeme: lexeme.into(),
        }
    }

    pub fn synthetic(kind: TokenKind) -> Self {
        Token {
            kind,
            span: Span::point(0),
            lexeme: String::new(),
        }
    }
}

/// All distinct token types in GalaxC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // -- Literals --
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,

    // -- Identifier --
    Identifier,

    // -- Keywords: declarations --
    Op,          // op
    Let,         // let
    Var,         // var
    Const,       // const
    Struct,      // struct
    Enum,        // enum
    Ability,     // ability
    Impl,        // impl
    Task,        // task
    Protected,   // protected
    Orbit,       // orbit
    Dock,        // dock
    Body,        // body
    Pub,         // pub
    Extern,      // extern

    // -- Keywords: control flow --
    If,          // if
    Else,        // else
    Match,       // match
    For,         // for
    While,       // while
    Loop,        // loop
    Break,       // break
    Continue,    // continue
    Return,      // return
    Select,      // select
    Accept,      // accept
    Or,          // or (in select and logical)
    Delay,       // delay (in select)
    When,        // when (guard)

    // -- Keywords: values --
    True,        // true
    False,       // false
    None_,       // none
    Ok_,         // ok
    Err_,        // err
    Some_,       // some
    SelfLower,   // self
    SelfUpper,   // Self

    // -- Keywords: safety and modifiers --
    Mut,         // mut
    Ref,         // ref
    Own,         // own
    Safe,        // safe
    Unsafe,      // unsafe
    As,          // as
    In,          // in
    End,         // end
    Async,       // async
    Unit,        // unit

    // -- Keywords: contracts --
    StaticAssert, // static_assert

    // -- Operators --
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    PlusPlus,    // ++  (string concat)
    Eq,          // ==
    NotEq,       // !=
    Lt,          // <
    Gt,          // >
    LtEq,       // <=
    GtEq,       // >=
    And,         // and
    Not,         // not
    Ampersand,   // &
    Pipe,        // |
    Caret,       // ^
    Tilde,       // ~
    ShiftLeft,   // <<
    ShiftRight,  // >>
    Pipeline,    // >> (contextually same as ShiftRight, resolved in parser)

    // -- Assignment --
    Assign,      // =
    PlusAssign,  // +=
    MinusAssign, // -=
    StarAssign,  // *=
    SlashAssign, // /=
    PercentAssign, // %=

    // -- Delimiters --
    LParen,      // (
    RParen,      // )
    LBracket,    // [
    RBracket,    // ]
    LBrace,      // {
    RBrace,      // }

    // -- Punctuation --
    Arrow,       // ->
    FatArrow,    // =>
    Colon,       // :
    ColonColon,  // ::
    Comma,       // ,
    Dot,         // .
    DotDot,      // ..
    Question,    // ?
    BangBang,    // !!
    At,          // @
    Semicolon,   // ;

    // -- Special --
    Newline,     // logical newline (used as statement separator)
    DocComment,  // --- ...
    Eof,         // end of file
}

impl TokenKind {
    /// Look up a keyword from its text, or return None if it is a plain identifier.
    pub fn keyword(text: &str) -> Option<TokenKind> {
        match text {
            "op" => Some(TokenKind::Op),
            "let" => Some(TokenKind::Let),
            "var" => Some(TokenKind::Var),
            "const" => Some(TokenKind::Const),
            "struct" => Some(TokenKind::Struct),
            "enum" => Some(TokenKind::Enum),
            "ability" => Some(TokenKind::Ability),
            "impl" => Some(TokenKind::Impl),
            "task" => Some(TokenKind::Task),
            "protected" => Some(TokenKind::Protected),
            "orbit" => Some(TokenKind::Orbit),
            "dock" => Some(TokenKind::Dock),
            "body" => Some(TokenKind::Body),
            "pub" => Some(TokenKind::Pub),
            "extern" => Some(TokenKind::Extern),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "match" => Some(TokenKind::Match),
            "for" => Some(TokenKind::For),
            "while" => Some(TokenKind::While),
            "loop" => Some(TokenKind::Loop),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "return" => Some(TokenKind::Return),
            "select" => Some(TokenKind::Select),
            "accept" => Some(TokenKind::Accept),
            "or" => Some(TokenKind::Or),
            "delay" => Some(TokenKind::Delay),
            "when" => Some(TokenKind::When),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "none" => Some(TokenKind::None_),
            "ok" => Some(TokenKind::Ok_),
            "err" => Some(TokenKind::Err_),
            "some" => Some(TokenKind::Some_),
            "self" => Some(TokenKind::SelfLower),
            "Self" => Some(TokenKind::SelfUpper),
            "mut" => Some(TokenKind::Mut),
            "ref" => Some(TokenKind::Ref),
            "own" => Some(TokenKind::Own),
            "safe" => Some(TokenKind::Safe),
            "unsafe" => Some(TokenKind::Unsafe),
            "as" => Some(TokenKind::As),
            "in" => Some(TokenKind::In),
            "end" => Some(TokenKind::End),
            "and" => Some(TokenKind::And),
            "not" => Some(TokenKind::Not),
            "async" => Some(TokenKind::Async),
            "unit" => Some(TokenKind::Unit),
            "static_assert" => Some(TokenKind::StaticAssert),
            _ => None,
        }
    }

    /// Whether this token kind can start a statement. Used for error recovery:
    /// when the parser encounters an error, it skips tokens until it finds one
    /// of these synchronization points.
    pub fn is_statement_start(&self) -> bool {
        matches!(
            self,
            TokenKind::Op
                | TokenKind::Let
                | TokenKind::Var
                | TokenKind::Const
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Ability
                | TokenKind::Impl
                | TokenKind::Task
                | TokenKind::Protected
                | TokenKind::If
                | TokenKind::Match
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Select
                | TokenKind::At
                | TokenKind::Pub
        )
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenKind::IntLiteral => "integer",
            TokenKind::FloatLiteral => "float",
            TokenKind::StringLiteral => "string",
            TokenKind::CharLiteral => "character",
            TokenKind::Identifier => "identifier",
            TokenKind::Op => "'op'",
            TokenKind::Let => "'let'",
            TokenKind::Var => "'var'",
            TokenKind::Const => "'const'",
            TokenKind::Struct => "'struct'",
            TokenKind::Enum => "'enum'",
            TokenKind::Ability => "'ability'",
            TokenKind::Impl => "'impl'",
            TokenKind::Task => "'task'",
            TokenKind::Protected => "'protected'",
            TokenKind::Orbit => "'orbit'",
            TokenKind::Dock => "'dock'",
            TokenKind::Body => "'body'",
            TokenKind::Pub => "'pub'",
            TokenKind::Extern => "'extern'",
            TokenKind::If => "'if'",
            TokenKind::Else => "'else'",
            TokenKind::Match => "'match'",
            TokenKind::For => "'for'",
            TokenKind::While => "'while'",
            TokenKind::Loop => "'loop'",
            TokenKind::Break => "'break'",
            TokenKind::Continue => "'continue'",
            TokenKind::Return => "'return'",
            TokenKind::Select => "'select'",
            TokenKind::Accept => "'accept'",
            TokenKind::Or => "'or'",
            TokenKind::Delay => "'delay'",
            TokenKind::When => "'when'",
            TokenKind::True => "'true'",
            TokenKind::False => "'false'",
            TokenKind::None_ => "'none'",
            TokenKind::Ok_ => "'ok'",
            TokenKind::Err_ => "'err'",
            TokenKind::Some_ => "'some'",
            TokenKind::SelfLower => "'self'",
            TokenKind::SelfUpper => "'Self'",
            TokenKind::Mut => "'mut'",
            TokenKind::Ref => "'ref'",
            TokenKind::Own => "'own'",
            TokenKind::Safe => "'safe'",
            TokenKind::Unsafe => "'unsafe'",
            TokenKind::As => "'as'",
            TokenKind::In => "'in'",
            TokenKind::End => "'end'",
            TokenKind::And => "'and'",
            TokenKind::Not => "'not'",
            TokenKind::Async => "'async'",
            TokenKind::Unit => "'unit'",
            TokenKind::StaticAssert => "'static_assert'",
            TokenKind::Plus => "'+'",
            TokenKind::Minus => "'-'",
            TokenKind::Star => "'*'",
            TokenKind::Slash => "'/'",
            TokenKind::Percent => "'%'",
            TokenKind::PlusPlus => "'++'",
            TokenKind::Eq => "'=='",
            TokenKind::NotEq => "'!='",
            TokenKind::Lt => "'<'",
            TokenKind::Gt => "'>'",
            TokenKind::LtEq => "'<='",
            TokenKind::GtEq => "'>='",
            TokenKind::Ampersand => "'&'",
            TokenKind::Pipe => "'|'",
            TokenKind::Caret => "'^'",
            TokenKind::Tilde => "'~'",
            TokenKind::ShiftLeft => "'<<'",
            TokenKind::ShiftRight => "'>>'",
            TokenKind::Pipeline => "'>>'",
            TokenKind::Assign => "'='",
            TokenKind::PlusAssign => "'+='",
            TokenKind::MinusAssign => "'-='",
            TokenKind::StarAssign => "'*='",
            TokenKind::SlashAssign => "'/='",
            TokenKind::PercentAssign => "'%='",
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::LBracket => "'['",
            TokenKind::RBracket => "']'",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::Arrow => "'->'",
            TokenKind::FatArrow => "'=>'",
            TokenKind::Colon => "':'",
            TokenKind::ColonColon => "'::'",
            TokenKind::Comma => "','",
            TokenKind::Dot => "'.'",
            TokenKind::DotDot => "'..'",
            TokenKind::Question => "'?'",
            TokenKind::BangBang => "'!!'",
            TokenKind::At => "'@'",
            TokenKind::Semicolon => "';'",
            TokenKind::Newline => "newline",
            TokenKind::DocComment => "doc comment",
            TokenKind::Eof => "end of file",
        };
        write!(f, "{s}")
    }
}
