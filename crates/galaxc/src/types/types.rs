// Semantic type representations used by the type checker.
// These are fully resolved types with no syntactic ambiguity.

// HashMap removed

/// Unique identifier for user-defined types (structs, enums, abilities).
pub type TypeId = u64;

/// A resolved semantic type.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Char,
    Text,
    Byte,
    Never,
    Unit,  // void/() equivalent

    // Compound types
    Array { element: Box<Type>, size: usize },
    Slice { element: Box<Type> },
    Tuple { elements: Vec<Type> },
    Option { inner: Box<Type> },
    Result { ok: Box<Type>, err: Box<Type> },

    // User-defined types
    Struct { id: TypeId, name: String, generics: Vec<Type> },
    Enum { id: TypeId, name: String, generics: Vec<Type> },

    // Reference types
    Ref { inner: Box<Type>, mutable: bool },

    // Function type (for closures and function pointers)
    Function { params: Vec<Type>, ret: Box<Type> },

    // Generic type parameter (unresolved during checking)
    TypeParam { name: String },

    // Unit-annotated numeric type
    UnitAnnotated { base: Box<Type>, unit_name: String },

    // Error placeholder for recovery after type errors
    Error,

    // Inferred (not yet determined)
    Inferred { id: u64 },
}

impl Type {
    /// Check if this is a numeric type suitable for arithmetic.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64
            | Type::Uint8 | Type::Uint16 | Type::Uint32 | Type::Uint64
            | Type::Float32 | Type::Float64
            | Type::UnitAnnotated { .. }
        )
    }

    /// Check if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64
            | Type::Uint8 | Type::Uint16 | Type::Uint32 | Type::Uint64
            | Type::Byte
        )
    }

    /// Check if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float32 | Type::Float64)
    }

    /// Check if this is the error recovery type.
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    /// Get the C type name for code generation.
    pub fn c_type_name(&self) -> String {
        match self {
            Type::Bool => "bool".to_string(),
            Type::Int => "int64_t".to_string(),
            Type::Int8 => "int8_t".to_string(),
            Type::Int16 => "int16_t".to_string(),
            Type::Int32 => "int32_t".to_string(),
            Type::Int64 => "int64_t".to_string(),
            Type::Uint8 => "uint8_t".to_string(),
            Type::Uint16 => "uint16_t".to_string(),
            Type::Uint32 => "uint32_t".to_string(),
            Type::Uint64 => "uint64_t".to_string(),
            Type::Float32 => "float".to_string(),
            Type::Float64 => "double".to_string(),
            Type::Char => "uint32_t".to_string(),
            Type::Text => "GxcText".to_string(),
            Type::Byte => "uint8_t".to_string(),
            Type::Never => "void".to_string(),
            Type::Unit => "void".to_string(),
            Type::Array { element, size: _ } => {
                format!("GxcArray_{}", element.c_type_name())
            }
            Type::Slice { element } => {
                format!("GxcSlice_{}", element.c_type_name())
            }
            Type::Struct { name, .. } => format!("Gxc_{name}"),
            Type::Enum { name, .. } => format!("Gxc_{name}"),
            Type::Option { inner } => format!("GxcOption_{}", inner.c_type_name()),
            Type::Result { ok, err } => {
                format!("GxcResult_{}_{}", ok.c_type_name(), err.c_type_name())
            }
            Type::Ref { inner, .. } => format!("{}*", inner.c_type_name()),
            Type::Function { .. } => "GxcClosure".to_string(),
            Type::Tuple { elements } => {
                let parts: Vec<_> = elements.iter().map(|t| t.c_type_name()).collect();
                format!("GxcTuple_{}", parts.join("_"))
            }
            Type::TypeParam { name } => format!("GxcTypeParam_{name}"),
            Type::UnitAnnotated { base, .. } => base.c_type_name(),
            Type::Error => "void /* error */".to_string(),
            Type::Inferred { id } => format!("GxcInferred_{id}"),
        }
    }

    /// Human-readable display name for diagnostics.
    pub fn display_name(&self) -> String {
        match self {
            Type::Bool => "Bool".to_string(),
            Type::Int => "Int".to_string(),
            Type::Int8 => "Int8".to_string(),
            Type::Int16 => "Int16".to_string(),
            Type::Int32 => "Int32".to_string(),
            Type::Int64 => "Int64".to_string(),
            Type::Uint8 => "Uint8".to_string(),
            Type::Uint16 => "Uint16".to_string(),
            Type::Uint32 => "Uint32".to_string(),
            Type::Uint64 => "Uint64".to_string(),
            Type::Float32 => "Float32".to_string(),
            Type::Float64 => "Float64".to_string(),
            Type::Char => "Char".to_string(),
            Type::Text => "Text".to_string(),
            Type::Byte => "Byte".to_string(),
            Type::Never => "Never".to_string(),
            Type::Unit => "()".to_string(),
            Type::Array { element, size } => format!("[{}; {size}]", element.display_name()),
            Type::Slice { element } => format!("Slice<{}>", element.display_name()),
            Type::Tuple { elements } => {
                let parts: Vec<_> = elements.iter().map(|t| t.display_name()).collect();
                format!("({})", parts.join(", "))
            }
            Type::Option { inner } => format!("Option<{}>", inner.display_name()),
            Type::Result { ok, err } => {
                format!("Result<{}, {}>", ok.display_name(), err.display_name())
            }
            Type::Struct { name, generics, .. } | Type::Enum { name, generics, .. } => {
                if generics.is_empty() {
                    name.clone()
                } else {
                    let params: Vec<_> = generics.iter().map(|t| t.display_name()).collect();
                    format!("{name}<{}>", params.join(", "))
                }
            }
            Type::Ref { inner, mutable } => {
                if *mutable {
                    format!("mut ref {}", inner.display_name())
                } else {
                    format!("ref {}", inner.display_name())
                }
            }
            Type::Function { params, ret } => {
                let p: Vec<_> = params.iter().map(|t| t.display_name()).collect();
                format!("op({}) -> {}", p.join(", "), ret.display_name())
            }
            Type::TypeParam { name } => name.clone(),
            Type::UnitAnnotated { base, unit_name } => {
                format!("{}<{unit_name}>", base.display_name())
            }
            Type::Error => "<error>".to_string(),
            Type::Inferred { .. } => "<inferred>".to_string(),
        }
    }
}

/// Information about a struct type registered in the type environment.
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub id: TypeId,
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub generic_params: Vec<String>,
    pub methods: Vec<FunctionInfo>,
}

/// Information about a single struct or enum field.
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: Type,
}

/// Information about an enum type.
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub id: TypeId,
    pub name: String,
    pub variants: Vec<VariantInfo>,
    pub generic_params: Vec<String>,
}

/// Information about a single enum variant.
#[derive(Debug, Clone)]
pub struct VariantInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

/// Information about a function signature.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub return_type: Type,
    pub generic_params: Vec<String>,
    pub effects: Vec<String>,
    pub is_pub: bool,
}

/// Information about a function parameter.
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub ty: Type,
    pub is_mut: bool,
}

/// Information about an ability (trait).
#[derive(Debug, Clone)]
pub struct AbilityInfo {
    pub id: TypeId,
    pub name: String,
    pub methods: Vec<FunctionInfo>,
    pub generic_params: Vec<String>,
}

/// Information about a task type.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub name: String,
    pub entries: Vec<FunctionInfo>,
}

/// Resolve a type name string to a primitive Type, if one matches.
pub fn resolve_primitive(name: &str) -> Option<Type> {
    match name {
        "Bool" => Some(Type::Bool),
        "Int" => Some(Type::Int),
        "Int8" => Some(Type::Int8),
        "Int16" => Some(Type::Int16),
        "Int32" => Some(Type::Int32),
        "Int64" => Some(Type::Int64),
        "Uint8" => Some(Type::Uint8),
        "Uint16" => Some(Type::Uint16),
        "Uint32" => Some(Type::Uint32),
        "Uint64" => Some(Type::Uint64),
        "Float32" => Some(Type::Float32),
        "Float64" => Some(Type::Float64),
        "Char" => Some(Type::Char),
        "Text" => Some(Type::Text),
        "Byte" => Some(Type::Byte),
        "Never" => Some(Type::Never),
        _ => None,
    }
}
