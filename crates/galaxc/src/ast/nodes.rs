// AST node definitions. Each node type corresponds to a grammatical production
// in the GalaxC language. Fields use Box<T> for recursive types and Vec<T>
// for repeated elements.

use crate::diagnostics::Span;

/// Unique identifier assigned to every AST node by the parser.
pub type NodeId = u64;

/// A complete source file (compilation unit).
#[derive(Debug, Clone)]
pub struct Program {
    pub module_decl: Option<ModuleDecl>,
    pub imports: Vec<ImportDecl>,
    pub items: Vec<Item>,
    pub span: Span,
}

// -- Top-level declarations --

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub path: ModulePath,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ModulePath {
    pub segments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: ModulePath,
    pub names: ImportNames,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImportNames {
    /// Import the module itself: `dock core.math`
    Module,
    /// Import specific names: `dock core.math.{sin, cos}`
    Specific(Vec<String>),
}

/// Any top-level item (function, struct, enum, etc.)
#[derive(Debug, Clone)]
pub enum Item {
    Op(OpDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Ability(AbilityDecl),
    ImplBlock(ImplBlock),
    Constant(ConstDecl),
    TaskDecl(TaskDecl),
    TaskBody(TaskBodyDecl),
    ProtectedDecl(ProtectedBlock),
    UnitDecl(UnitDeclNode),
    ExternBlock(ExternBlock),
    StaticAssert(StaticAssertNode),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Op(f) => f.span,
            Item::Struct(s) => s.span,
            Item::Enum(e) => e.span,
            Item::Ability(a) => a.span,
            Item::ImplBlock(i) => i.span,
            Item::Constant(c) => c.span,
            Item::TaskDecl(t) => t.span,
            Item::TaskBody(t) => t.span,
            Item::ProtectedDecl(p) => p.span,
            Item::UnitDecl(u) => u.span,
            Item::ExternBlock(e) => e.span,
            Item::StaticAssert(s) => s.span,
        }
    }
}

// -- Operations (Functions) --

#[derive(Debug, Clone)]
pub struct OpDecl {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub effects: Vec<String>,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Option<Block>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_expr: TypeExpr,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeExpr>,
    pub span: Span,
}

// -- Annotations --

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<AnnotationArg>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AnnotationArg {
    pub key: Option<String>,
    pub value: Expr,
    pub span: Span,
}

// -- Types --

#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Simple named type: Int, Float64, Text
    Named {
        name: String,
        generics: Vec<TypeExpr>,
        span: Span,
    },
    /// Unit-annotated type: Float64<meters>
    UnitType {
        base: String,
        unit: String,
        span: Span,
    },
    /// Array type: [Int; 8]
    Array {
        element: Box<TypeExpr>,
        size: Box<Expr>,
        span: Span,
    },
    /// Slice type: Slice<T>
    Slice {
        element: Box<TypeExpr>,
        span: Span,
    },
    /// Tuple type: (Int, Text)
    Tuple {
        elements: Vec<TypeExpr>,
        span: Span,
    },
    /// Reference type: ref T, mut ref T
    Reference {
        inner: Box<TypeExpr>,
        is_mut: bool,
        span: Span,
    },
    /// Result type shorthand: Result<T, E>
    Result {
        ok_type: Box<TypeExpr>,
        err_type: Box<TypeExpr>,
        span: Span,
    },
    /// Option type shorthand: Option<T>
    Option {
        inner: Box<TypeExpr>,
        span: Span,
    },
    /// Self type
    SelfType { span: Span },
    /// Inferred type (used internally)
    Inferred { span: Span },
    /// Never type
    Never { span: Span },
    /// Qualified path: module::Type
    Path {
        segments: Vec<String>,
        generics: Vec<TypeExpr>,
        span: Span,
    },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. }
            | TypeExpr::UnitType { span, .. }
            | TypeExpr::Array { span, .. }
            | TypeExpr::Slice { span, .. }
            | TypeExpr::Tuple { span, .. }
            | TypeExpr::Reference { span, .. }
            | TypeExpr::Result { span, .. }
            | TypeExpr::Option { span, .. }
            | TypeExpr::SelfType { span }
            | TypeExpr::Inferred { span }
            | TypeExpr::Never { span }
            | TypeExpr::Path { span, .. } => *span,
        }
    }
}

// -- Structs --

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<FieldDecl>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: Span,
}

// -- Enums --

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<VariantDecl>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VariantDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

// -- Abilities (traits) --

#[derive(Debug, Clone)]
pub struct AbilityDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<OpDecl>,
    pub constants: Vec<ConstDecl>,
    pub is_pub: bool,
    pub span: Span,
}

// -- Impl blocks --

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub ability: Option<String>,
    pub target: String,
    pub methods: Vec<OpDecl>,
    pub span: Span,
}

// -- Constants --

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub name: String,
    pub type_expr: Option<TypeExpr>,
    pub value: Expr,
    pub is_pub: bool,
    pub span: Span,
}

// -- Tasks --

#[derive(Debug, Clone)]
pub struct TaskDecl {
    pub name: String,
    pub entries: Vec<TaskEntry>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TaskEntry {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TaskBodyDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

// -- Protected objects --

#[derive(Debug, Clone)]
pub struct ProtectedBlock {
    pub name: String,
    pub fields: Vec<ProtectedField>,
    pub methods: Vec<OpDecl>,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ProtectedField {
    pub name: String,
    pub type_expr: TypeExpr,
    pub default: Expr,
    pub span: Span,
}

// -- Units --

#[derive(Debug, Clone)]
pub struct UnitDeclNode {
    pub name: String,
    pub definition: String,
    pub span: Span,
}

// -- Extern --

#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String,
    pub functions: Vec<OpDecl>,
    pub span: Span,
}

// -- Static assert --

#[derive(Debug, Clone)]
pub struct StaticAssertNode {
    pub condition: Expr,
    pub message: Option<String>,
    pub span: Span,
}

// -- Blocks and statements --

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Var(VarStmt),
    Assign(AssignStmt),
    Expr(ExprStmt),
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Loop(LoopStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Select(SelectStmt),
    Item(Box<Item>),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Let(s) => s.span,
            Stmt::Var(s) => s.span,
            Stmt::Assign(s) => s.span,
            Stmt::Expr(s) => s.span,
            Stmt::If(s) => s.span,
            Stmt::Match(s) => s.span,
            Stmt::For(s) => s.span,
            Stmt::While(s) => s.span,
            Stmt::Loop(s) => s.span,
            Stmt::Return(s) => s.span,
            Stmt::Break(s) => s.span,
            Stmt::Continue(s) => s.span,
            Stmt::Select(s) => s.span,
            Stmt::Item(item) => item.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: String,
    pub type_expr: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VarStmt {
    pub name: String,
    pub type_expr: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub target: Expr,
    pub op: AssignOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_ifs: Vec<(Expr, Block)>,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: MatchArmBody,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum MatchArmBody {
    Block(Block),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub binding: String,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SelectStmt {
    pub arms: Vec<SelectArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum SelectArm {
    Accept {
        entry_name: String,
        params: Vec<Param>,
        body: Block,
        span: Span,
    },
    Delay {
        duration: Expr,
        body: Block,
        span: Span,
    },
    When {
        guard: Expr,
        accept: Box<SelectArm>,
        span: Span,
    },
}

// -- Patterns --

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Matches a specific enum variant: Command.Thrust(force)
    Variant {
        path: Vec<String>,
        fields: Vec<PatternField>,
        span: Span,
    },
    /// Matches a literal value
    Literal {
        value: LiteralValue,
        span: Span,
    },
    /// Binds the matched value to a name
    Binding {
        name: String,
        span: Span,
    },
    /// Wildcard: _
    Wildcard {
        span: Span,
    },
    /// Tuple pattern: (a, b)
    Tuple {
        elements: Vec<Pattern>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct PatternField {
    pub name: Option<String>,
    pub pattern: Pattern,
    pub span: Span,
}

// -- Expressions --

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Identifier(IdentExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
    FieldAccess(FieldAccessExpr),
    Index(IndexExpr),
    StructLiteral(StructLiteralExpr),
    Closure(ClosureExpr),
    If(Box<IfExpr>),
    Match(Box<MatchExpr>),
    Block(Box<Block>),
    Propagate(PropagateExpr),
    ErrorConvert(ErrorConvertExpr),
    Range(RangeExpr),
    ArrayLiteral(ArrayLiteralExpr),
    TupleLiteral(TupleLiteralExpr),
    Path(PathExpr),
    UnsafeBlock(UnsafeBlockExpr),
    SelfExpr(SelfExprNode),
    Pipeline(PipelineExpr),
    Concat(ConcatExpr),
    Cast(Box<CastExpr>),
}

#[derive(Debug, Clone)]
pub struct CastExpr {
    pub expr: Expr,
    pub target_type: TypeExpr,
    pub span: Span,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(e) => e.span,
            Expr::Identifier(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::MethodCall(e) => e.span,
            Expr::FieldAccess(e) => e.span,
            Expr::Index(e) => e.span,
            Expr::StructLiteral(e) => e.span,
            Expr::Closure(e) => e.span,
            Expr::If(e) => e.span,
            Expr::Match(e) => e.span,
            Expr::Block(b) => b.span,
            Expr::Propagate(e) => e.span,
            Expr::ErrorConvert(e) => e.span,
            Expr::Range(e) => e.span,
            Expr::ArrayLiteral(e) => e.span,
            Expr::TupleLiteral(e) => e.span,
            Expr::Path(e) => e.span,
            Expr::UnsafeBlock(e) => e.span,
            Expr::SelfExpr(e) => e.span,
            Expr::Pipeline(e) => e.span,
            Expr::Concat(e) => e.span,
            Expr::Cast(e) => e.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub value: LiteralValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    None,
}

#[derive(Debug, Clone)]
pub struct IdentExpr {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor, ShiftLeft, ShiftRight,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BinaryOp::Add => "+", BinaryOp::Sub => "-",
            BinaryOp::Mul => "*", BinaryOp::Div => "/", BinaryOp::Mod => "%",
            BinaryOp::Eq => "==", BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<", BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=", BinaryOp::GtEq => ">=",
            BinaryOp::And => "and", BinaryOp::Or => "or",
            BinaryOp::BitAnd => "&", BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::ShiftLeft => "<<", BinaryOp::ShiftRight => ">>",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,    // -
    Not,    // not
    BitNot, // ~
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub receiver: Box<Expr>,
    pub method: String,
    pub args: Vec<CallArg>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<Expr>,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub object: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructLiteralExpr {
    pub name: String,
    pub fields: Vec<StructFieldInit>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructFieldInit {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<ClosureParam>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub name: String,
    pub type_expr: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Expr,
    pub then_expr: Expr,
    pub else_expr: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PropagateExpr {
    pub inner: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ErrorConvertExpr {
    pub inner: Box<Expr>,
    pub fallback: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub inclusive: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteralExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TupleLiteralExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PathExpr {
    pub segments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UnsafeBlockExpr {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SelfExprNode {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PipelineExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConcatExpr {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}
