// Intermediate representation module. Provides a simplified, flattened
// representation of the program suitable for code generation.
// The IR is close to C semantics but retains GalaxC type information.

use crate::ast::*;
// Span import removed

/// A complete IR program.
#[derive(Debug, Clone)]
pub struct IrProgram {
    pub module_name: Option<String>,
    pub structs: Vec<IrStruct>,
    pub enums: Vec<IrEnum>,
    pub functions: Vec<IrFunction>,
    pub constants: Vec<IrConst>,
    pub entry_point: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IrStruct {
    pub name: String,
    pub fields: Vec<IrField>,
}

#[derive(Debug, Clone)]
pub struct IrField {
    pub name: String,
    pub c_type: String,
}

#[derive(Debug, Clone)]
pub struct IrEnum {
    pub name: String,
    pub variants: Vec<IrVariant>,
}

#[derive(Debug, Clone)]
pub struct IrVariant {
    pub name: String,
    pub tag: usize,
    pub fields: Vec<IrField>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<IrParam>,
    pub return_type: String,
    pub body: Vec<IrStmt>,
    pub is_entry: bool,
}

#[derive(Debug, Clone)]
pub struct IrParam {
    pub name: String,
    pub c_type: String,
}

#[derive(Debug, Clone)]
pub struct IrConst {
    pub name: String,
    pub c_type: String,
    pub value: String,
}

/// IR statements -- flattened from AST with explicit temporaries.
#[derive(Debug, Clone)]
pub enum IrStmt {
    VarDecl { name: String, c_type: String, init: Option<String> },
    Assign { target: String, value: String },
    Return { value: Option<String> },
    Expr { expr: String },
    If { condition: String, then_body: Vec<IrStmt>, else_body: Vec<IrStmt> },
    While { condition: String, body: Vec<IrStmt> },
    For { init: String, condition: String, update: String, body: Vec<IrStmt> },
    Block { body: Vec<IrStmt> },
    Break,
    Continue,
    Comment { text: String },
    Raw { code: String },
}

/// Lower a type-checked AST into IR.
pub fn lower(program: &Program) -> IrProgram {
    let mut lowerer = IrLowerer::new();
    lowerer.lower_program(program)
}

/// Format an IR program as a human-readable string (for galaxc emit-ir).
pub fn display(program: &IrProgram) -> String {
    let mut out = String::new();

    if let Some(ref name) = program.module_name {
        out.push_str(&format!("module {name}\n\n"));
    }

    for s in &program.structs {
        out.push_str(&format!("struct {} {{\n", s.name));
        for f in &s.fields {
            out.push_str(&format!("  {}: {}\n", f.name, f.c_type));
        }
        out.push_str("}\n\n");
    }

    for e in &program.enums {
        out.push_str(&format!("enum {} {{\n", e.name));
        for v in &e.variants {
            out.push_str(&format!("  {}(tag={}) ", v.name, v.tag));
            if !v.fields.is_empty() {
                out.push_str("{ ");
                for f in &v.fields {
                    out.push_str(&format!("{}: {}, ", f.name, f.c_type));
                }
                out.push_str("}");
            }
            out.push('\n');
        }
        out.push_str("}\n\n");
    }

    for f in &program.functions {
        let params: Vec<String> = f.params.iter()
            .map(|p| format!("{}: {}", p.name, p.c_type))
            .collect();
        out.push_str(&format!("fn {}({}) -> {} {{\n", f.name, params.join(", "), f.return_type));
        for stmt in &f.body {
            display_stmt(&mut out, stmt, 1);
        }
        out.push_str("}\n\n");
    }

    out
}

fn display_stmt(out: &mut String, stmt: &IrStmt, indent: usize) {
    let pad = "  ".repeat(indent);
    match stmt {
        IrStmt::VarDecl { name, c_type, init } => {
            if let Some(init) = init {
                out.push_str(&format!("{pad}{c_type} {name} = {init};\n"));
            } else {
                out.push_str(&format!("{pad}{c_type} {name};\n"));
            }
        }
        IrStmt::Assign { target, value } => {
            out.push_str(&format!("{pad}{target} = {value};\n"));
        }
        IrStmt::Return { value } => {
            if let Some(v) = value {
                out.push_str(&format!("{pad}return {v};\n"));
            } else {
                out.push_str(&format!("{pad}return;\n"));
            }
        }
        IrStmt::Expr { expr } => {
            out.push_str(&format!("{pad}{expr};\n"));
        }
        IrStmt::If { condition, then_body, else_body } => {
            out.push_str(&format!("{pad}if ({condition}) {{\n"));
            for s in then_body {
                display_stmt(out, s, indent + 1);
            }
            if !else_body.is_empty() {
                out.push_str(&format!("{pad}}} else {{\n"));
                for s in else_body {
                    display_stmt(out, s, indent + 1);
                }
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        IrStmt::While { condition, body } => {
            out.push_str(&format!("{pad}while ({condition}) {{\n"));
            for s in body {
                display_stmt(out, s, indent + 1);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        IrStmt::For { init, condition, update, body } => {
            out.push_str(&format!("{pad}for ({init}; {condition}; {update}) {{\n"));
            for s in body {
                display_stmt(out, s, indent + 1);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        IrStmt::Block { body } => {
            out.push_str(&format!("{pad}{{\n"));
            for s in body {
                display_stmt(out, s, indent + 1);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        IrStmt::Break => out.push_str(&format!("{pad}break;\n")),
        IrStmt::Continue => out.push_str(&format!("{pad}continue;\n")),
        IrStmt::Comment { text } => out.push_str(&format!("{pad}// {text}\n")),
        IrStmt::Raw { code } => out.push_str(&format!("{pad}{code}\n")),
    }
}

struct IrLowerer {
    temp_counter: usize,
}

impl IrLowerer {
    fn new() -> Self {
        IrLowerer { temp_counter: 0 }
    }

    fn fresh_temp(&mut self) -> String {
        let name = format!("_t{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    fn lower_program(&mut self, program: &Program) -> IrProgram {
        let module_name = program.module_decl.as_ref().map(|m| {
            m.path.segments.join("_")
        });

        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut functions = Vec::new();
        let mut constants = Vec::new();

        for item in &program.items {
            match item {
                Item::Struct(s) => structs.push(self.lower_struct(s)),
                Item::Enum(e) => enums.push(self.lower_enum(e)),
                Item::Function(f) => functions.push(self.lower_function(f)),
                Item::Constant(c) => constants.push(self.lower_const(c)),
                Item::ImplBlock(block) => {
                    for method in &block.methods {
                        let mut ir_func = self.lower_function(method);
                        ir_func.name = format!("{}_{}", block.target, ir_func.name);
                        functions.push(ir_func);
                    }
                }
                _ => {} // Other items handled separately
            }
        }

        // Determine the entry point
        let entry_point = functions.iter()
            .find(|f| f.name == "launch")
            .map(|_| "launch".to_string());

        IrProgram {
            module_name,
            structs,
            enums,
            functions,
            constants,
            entry_point,
        }
    }

    fn lower_struct(&self, decl: &StructDecl) -> IrStruct {
        IrStruct {
            name: decl.name.clone(),
            fields: decl.fields.iter().map(|f| IrField {
                name: f.name.clone(),
                c_type: self.type_expr_to_c(&f.type_expr),
            }).collect(),
        }
    }

    fn lower_enum(&self, decl: &EnumDecl) -> IrEnum {
        IrEnum {
            name: decl.name.clone(),
            variants: decl.variants.iter().enumerate().map(|(i, v)| IrVariant {
                name: v.name.clone(),
                tag: i,
                fields: v.fields.iter().map(|f| IrField {
                    name: f.name.clone(),
                    c_type: self.type_expr_to_c(&f.type_expr),
                }).collect(),
            }).collect(),
        }
    }

    fn lower_function(&mut self, decl: &FunctionDecl) -> IrFunction {
        let params: Vec<IrParam> = decl.params.iter().map(|p| IrParam {
            name: p.name.clone(),
            c_type: self.type_expr_to_c(&p.type_expr),
        }).collect();

        let return_type = decl.return_type.as_ref()
            .map(|t| self.type_expr_to_c(t))
            .unwrap_or_else(|| "void".to_string());

        let body = if let Some(ref block) = decl.body {
            self.lower_block(block)
        } else {
            Vec::new()
        };

        let is_entry = decl.name == "launch";

        IrFunction {
            name: decl.name.clone(),
            params,
            return_type,
            body,
            is_entry,
        }
    }

    fn lower_const(&self, decl: &ConstDecl) -> IrConst {
        let c_type = decl.type_expr.as_ref()
            .map(|t| self.type_expr_to_c(t))
            .unwrap_or_else(|| "int64_t".to_string());

        let value = self.expr_to_c(&decl.value);

        IrConst {
            name: decl.name.clone(),
            c_type,
            value,
        }
    }

    fn lower_block(&mut self, block: &Block) -> Vec<IrStmt> {
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            stmts.extend(self.lower_stmt(stmt));
        }
        stmts
    }

    fn lower_stmt(&mut self, stmt: &Stmt) -> Vec<IrStmt> {
        match stmt {
            Stmt::Let(s) => {
                let c_type = s.type_expr.as_ref()
                    .map(|t| self.type_expr_to_c(t))
                    .unwrap_or_else(|| self.infer_c_type(&s.value));
                let init = self.expr_to_c(&s.value);
                vec![IrStmt::VarDecl {
                    name: s.name.clone(),
                    c_type,
                    init: Some(init),
                }]
            }

            Stmt::Var(s) => {
                let c_type = s.type_expr.as_ref()
                    .map(|t| self.type_expr_to_c(t))
                    .unwrap_or_else(|| self.infer_c_type(&s.value));
                let init = self.expr_to_c(&s.value);
                vec![IrStmt::VarDecl {
                    name: s.name.clone(),
                    c_type,
                    init: Some(init),
                }]
            }

            Stmt::Assign(s) => {
                let target = self.expr_to_c(&s.target);
                let value = self.expr_to_c(&s.value);
                let op_str = match s.op {
                    AssignOp::Assign => "=",
                    AssignOp::AddAssign => "+=",
                    AssignOp::SubAssign => "-=",
                    AssignOp::MulAssign => "*=",
                    AssignOp::DivAssign => "/=",
                    AssignOp::ModAssign => "%=",
                };
                vec![IrStmt::Assign {
                    target,
                    value: format!("{op_str} {value}"),
                }]
            }

            Stmt::Expr(s) => {
                vec![IrStmt::Expr { expr: self.expr_to_c(&s.expr) }]
            }

            Stmt::If(s) => {
                let condition = self.expr_to_c(&s.condition);
                let then_body = self.lower_block(&s.then_block);
                let else_body = if let Some(ref eb) = s.else_block {
                    self.lower_block(eb)
                } else {
                    Vec::new()
                };
                vec![IrStmt::If { condition, then_body, else_body }]
            }

            Stmt::While(s) => {
                let condition = self.expr_to_c(&s.condition);
                let body = self.lower_block(&s.body);
                vec![IrStmt::While { condition, body }]
            }

            Stmt::Loop(s) => {
                let body = self.lower_block(&s.body);
                vec![IrStmt::While { condition: "1".to_string(), body }]
            }

            Stmt::For(s) => {
                let iterable = self.expr_to_c(&s.iterable);
                let body = self.lower_block(&s.body);
                // Simplified: for-each becomes a while loop with an index
                let idx = self.fresh_temp();
                let mut all = Vec::new();
                all.push(IrStmt::Comment { text: format!("for {} in ...", s.binding) });
                all.push(IrStmt::VarDecl {
                    name: idx.clone(),
                    c_type: "int64_t".to_string(),
                    init: Some("0".to_string()),
                });
                let mut loop_body = Vec::new();
                loop_body.push(IrStmt::VarDecl {
                    name: s.binding.clone(),
                    c_type: "int64_t".to_string(),
                    init: Some(format!("{idx}")),
                });
                loop_body.extend(body);
                loop_body.push(IrStmt::Assign {
                    target: idx.clone(),
                    value: format!("= {idx} + 1"),
                });
                all.push(IrStmt::While {
                    condition: format!("{idx} < gxc_len({iterable})"),
                    body: loop_body,
                });
                all
            }

            Stmt::Return(s) => {
                let value = s.value.as_ref().map(|v| self.expr_to_c(v));
                vec![IrStmt::Return { value }]
            }

            Stmt::Break(_) => vec![IrStmt::Break],
            Stmt::Continue(_) => vec![IrStmt::Continue],

            Stmt::Match(s) => {
                // Lower match to a chain of if/else-if
                let subject = self.expr_to_c(&s.subject);
                let subject_temp = self.fresh_temp();
                let mut stmts = vec![IrStmt::VarDecl {
                    name: subject_temp.clone(),
                    c_type: "int64_t".to_string(),
                    init: Some(subject),
                }];

                let remaining: Vec<&MatchArm> = s.arms.iter().collect();
                if !remaining.is_empty() {
                    let chain = self.lower_match_chain(&subject_temp, &remaining);
                    stmts.extend(chain);
                }
                stmts
            }

            Stmt::Select(_) => {
                vec![IrStmt::Comment {
                    text: "select statement (tasking runtime required)".to_string(),
                }]
            }

            Stmt::Item(_) => Vec::new(),
        }
    }

    fn lower_match_chain(&mut self, subject: &str, arms: &[&MatchArm]) -> Vec<IrStmt> {
        if arms.is_empty() {
            return Vec::new();
        }

        let arm = arms[0];
        let condition = self.pattern_to_condition(subject, &arm.pattern);
        let body = match &arm.body {
            MatchArmBody::Block(block) => self.lower_block(block),
            MatchArmBody::Expr(expr) => vec![IrStmt::Expr { expr: self.expr_to_c(expr) }],
        };

        let else_body = if arms.len() > 1 {
            self.lower_match_chain(subject, &arms[1..])
        } else {
            Vec::new()
        };

        vec![IrStmt::If { condition, then_body: body, else_body }]
    }

    fn pattern_to_condition(&self, subject: &str, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard { .. } => "1".to_string(),
            Pattern::Binding { .. } => "1".to_string(),
            Pattern::Literal { value, .. } => {
                let lit = match value {
                    LiteralValue::Int(n) => format!("{n}"),
                    LiteralValue::Float(n) => format!("{n}"),
                    LiteralValue::Bool(b) => format!("{b}"),
                    LiteralValue::String(s) => format!("gxc_text_eq({subject}, \"{s}\")"),
                    LiteralValue::Char(c) => format!("'{c}'"),
                    LiteralValue::None => "0".to_string(),
                };
                format!("{subject} == {lit}")
            }
            Pattern::Variant { path, .. } => {
                let variant_name = path.last().cloned().unwrap_or_default();
                format!("{subject}.tag == GXC_TAG_{}", variant_name.to_uppercase())
            }
            Pattern::Tuple { .. } => "1".to_string(),
        }
    }

    /// Convert a type expression to its C representation.
    fn type_expr_to_c(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Named { name, .. } => match name.as_str() {
                "Bool" => "bool".to_string(),
                "Int" | "Int64" => "int64_t".to_string(),
                "Int8" => "int8_t".to_string(),
                "Int16" => "int16_t".to_string(),
                "Int32" => "int32_t".to_string(),
                "Uint8" | "Byte" => "uint8_t".to_string(),
                "Uint16" => "uint16_t".to_string(),
                "Uint32" => "uint32_t".to_string(),
                "Uint64" => "uint64_t".to_string(),
                "Float32" => "float".to_string(),
                "Float64" => "double".to_string(),
                "Text" => "GxcText".to_string(),
                "Char" => "uint32_t".to_string(),
                _ => format!("Gxc_{name}"),
            },
            TypeExpr::UnitType { base, .. } => match base.as_str() {
                "Float64" => "double".to_string(),
                "Float32" => "float".to_string(),
                "Int" | "Int64" => "int64_t".to_string(),
                _ => "double".to_string(),
            },
            TypeExpr::SelfType { .. } => "void*".to_string(),
            TypeExpr::Reference { inner, .. } => format!("{}*", self.type_expr_to_c(inner)),
            TypeExpr::Array { element, .. } => format!("GxcArray_{}", self.type_expr_to_c(element)),
            _ => "int64_t".to_string(),
        }
    }

    /// Convert an expression to its C representation string.
    fn expr_to_c(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match &lit.value {
                LiteralValue::Int(n) => format!("{n}LL"),
                LiteralValue::Float(f) => {
                    if f.fract() == 0.0 {
                        format!("{f:.1}")
                    } else {
                        format!("{f}")
                    }
                }
                LiteralValue::String(s) => format!("gxc_text_from(\"{s}\")"),
                LiteralValue::Char(c) => format!("'{c}'"),
                LiteralValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
                LiteralValue::None => "GXC_NONE".to_string(),
            },

            Expr::Identifier(id) => id.name.clone(),

            Expr::Binary(bin) => {
                let left = self.expr_to_c(&bin.left);
                let right = self.expr_to_c(&bin.right);
                let op = match bin.op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::NotEq => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::ShiftLeft => "<<",
                    BinaryOp::ShiftRight => ">>",
                };
                format!("({left} {op} {right})")
            }

            Expr::Unary(un) => {
                let operand = self.expr_to_c(&un.operand);
                match un.op {
                    UnaryOp::Neg => format!("(-{operand})"),
                    UnaryOp::Not => format!("(!{operand})"),
                    UnaryOp::BitNot => format!("(~{operand})"),
                }
            }

            Expr::Call(call) => {
                let callee = self.expr_to_c(&call.callee);
                let args: Vec<String> = call.args.iter()
                    .map(|a| self.expr_to_c(&a.value))
                    .collect();
                format!("{callee}({})", args.join(", "))
            }

            Expr::MethodCall(mc) => {
                let receiver = self.expr_to_c(&mc.receiver);
                let args_strs: Vec<String> = mc.args.iter()
                    .map(|a| self.expr_to_c(&a.value))
                    .collect();
                
                if receiver == "console" {
                    format!("GxcConsole_{}({})", mc.method, args_strs.join(", "))
                } else {
                    // Standard method call: Type_method(receiver, args)
                    // For now, we still don't have full type info here, so we'll use a placeholder
                    // or just keep the dot until we have a typed-IR pass.
                    format!("{receiver}.{}({})", mc.method, args_strs.join(", "))
                }
            }

            Expr::FieldAccess(fa) => {
                let obj = self.expr_to_c(&fa.object);
                format!("{obj}.{}", fa.field)
            }

            Expr::Index(idx) => {
                let obj = self.expr_to_c(&idx.object);
                let index = self.expr_to_c(&idx.index);
                format!("gxc_bounds_check({obj}, {index})")
            }

            Expr::StructLiteral(sl) => {
                let fields: Vec<String> = sl.fields.iter()
                    .map(|f| format!(".{} = {}", f.name, self.expr_to_c(&f.value)))
                    .collect();
                format!("(Gxc_{}){{ {} }}", sl.name, fields.join(", "))
            }

            Expr::Concat(c) => {
                let left = self.expr_to_c(&c.left);
                let right = self.expr_to_c(&c.right);
                format!("gxc_text_concat({left}, {right})")
            }

            Expr::Path(p) => p.segments.join("_"),

            Expr::SelfExpr(_) => "self".to_string(),

            Expr::Propagate(p) => {
                let inner = self.expr_to_c(&p.inner);
                format!("GXC_PROPAGATE({inner})")
            }

            Expr::Range(r) => {
                let start = self.expr_to_c(&r.start);
                let end = self.expr_to_c(&r.end);
                format!("gxc_range({start}, {end})")
            }

            _ => "/* unhandled expr */".to_string(),
        }
    }

    fn infer_c_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit) => match &lit.value {
                LiteralValue::Int(_) => "int64_t".to_string(),
                LiteralValue::Float(_) => "double".to_string(),
                LiteralValue::String(_) => "GxcText".to_string(),
                LiteralValue::Char(_) => "uint32_t".to_string(),
                LiteralValue::Bool(_) => "bool".to_string(),
                LiteralValue::None => "GxcOption".to_string(),
            },
            Expr::Binary(bin) => match bin.op {
                BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Lt | BinaryOp::Gt
                | BinaryOp::LtEq | BinaryOp::GtEq | BinaryOp::And | BinaryOp::Or => {
                    "bool".to_string()
                }
                _ => self.infer_c_type(&bin.left),
            },
            Expr::Concat(_) => "GxcText".to_string(),
            _ => "int64_t".to_string(),
        }
    }
}
