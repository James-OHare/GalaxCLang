// Core type-checking pass. Walks the AST, resolves type expressions to 
// semantic types, checks assignments, verifies function calls, and
// annotates every expression with its resolved type.

use super::env::TypeEnv;
use crate::ast::*;
use crate::types::*;
use crate::diagnostics::Diagnostic;

/// Run the type checker on a parsed program. Returns the validated AST
/// (unchanged for now -- full typed-AST annotation is a future pass)
/// or a list of type errors.
pub fn check(program: &Program, filename: &str) -> Result<Program, Vec<Diagnostic>> {
    let mut checker = TypeChecker::new();
    checker.check_program(program);

    if checker.errors.is_empty() {
        Ok(program.clone())
    } else {
        // Attach filename to all diagnostics that don't have one
        for diag in &mut checker.errors {
            if diag.filename.is_none() {
                diag.filename = Some(filename.to_string());
            }
        }
        Err(checker.errors)
    }
}

struct TypeChecker {
    env: TypeEnv,
    errors: Vec<Diagnostic>,
    current_return_type: Option<Type>,
    in_loop: bool,
}

impl TypeChecker {
    fn new() -> Self {
        TypeChecker {
            env: TypeEnv::new(),
            errors: Vec::new(),
            current_return_type: None,
            in_loop: false,
        }
    }

    fn check_program(&mut self, program: &Program) {
        // First pass: register all top-level type declarations so they
        // can reference each other regardless of source order.
        for item in &program.items {
            self.register_item(item);
        }

        // Second pass: check function bodies and verify type consistency.
        for item in &program.items {
            self.check_item(item);
        }
    }

    // -- Registration pass: build the type environment --

    fn register_item(&mut self, item: &Item) {
        match item {
            Item::Op(f) => self.register_op(f),
            Item::Struct(s) => self.register_struct(s),
            Item::Enum(e) => self.register_enum(e),
            Item::Ability(a) => self.register_ability(a),
            Item::Constant(c) => self.register_const(c),
            Item::TaskDecl(t) => self.register_task(t),
            Item::ImplBlock(_) => {} // handled in check pass
            Item::TaskBody(_) => {}
            Item::ProtectedDecl(_) => {} // handled in check pass
            Item::UnitDecl(_) => {}
            Item::ExternBlock(e) => {
                for f in &e.functions {
                    self.register_op(f);
                }
            }
            Item::StaticAssert(_) => {}
        }
    }

    fn register_op(&mut self, func: &OpDecl) {
        let params: Vec<ParamInfo> = func
            .params
            .iter()
            .map(|p| ParamInfo {
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
                is_mut: p.is_mut,
            })
            .collect();

        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(Type::Unit);

        let effects: Vec<String> = func
            .annotations
            .iter()
            .filter(|a| a.name == "effect")
            .flat_map(|a| {
                a.args.iter().filter_map(|arg| {
                    if let Expr::Identifier(id) = &arg.value {
                        Some(id.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        let generic_params: Vec<String> = func.generics.iter().map(|g| g.name.clone()).collect();

        self.env.register_function(FunctionInfo {
            name: func.name.clone(),
            params,
            return_type,
            generic_params,
            effects,
            is_pub: func.is_pub,
        });
    }

    fn register_struct(&mut self, decl: &StructDecl) {
        let id = self.env.fresh_type_id();
        let fields: Vec<FieldInfo> = decl
            .fields
            .iter()
            .map(|f| FieldInfo {
                name: f.name.clone(),
                ty: self.resolve_type_expr(&f.type_expr),
            })
            .collect();

        let generic_params: Vec<String> = decl.generics.iter().map(|g| g.name.clone()).collect();

        self.env.register_struct(StructInfo {
            id,
            name: decl.name.clone(),
            fields,
            generic_params,
            methods: Vec::new(),
        });
    }

    fn register_enum(&mut self, decl: &EnumDecl) {
        let id = self.env.fresh_type_id();
        let variants: Vec<VariantInfo> = decl
            .variants
            .iter()
            .map(|v| VariantInfo {
                name: v.name.clone(),
                fields: v
                    .fields
                    .iter()
                    .map(|f| FieldInfo {
                        name: f.name.clone(),
                        ty: self.resolve_type_expr(&f.type_expr),
                    })
                    .collect(),
            })
            .collect();

        let generic_params: Vec<String> = decl.generics.iter().map(|g| g.name.clone()).collect();

        self.env.register_enum(EnumInfo {
            id,
            name: decl.name.clone(),
            variants,
            generic_params,
        });
    }

    fn register_ability(&mut self, decl: &AbilityDecl) {
        let id = self.env.fresh_type_id();
        let methods: Vec<FunctionInfo> = decl
            .methods
            .iter()
            .map(|m| {
                let params = m
                    .params
                    .iter()
                    .map(|p| ParamInfo {
                        name: p.name.clone(),
                        ty: self.resolve_type_expr(&p.type_expr),
                        is_mut: p.is_mut,
                    })
                    .collect();

                let ret = m
                    .return_type
                    .as_ref()
                    .map(|t| self.resolve_type_expr(t))
                    .unwrap_or(Type::Unit);

                FunctionInfo {
                    name: m.name.clone(),
                    params,
                    return_type: ret,
                    generic_params: Vec::new(),
                    effects: Vec::new(),
                    is_pub: true,
                }
            })
            .collect();

        let generic_params: Vec<String> = decl.generics.iter().map(|g| g.name.clone()).collect();

        self.env.register_ability(AbilityInfo {
            id,
            name: decl.name.clone(),
            methods,
            generic_params,
        });
    }

    fn register_const(&mut self, decl: &ConstDecl) {
        let ty = decl
            .type_expr
            .as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or_else(|| self.infer_expr_type(&decl.value));

        self.env.bind_var(&decl.name, ty, false);
    }

    fn register_task(&mut self, decl: &TaskDecl) {
        let entries: Vec<FunctionInfo> = decl
            .entries
            .iter()
            .map(|e| {
                let params = e
                    .params
                    .iter()
                    .map(|p| ParamInfo {
                        name: p.name.clone(),
                        ty: self.resolve_type_expr(&p.type_expr),
                        is_mut: p.is_mut,
                    })
                    .collect();

                let ret = e
                    .return_type
                    .as_ref()
                    .map(|t| self.resolve_type_expr(t))
                    .unwrap_or(Type::Unit);

                FunctionInfo {
                    name: e.name.clone(),
                    params,
                    return_type: ret,
                    generic_params: Vec::new(),
                    effects: Vec::new(),
                    is_pub: true,
                }
            })
            .collect();

        self.env.register_task(TaskInfo {
            name: decl.name.clone(),
            entries,
        });
    }

    // -- Checking pass: verify function bodies and type consistency --

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Op(f) => self.check_op(f),
            Item::ImplBlock(block) => {
                for method in &block.methods {
                    self.register_op(method);
                    self.check_op(method);
                }
            }
            Item::TaskBody(tb) => self.check_task_body(tb),
            Item::Constant(c) => self.check_const(c),
            Item::StaticAssert(sa) => self.check_static_assert(sa),
            _ => {} // Struct/enum/ability declarations are checked during registration
        }
    }

    fn check_op(&mut self, func: &OpDecl) {
        let return_type = func
            .return_type
            .as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(Type::Unit);

        self.current_return_type = Some(return_type);
        self.env.push_scope();

        // Bind parameters
        for param in &func.params {
            let ty = self.resolve_type_expr(&param.type_expr);
            self.env.bind_var(&param.name, ty, param.is_mut);
        }

        // Check body
        if let Some(ref body) = func.body {
            self.check_block(body);
        }

        self.env.pop_scope();
        self.current_return_type = None;
    }

    fn check_task_body(&mut self, tb: &TaskBodyDecl) {
        self.env.push_scope();
        for param in &tb.params {
            let ty = self.resolve_type_expr(&param.type_expr);
            self.env.bind_var(&param.name, ty, param.is_mut);
        }
        self.check_block(&tb.body);
        self.env.pop_scope();
    }

    fn check_const(&mut self, decl: &ConstDecl) {
        let declared = decl.type_expr.as_ref().map(|t| self.resolve_type_expr(t));
        let inferred = self.infer_expr_type(&decl.value);

        if let Some(ref declared_ty) = declared {
            if !self.types_compatible(declared_ty, &inferred) {
                self.errors.push(
                    Diagnostic::error(format!(
                        "constant '{}' declared as {} but initialized with {}",
                        decl.name,
                        declared_ty.display_name(),
                        inferred.display_name()
                    ))
                    .with_span(decl.span),
                );
            }
        }
    }

    fn check_static_assert(&mut self, sa: &StaticAssertNode) {
        let ty = self.infer_expr_type(&sa.condition);
        if ty != Type::Bool && !ty.is_error() {
            self.errors.push(
                Diagnostic::error("static_assert condition must be a Bool expression")
                    .with_span(sa.span),
            );
        }
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(s) => {
                let inferred = self.infer_expr_type(&s.value);
                let ty = if let Some(ref declared) = s.type_expr {
                    let declared_type = self.resolve_type_expr(declared);
                    if !self.types_compatible(&declared_type, &inferred) && !inferred.is_error() {
                        self.errors.push(
                            Diagnostic::error(format!(
                                "cannot assign {} to variable '{}' of type {}",
                                inferred.display_name(),
                                s.name,
                                declared_type.display_name()
                            ))
                            .with_span(s.span),
                        );
                    }
                    declared_type
                } else {
                    inferred
                };
                self.env.bind_var(&s.name, ty, false);
            }

            Stmt::Var(s) => {
                let inferred = self.infer_expr_type(&s.value);
                let ty = if let Some(ref declared) = s.type_expr {
                    let declared_type = self.resolve_type_expr(declared);
                    if !self.types_compatible(&declared_type, &inferred) && !inferred.is_error() {
                        self.errors.push(
                            Diagnostic::error(format!(
                                "cannot assign {} to variable '{}' of type {}",
                                inferred.display_name(),
                                s.name,
                                declared_type.display_name()
                            ))
                            .with_span(s.span),
                        );
                    }
                    declared_type
                } else {
                    inferred
                };
                self.env.bind_var(&s.name, ty, true);
            }

            Stmt::Assign(s) => {
                if !self.check_lvalue_mutability(&s.target) {
                    self.errors.push(
                        Diagnostic::error("cannot assign to immutable location")
                            .with_span(s.target.span())
                            .with_help("make the base variable mutable with 'var' or 'mut'"),
                    );
                }
                let _value_ty = self.infer_expr_type(&s.value);
            }

            Stmt::Expr(s) => {
                let ty = self.infer_expr_type(&s.expr);
                if ty.is_must_use() {
                    self.errors.push(
                        Diagnostic::error(format!(
                            "unused result of type {} that must be handled",
                            ty.display_name()
                        ))
                        .with_span(s.expr.span())
                        .with_help("handle the value with 'match', '?', or assign it to a variable"),
                    );
                }
            }

            Stmt::If(s) => {
                let cond_ty = self.infer_expr_type(&s.condition);
                if cond_ty != Type::Bool && !cond_ty.is_error() {
                    self.errors.push(
                        Diagnostic::error(format!(
                            "if condition must be Bool, found {}",
                            cond_ty.display_name()
                        ))
                        .with_span(s.condition.span()),
                    );
                }
                self.env.push_scope();
                self.check_block(&s.then_block);
                self.env.pop_scope();

                for (cond, block) in &s.else_ifs {
                    let cond_ty = self.infer_expr_type(cond);
                    if cond_ty != Type::Bool && !cond_ty.is_error() {
                        self.errors.push(
                            Diagnostic::error("else-if condition must be Bool")
                                .with_span(cond.span()),
                        );
                    }
                    self.env.push_scope();
                    self.check_block(block);
                    self.env.pop_scope();
                }

                if let Some(ref block) = s.else_block {
                    self.env.push_scope();
                    self.check_block(block);
                    self.env.pop_scope();
                }
            }

            Stmt::Match(s) => {
                let _subject_ty = self.infer_expr_type(&s.subject);
                for arm in &s.arms {
                    self.env.push_scope();
                    self.bind_pattern(&arm.pattern);
                    match &arm.body {
                        MatchArmBody::Block(block) => self.check_block(block),
                        MatchArmBody::Expr(expr) => { self.infer_expr_type(expr); }
                    }
                    self.env.pop_scope();
                }
            }

            Stmt::For(s) => {
                let iter_ty = self.infer_expr_type(&s.iterable);
                // Infer element type from the iterable
                let elem_ty = match &iter_ty {
                    Type::Array { element, .. } | Type::Slice { element } => *element.clone(),
                    _ => Type::Error,
                };
                self.env.push_scope();
                self.env.bind_var(&s.binding, elem_ty, false);
                self.in_loop = true;
                self.check_block(&s.body);
                self.in_loop = false;
                self.env.pop_scope();
            }

            Stmt::While(s) => {
                let cond_ty = self.infer_expr_type(&s.condition);
                if cond_ty != Type::Bool && !cond_ty.is_error() {
                    self.errors.push(
                        Diagnostic::error("while condition must be Bool")
                            .with_span(s.condition.span()),
                    );
                }
                self.env.push_scope();
                self.in_loop = true;
                self.check_block(&s.body);
                self.in_loop = false;
                self.env.pop_scope();
            }

            Stmt::Loop(s) => {
                self.env.push_scope();
                self.in_loop = true;
                self.check_block(&s.body);
                self.in_loop = false;
                self.env.pop_scope();
            }

            Stmt::Return(s) => {
                if let Some(ref value) = s.value {
                    let _value_ty = self.infer_expr_type(value);
                }
            }

            Stmt::Break(s) => {
                if !self.in_loop {
                    self.errors.push(
                        Diagnostic::error("'break' used outside of a loop").with_span(s.span),
                    );
                }
            }

            Stmt::Continue(s) => {
                if !self.in_loop {
                    self.errors.push(
                        Diagnostic::error("'continue' used outside of a loop").with_span(s.span),
                    );
                }
            }

            Stmt::Select(s) => {
                for arm in &s.arms {
                    self.check_select_arm(arm);
                }
            }

            Stmt::Item(item) => {
                self.register_item(item);
                self.check_item(item);
            }
        }
    }

    fn check_select_arm(&mut self, arm: &SelectArm) {
        match arm {
            SelectArm::Accept { body, params, .. } => {
                self.env.push_scope();
                for p in params {
                    let ty = self.resolve_type_expr(&p.type_expr);
                    self.env.bind_var(&p.name, ty, p.is_mut);
                }
                self.check_block(body);
                self.env.pop_scope();
            }
            SelectArm::Delay { duration, body, .. } => {
                self.infer_expr_type(duration);
                self.env.push_scope();
                self.check_block(body);
                self.env.pop_scope();
            }
            SelectArm::When { guard, accept, .. } => {
                let guard_ty = self.infer_expr_type(guard);
                if guard_ty != Type::Bool && !guard_ty.is_error() {
                    self.errors.push(
                        Diagnostic::error("select guard must be Bool")
                            .with_span(guard.span()),
                    );
                }
                self.check_select_arm(accept);
            }
        }
    }

    // -- Type inference for expressions --

    fn infer_expr_type(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit) => match &lit.value {
                LiteralValue::Int(_) => Type::Int,
                LiteralValue::Float(_) => Type::Float64,
                LiteralValue::String(_) => Type::Text,
                LiteralValue::Char(_) => Type::Char,
                LiteralValue::Bool(_) => Type::Bool,
                LiteralValue::None => Type::Option { inner: Box::new(Type::Inferred { id: self.env.fresh_infer_id() }) },
            },

            Expr::Identifier(id) => {
                if let Some(binding) = self.env.lookup_var(&id.name) {
                    binding.ty.clone()
                } else if self.env.get_function(&id.name).is_some() {
                    // Function reference (for passing as value)
                    Type::Function {
                        params: Vec::new(),
                        ret: Box::new(Type::Inferred { id: self.env.fresh_infer_id() }),
                    }
                } else {
                    self.errors.push(
                        Diagnostic::error(format!("undefined variable '{}'", id.name))
                            .with_span(id.span),
                    );
                    Type::Error
                }
            }

            Expr::Binary(bin) => {
                let left_ty = self.infer_expr_type(&bin.left);
                let right_ty = self.infer_expr_type(&bin.right);

                match bin.op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if left_ty.is_numeric() && right_ty.is_numeric() {
                            left_ty // simplified: real impl would unify
                        } else if left_ty.is_error() || right_ty.is_error() {
                            Type::Error
                        } else {
                            self.errors.push(
                                Diagnostic::error(format!(
                                    "arithmetic operator '{}' requires numeric operands, found {} and {}",
                                    bin.op, left_ty.display_name(), right_ty.display_name()
                                ))
                                .with_span(bin.span),
                            );
                            Type::Error
                        }
                    }
                    BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::LtEq | BinaryOp::GtEq => Type::Bool,
                    BinaryOp::And | BinaryOp::Or => {
                        if left_ty != Type::Bool && !left_ty.is_error() {
                            self.errors.push(
                                Diagnostic::error(format!(
                                    "logical '{}' requires Bool operands, found {}",
                                    bin.op, left_ty.display_name()
                                ))
                                .with_span(bin.left.span()),
                            );
                        }
                        Type::Bool
                    }
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor
                    | BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                        if left_ty.is_integer() {
                            left_ty
                        } else {
                            Type::Error
                        }
                    }
                }
            }

            Expr::Unary(un) => {
                let operand_ty = self.infer_expr_type(&un.operand);
                match un.op {
                    UnaryOp::Neg => operand_ty,
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::BitNot => operand_ty,
                }
            }

            Expr::Call(call) => {
                // Infer from callee
                if let Expr::Identifier(ref id) = *call.callee {
                    // Built-in constructors
                    match id.name.as_str() {
                        "ok" | "some" => {
                            if let Some(arg) = call.args.first() {
                                let inner = self.infer_expr_type(&arg.value);
                                if id.name == "ok" {
                                    Type::Result {
                                        ok: Box::new(inner),
                                        err: Box::new(Type::Error),
                                    }
                                } else {
                                    Type::Option { inner: Box::new(inner) }
                                }
                            } else {
                                Type::Error
                            }
                        }
                        "err" => {
                            if let Some(arg) = call.args.first() {
                                let inner = self.infer_expr_type(&arg.value);
                                Type::Result {
                                    ok: Box::new(Type::Inferred { id: self.env.fresh_infer_id() }),
                                    err: Box::new(inner),
                                }
                            } else {
                                Type::Error
                            }
                        }
                        _ => {
                            if let Some(func_info) = self.env.get_function(&id.name).cloned() {
                                // Check argument count
                                let expected = func_info.params.len();
                                let got = call.args.len();
                                if expected != got {
                                    self.errors.push(
                                        Diagnostic::error(format!(
                                            "function '{}' expects {expected} argument(s), got {got}",
                                            id.name
                                        ))
                                        .with_span(call.span),
                                    );
                                }
                                // Check argument types
                                for (arg, _param) in call.args.iter().zip(&func_info.params) {
                                    let _arg_ty = self.infer_expr_type(&arg.value);
                                }
                                func_info.return_type.clone()
                            } else {
                                // Might be a struct constructor or unknown function
                                for arg in &call.args {
                                    self.infer_expr_type(&arg.value);
                                }
                                Type::Inferred { id: self.env.fresh_infer_id() }
                            }
                        }
                    }
                } else {
                    let _callee_ty = self.infer_expr_type(&call.callee);
                    for arg in &call.args {
                        self.infer_expr_type(&arg.value);
                    }
                    Type::Inferred { id: self.env.fresh_infer_id() }
                }
            }

            Expr::MethodCall(mc) => {
                let recv_ty = self.infer_expr_type(&mc.receiver);
                if let Type::Struct { id, .. } = &recv_ty {
                    if let Some(info) = self.env.get_struct(*id) {
                        if let Some(method) = info.methods.iter().find(|m| m.name == mc.method) {
                            // Check argument count and types (simplified)
                            return method.return_type.clone();
                        }
                    }
                }
                
                // Fallback for inferred or unknown
                for arg in &mc.args {
                    self.infer_expr_type(&arg.value);
                }
                Type::Inferred { id: self.env.fresh_infer_id() }
            }

            Expr::FieldAccess(fa) => {
                let obj_ty = self.infer_expr_type(&fa.object);
                if let Type::Struct { id, .. } = &obj_ty {
                    if let Some(info) = self.env.get_struct(*id) {
                        if let Some(field) = info.fields.iter().find(|f| f.name == fa.field) {
                            return field.ty.clone();
                        }
                    }
                }
                Type::Inferred { id: self.env.fresh_infer_id() }
            }

            Expr::Index(idx) => {
                let obj_ty = self.infer_expr_type(&idx.object);
                let _index_ty = self.infer_expr_type(&idx.index);
                match obj_ty {
                    Type::Array { element, .. } | Type::Slice { element } => *element,
                    _ => Type::Inferred { id: self.env.fresh_infer_id() },
                }
            }

            Expr::StructLiteral(sl) => {
                for field in &sl.fields {
                    self.infer_expr_type(&field.value);
                }
                if let Some(info) = self.env.get_struct_by_name(&sl.name) {
                    Type::Struct {
                        id: info.id,
                        name: sl.name.clone(),
                        generics: Vec::new(),
                    }
                } else {
                    self.errors.push(
                        Diagnostic::error(format!("unknown struct type '{}'", sl.name))
                            .with_span(sl.span),
                    );
                    Type::Error
                }
            }

            Expr::Closure(cl) => {
                let body_ty = self.infer_expr_type(&cl.body);
                let params: Vec<Type> = cl.params.iter().map(|p| {
                    p.type_expr.as_ref()
                        .map(|t| self.resolve_type_expr(t))
                        .unwrap_or(Type::Inferred { id: self.env.fresh_infer_id() })
                }).collect();
                Type::Function {
                    params,
                    ret: Box::new(body_ty),
                }
            }

            Expr::Propagate(prop) => {
                let inner_ty = self.infer_expr_type(&prop.inner);
                match inner_ty {
                    Type::Result { ok, .. } => *ok,
                    Type::Option { inner } => *inner,
                    _ => {
                        self.errors.push(
                            Diagnostic::error("'?' operator can only be used on Result or Option types")
                                .with_span(prop.span),
                        );
                        Type::Error
                    }
                }
            }

            Expr::ErrorConvert(ec) => {
                let _inner_ty = self.infer_expr_type(&ec.inner);
                let _fallback_ty = self.infer_expr_type(&ec.fallback);
                Type::Inferred { id: self.env.fresh_infer_id() }
            }

            Expr::Range(r) => {
                let start_ty = self.infer_expr_type(&r.start);
                let _end_ty = self.infer_expr_type(&r.end);
                Type::Array {
                    element: Box::new(start_ty),
                    size: 0,
                }
            }

            Expr::ArrayLiteral(al) => {
                let elem_ty = if let Some(first) = al.elements.first() {
                    self.infer_expr_type(first)
                } else {
                    Type::Inferred { id: self.env.fresh_infer_id() }
                };
                for elem in al.elements.iter().skip(1) {
                    self.infer_expr_type(elem);
                }
                Type::Array {
                    element: Box::new(elem_ty),
                    size: al.elements.len(),
                }
            }

            Expr::TupleLiteral(tl) => {
                let elements: Vec<Type> = tl.elements.iter()
                    .map(|e| self.infer_expr_type(e))
                    .collect();
                Type::Tuple { elements }
            }

            Expr::Path(_) => Type::Inferred { id: self.env.fresh_infer_id() },
            Expr::SelfExpr(_) => Type::Inferred { id: self.env.fresh_infer_id() },
            Expr::UnsafeBlock(ub) => {
                self.check_block(&ub.body);
                Type::Inferred { id: self.env.fresh_infer_id() }
            }

            Expr::If(if_expr) => {
                let _cond = self.infer_expr_type(&if_expr.condition);
                let then_ty = self.infer_expr_type(&if_expr.then_expr);
                if let Some(ref else_expr) = if_expr.else_expr {
                    self.infer_expr_type(else_expr);
                }
                then_ty
            }

            Expr::Match(match_expr) => {
                let _subj = self.infer_expr_type(&match_expr.subject);
                Type::Inferred { id: self.env.fresh_infer_id() }
            }

            Expr::Block(block) => {
                self.env.push_scope();
                self.check_block(block);
                self.env.pop_scope();
                Type::Unit
            }

            Expr::Pipeline(p) => {
                let _left = self.infer_expr_type(&p.left);
                self.infer_expr_type(&p.right)
            }

            Expr::Concat(c) => {
                let left_ty = self.infer_expr_type(&c.left);
                let _right_ty = self.infer_expr_type(&c.right);
                if left_ty != Type::Text && !left_ty.is_error() {
                    self.errors.push(
                        Diagnostic::error("'++' operator requires Text operands")
                            .with_span(c.span),
                    );
                }
                Type::Text
            }

            Expr::Cast(c) => self.resolve_type_expr(&c.target_type),
        }
    }

    /// Bind variables introduced by a pattern.
    fn bind_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding { name, .. } => {
                let id = self.env.fresh_infer_id();
                self.env.bind_var(name, Type::Inferred { id }, false);
            }
            Pattern::Variant { fields, .. } => {
                for field in fields {
                    self.bind_pattern(&field.pattern);
                }
            }
            Pattern::Tuple { elements, .. } => {
                for elem in elements {
                    self.bind_pattern(elem);
                }
            }
            Pattern::Wildcard { .. } | Pattern::Literal { .. } => {}
        }
    }

    // -- Type resolution --

    fn resolve_type_expr(&mut self, type_expr: &TypeExpr) -> Type {
        match type_expr {
            TypeExpr::Named { name, generics, .. } => {
                // Check for built-in types first
                if let Some(prim) = resolve_primitive(name) {
                    return prim;
                }

                // Check for Option and Result shorthands
                if name == "Option" && generics.len() == 1 {
                    let inner = self.resolve_type_expr(&generics[0]);
                    return Type::Option { inner: Box::new(inner) };
                }
                if name == "Result" && generics.len() == 2 {
                    let ok = self.resolve_type_expr(&generics[0]);
                    let err = self.resolve_type_expr(&generics[1]);
                    return Type::Result {
                        ok: Box::new(ok),
                        err: Box::new(err),
                    };
                }
                if name == "Slice" && generics.len() == 1 {
                    let elem = self.resolve_type_expr(&generics[0]);
                    return Type::Slice { element: Box::new(elem) };
                }
                if name == "Vec" && generics.len() == 1 {
                    let elem = self.resolve_type_expr(&generics[0]);
                    return Type::Array { element: Box::new(elem), size: 0 };
                }

                // Look up user-defined type
                if let Some(id) = self.env.lookup_type(name) {
                    if self.env.get_struct(id).is_some() {
                        let resolved_generics: Vec<Type> = generics
                            .iter()
                            .map(|g| self.resolve_type_expr(g))
                            .collect();
                        return Type::Struct {
                            id,
                            name: name.clone(),
                            generics: resolved_generics,
                        };
                    }
                    if self.env.get_enum(id).is_some() {
                        let resolved_generics: Vec<Type> = generics
                            .iter()
                            .map(|g| self.resolve_type_expr(g))
                            .collect();
                        return Type::Enum {
                            id,
                            name: name.clone(),
                            generics: resolved_generics,
                        };
                    }
                }

                // Could be a generic type parameter
                Type::TypeParam { name: name.clone() }
            }

            TypeExpr::UnitType { base, unit, span: _ } => {
                let base_type = if let Some(prim) = resolve_primitive(base) {
                    prim
                } else {
                    Type::Float64
                };
                Type::UnitAnnotated {
                    base: Box::new(base_type),
                    unit_name: unit.clone(),
                }
            }

            TypeExpr::Array { element, size, .. } => {
                let elem = self.resolve_type_expr(element);
                let sz = if let Expr::Literal(LiteralExpr { value: LiteralValue::Int(n), .. }) = size.as_ref() {
                    *n as usize
                } else {
                    0
                };
                Type::Array { element: Box::new(elem), size: sz }
            }

            TypeExpr::Slice { element, .. } => {
                let elem = self.resolve_type_expr(element);
                Type::Slice { element: Box::new(elem) }
            }

            TypeExpr::Tuple { elements, .. } => {
                let resolved: Vec<Type> = elements.iter().map(|e| self.resolve_type_expr(e)).collect();
                Type::Tuple { elements: resolved }
            }

            TypeExpr::Reference { inner, is_mut, .. } => {
                let inner_type = self.resolve_type_expr(inner);
                Type::Ref { inner: Box::new(inner_type), mutable: *is_mut }
            }

            TypeExpr::Result { ok_type, err_type, .. } => {
                let ok = self.resolve_type_expr(ok_type);
                let err = self.resolve_type_expr(err_type);
                Type::Result { ok: Box::new(ok), err: Box::new(err) }
            }

            TypeExpr::Option { inner, .. } => {
                let inner_type = self.resolve_type_expr(inner);
                Type::Option { inner: Box::new(inner_type) }
            }

            TypeExpr::SelfType { .. } => Type::TypeParam { name: "Self".to_string() },
            TypeExpr::Inferred { .. } => Type::Inferred { id: self.env.fresh_infer_id() },
            TypeExpr::Never { .. } => Type::Never,

            TypeExpr::Path { segments, generics, .. } => {
                let name = segments.last().cloned().unwrap_or_default();
                if let Some(id) = self.env.lookup_type(&name) {
                    let resolved_generics: Vec<Type> = generics
                        .iter()
                        .map(|g| self.resolve_type_expr(g))
                        .collect();
                    Type::Struct { id, name, generics: resolved_generics }
                } else {
                    Type::TypeParam { name }
                }
            }
        }
    }

    /// Check whether two types are compatible for assignment.
    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }
        // Error type is compatible with everything (for error recovery)
        if expected.is_error() || actual.is_error() {
            return true;
        }
        // Inferred is compatible with everything (type inference not yet fully resolved)
        if matches!(expected, Type::Inferred { .. }) || matches!(actual, Type::Inferred { .. }) {
            return true;
        }
        // Int is compatible with all integer types (implicit widening)
        if expected.is_integer() && actual.is_integer() {
            return true;
        }
        // Float64 is compatible with Float32 (widening)
        if matches!(expected, Type::Float64) && matches!(actual, Type::Float32) {
            return true;
        }
        false
    }

    fn check_lvalue_mutability(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(id) => {
                if let Some(binding) = self.env.lookup_var(&id.name) {
                    binding.mutable
                } else {
                    true // Global or not found
                }
            }
            Expr::FieldAccess(fa) => self.check_lvalue_mutability(&fa.object),
            Expr::Index(idx) => self.check_lvalue_mutability(&idx.object),
            Expr::SelfExpr(_) => {
                if let Some(binding) = self.env.lookup_var("self") {
                    binding.mutable
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
