// Type environment -- tracks variable bindings, type definitions,
// function signatures, and scope nesting during type checking.

use std::collections::HashMap;
use crate::types::*;

/// A single scope level in the type environment.
#[derive(Debug, Clone)]
struct Scope {
    variables: HashMap<String, VarBinding>,
    types: HashMap<String, TypeId>,
}

/// Information about a bound variable.
#[derive(Debug, Clone)]
pub struct VarBinding {
    pub ty: Type,
    pub mutable: bool,
    pub _initialized: bool,
}

/// The type environment tracks all named entities across nested scopes.
pub struct TypeEnv {
    scopes: Vec<Scope>,
    structs: HashMap<TypeId, StructInfo>,
    enums: HashMap<TypeId, EnumInfo>,
    abilities: HashMap<TypeId, AbilityInfo>,
    functions: HashMap<String, FunctionInfo>,
    tasks: HashMap<String, TaskInfo>,
    next_type_id: TypeId,
    next_infer_id: u64,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            scopes: vec![Scope {
                variables: HashMap::new(),
                types: HashMap::new(),
            }],
            structs: HashMap::new(),
            enums: HashMap::new(),
            abilities: HashMap::new(),
            functions: HashMap::new(),
            tasks: HashMap::new(),
            next_type_id: 1,
            next_infer_id: 1,
        };

        // Register built-in types
        env.register_builtin_enum("Option", &["Some", "None"]);
        env.register_builtin_enum("Result", &["Ok", "Err"]);

        // Register console built-in
        let console_id = env.fresh_type_id();
        env.structs.insert(console_id, StructInfo {
            id: console_id,
            name: "Console".to_string(),
            fields: Vec::new(),
            generic_params: Vec::new(),
            methods: vec![FunctionInfo {
                name: "write".to_string(),
                params: vec![ParamInfo {
                    name: "text".to_string(),
                    ty: Type::Text,
                    is_mut: false,
                }],
                return_type: Type::Unit,
                generic_params: Vec::new(),
                effects: vec!["io".to_string()],
                is_pub: true,
            }],
        });
        env.current_scope_mut().types.insert("Console".to_string(), console_id);
        env.bind_var("console", Type::Struct { id: console_id, name: "Console".to_string(), generics: Vec::new() }, false);

        env
    }

    fn register_builtin_enum(&mut self, name: &str, variants: &[&str]) {
        let id = self.fresh_type_id();
        let variant_infos: Vec<VariantInfo> = variants
            .iter()
            .map(|v| VariantInfo {
                name: v.to_string(),
                fields: Vec::new(),
            })
            .collect();
        self.enums.insert(id, EnumInfo {
            id,
            name: name.to_string(),
            variants: variant_infos,
            generic_params: vec!["T".to_string()],
        });
        self.current_scope_mut().types.insert(name.to_string(), id);
    }

    pub fn fresh_type_id(&mut self) -> TypeId {
        let id = self.next_type_id;
        self.next_type_id += 1;
        id
    }

    pub fn fresh_infer_id(&mut self) -> u64 {
        let id = self.next_infer_id;
        self.next_infer_id += 1;
        id
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
            types: HashMap::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().expect("scope stack must never be empty")
    }

    /// Bind a variable in the current scope.
    pub fn bind_var(&mut self, name: &str, ty: Type, mutable: bool) {
        self.current_scope_mut().variables.insert(
            name.to_string(),
            VarBinding {
                ty,
                mutable,
                _initialized: true,
            },
        );
    }

    /// Look up a variable by name, searching from innermost to outermost scope.
    pub fn lookup_var(&self, name: &str) -> Option<&VarBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.variables.get(name) {
                return Some(binding);
            }
        }
        None
    }

    /// Register a struct definition.
    pub fn register_struct(&mut self, info: StructInfo) {
        let id = info.id;
        let name = info.name.clone();
        self.structs.insert(id, info);
        self.current_scope_mut().types.insert(name, id);
    }

    /// Register an enum definition.
    pub fn register_enum(&mut self, info: EnumInfo) {
        let id = info.id;
        let name = info.name.clone();
        self.enums.insert(id, info);
        self.current_scope_mut().types.insert(name, id);
    }

    /// Register an ability definition.
    pub fn register_ability(&mut self, info: AbilityInfo) {
        let id = info.id;
        let name = info.name.clone();
        self.abilities.insert(id, info);
        self.current_scope_mut().types.insert(name, id);
    }

    /// Register a function.
    pub fn register_function(&mut self, info: FunctionInfo) {
        self.functions.insert(info.name.clone(), info);
    }

    /// Register a task.
    pub fn register_task(&mut self, info: TaskInfo) {
        self.tasks.insert(info.name.clone(), info);
    }

    /// Look up a type by name.
    pub fn lookup_type(&self, name: &str) -> Option<TypeId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.types.get(name) {
                return Some(*id);
            }
        }
        None
    }

    /// Get struct info by ID.
    pub fn get_struct(&self, id: TypeId) -> Option<&StructInfo> {
        self.structs.get(&id)
    }

    /// Get enum info by ID.
    pub fn get_enum(&self, id: TypeId) -> Option<&EnumInfo> {
        self.enums.get(&id)
    }

    /// Get function info by name.
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    /// Get struct info by name.
    pub fn get_struct_by_name(&self, name: &str) -> Option<&StructInfo> {
        if let Some(id) = self.lookup_type(name) {
            self.structs.get(&id)
        } else {
            None
        }
    }
}
