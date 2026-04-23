// AST visitor trait. Provides a walk-the-tree pattern for passes that
// need to inspect or transform every node (type checking, IR lowering, etc.)

use super::nodes::*;

/// Visitor trait with default no-op implementations. Override only the
/// node types your pass cares about.
pub trait AstVisitor {
    fn visit_program(&mut self, program: &Program) {
        if let Some(ref module) = program.module_decl {
            self.visit_module_decl(module);
        }
        for import in &program.imports {
            self.visit_import(import);
        }
        for item in &program.items {
            self.visit_item(item);
        }
    }

    fn visit_module_decl(&mut self, _module: &ModuleDecl) {}
    fn visit_import(&mut self, _import: &ImportDecl) {}

    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.visit_function(f),
            Item::Struct(s) => self.visit_struct(s),
            Item::Enum(e) => self.visit_enum(e),
            Item::Ability(a) => self.visit_ability(a),
            Item::ImplBlock(i) => self.visit_impl(i),
            Item::Constant(c) => self.visit_const(c),
            Item::TaskDecl(t) => self.visit_task_decl(t),
            Item::TaskBody(t) => self.visit_task_body(t),
            Item::ProtectedDecl(p) => self.visit_protected(p),
            Item::UnitDecl(u) => self.visit_unit_decl(u),
            Item::ExternBlock(e) => self.visit_extern(e),
            Item::StaticAssert(s) => self.visit_static_assert(s),
        }
    }

    fn visit_function(&mut self, func: &FunctionDecl) {
        if let Some(ref body) = func.body {
            self.visit_block(body);
        }
    }

    fn visit_struct(&mut self, _decl: &StructDecl) {}
    fn visit_enum(&mut self, _decl: &EnumDecl) {}
    fn visit_ability(&mut self, _decl: &AbilityDecl) {}

    fn visit_impl(&mut self, block: &ImplBlock) {
        for method in &block.methods {
            self.visit_function(method);
        }
    }

    fn visit_const(&mut self, _decl: &ConstDecl) {}
    fn visit_task_decl(&mut self, _decl: &TaskDecl) {}

    fn visit_task_body(&mut self, decl: &TaskBodyDecl) {
        self.visit_block(&decl.body);
    }

    fn visit_protected(&mut self, _decl: &ProtectedBlock) {}
    fn visit_unit_decl(&mut self, _decl: &UnitDeclNode) {}
    fn visit_extern(&mut self, _decl: &ExternBlock) {}
    fn visit_static_assert(&mut self, _node: &StaticAssertNode) {}

    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(s) => self.visit_let(s),
            Stmt::Var(s) => self.visit_var(s),
            Stmt::Assign(s) => self.visit_assign(s),
            Stmt::Expr(s) => self.visit_expr_stmt(s),
            Stmt::If(s) => self.visit_if(s),
            Stmt::Match(s) => self.visit_match(s),
            Stmt::For(s) => self.visit_for(s),
            Stmt::While(s) => self.visit_while(s),
            Stmt::Loop(s) => self.visit_loop(s),
            Stmt::Return(s) => self.visit_return(s),
            Stmt::Break(_) => {}
            Stmt::Continue(_) => {}
            Stmt::Select(s) => self.visit_select(s),
            Stmt::Item(item) => self.visit_item(item),
        }
    }

    fn visit_let(&mut self, stmt: &LetStmt) {
        self.visit_expr(&stmt.value);
    }

    fn visit_var(&mut self, stmt: &VarStmt) {
        self.visit_expr(&stmt.value);
    }

    fn visit_assign(&mut self, stmt: &AssignStmt) {
        self.visit_expr(&stmt.target);
        self.visit_expr(&stmt.value);
    }

    fn visit_expr_stmt(&mut self, stmt: &ExprStmt) {
        self.visit_expr(&stmt.expr);
    }

    fn visit_if(&mut self, stmt: &IfStmt) {
        self.visit_expr(&stmt.condition);
        self.visit_block(&stmt.then_block);
        for (cond, block) in &stmt.else_ifs {
            self.visit_expr(cond);
            self.visit_block(block);
        }
        if let Some(ref block) = stmt.else_block {
            self.visit_block(block);
        }
    }

    fn visit_match(&mut self, stmt: &MatchStmt) {
        self.visit_expr(&stmt.subject);
    }

    fn visit_for(&mut self, stmt: &ForStmt) {
        self.visit_expr(&stmt.iterable);
        self.visit_block(&stmt.body);
    }

    fn visit_while(&mut self, stmt: &WhileStmt) {
        self.visit_expr(&stmt.condition);
        self.visit_block(&stmt.body);
    }

    fn visit_loop(&mut self, stmt: &LoopStmt) {
        self.visit_block(&stmt.body);
    }

    fn visit_return(&mut self, stmt: &ReturnStmt) {
        if let Some(ref value) = stmt.value {
            self.visit_expr(value);
        }
    }

    fn visit_select(&mut self, _stmt: &SelectStmt) {}

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(_) => {}
            Expr::Identifier(_) => {}
            Expr::Binary(e) => {
                self.visit_expr(&e.left);
                self.visit_expr(&e.right);
            }
            Expr::Unary(e) => self.visit_expr(&e.operand),
            Expr::Call(e) => {
                self.visit_expr(&e.callee);
                for arg in &e.args {
                    self.visit_expr(&arg.value);
                }
            }
            Expr::MethodCall(e) => {
                self.visit_expr(&e.receiver);
                for arg in &e.args {
                    self.visit_expr(&arg.value);
                }
            }
            Expr::FieldAccess(e) => self.visit_expr(&e.object),
            Expr::Index(e) => {
                self.visit_expr(&e.object);
                self.visit_expr(&e.index);
            }
            Expr::StructLiteral(e) => {
                for field in &e.fields {
                    self.visit_expr(&field.value);
                }
            }
            Expr::Closure(e) => self.visit_expr(&e.body),
            Expr::If(e) => {
                self.visit_expr(&e.condition);
                self.visit_expr(&e.then_expr);
                if let Some(ref else_expr) = e.else_expr {
                    self.visit_expr(else_expr);
                }
            }
            Expr::Match(e) => self.visit_expr(&e.subject),
            Expr::Block(b) => self.visit_block(b),
            Expr::Propagate(e) => self.visit_expr(&e.inner),
            Expr::ErrorConvert(e) => {
                self.visit_expr(&e.inner);
                self.visit_expr(&e.fallback);
            }
            Expr::Range(e) => {
                self.visit_expr(&e.start);
                self.visit_expr(&e.end);
            }
            Expr::ArrayLiteral(e) => {
                for elem in &e.elements {
                    self.visit_expr(elem);
                }
            }
            Expr::TupleLiteral(e) => {
                for elem in &e.elements {
                    self.visit_expr(elem);
                }
            }
            Expr::Path(_) => {}
            Expr::UnsafeBlock(e) => self.visit_block(&e.body),
            Expr::SelfExpr(_) => {}
            Expr::Pipeline(e) => {
                self.visit_expr(&e.left);
                self.visit_expr(&e.right);
            }
            Expr::Concat(e) => {
                self.visit_expr(&e.left);
                self.visit_expr(&e.right);
            }
        }
    }
}
