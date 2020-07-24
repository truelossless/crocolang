use crate::ast::AstNode;
use crate::builtin::{get_module, BuiltinCallback, BuiltinFunction, BuiltinVar};
use crate::parser::TypedArg;
use crate::token::{literal_eq, Identifier, LiteralEnum};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Either the function is a classic function or a built-in function
#[derive(Clone)]
pub enum FunctionKind {
    Regular(Box<dyn AstNode>),
    Builtin(BuiltinCallback),
}

impl fmt::Debug for FunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionKind::Regular(_) => write!(f, "Regular"),
            FunctionKind::Builtin(_) => write!(f, "Builtin"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub args: Vec<TypedArg>,
    pub body: FunctionKind,
    pub return_type: SymbolContent,
}

impl FunctionDecl {
    pub fn new(args: Vec<TypedArg>, return_type: SymbolContent, body: FunctionKind) -> Self {
        FunctionDecl {
            args,
            return_type,
            body,
        }
    }
}

/// representation of a built struct
#[derive(Clone, Debug)]
pub struct Struct {
    // the fields, populated with values
    // we use an option because in declarations we don't want the overhead of an HashMap allocation
    pub fields: Option<HashMap<String, Symbol>>,

    // the corresponding type of the struct, as as StructDecl
    // TODO: consider edge cases where someone could override a struct with a deeper scoped struct of the same name
    pub struct_type: String,
}

impl Struct {
    pub fn new(struct_type: String) -> Self {
        Struct {
            fields: None,
            struct_type,
        }
    }
}

#[derive(Clone)]
pub struct Map {
    pub contents: HashMap<Symbol, Symbol>,
    pub key_type: Box<SymbolContent>,
    pub value_type: Box<SymbolContent>,
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map<{:?}, {:?}>", self.key_type, self.value_type)
    }
}

#[derive(Clone, Debug)]
pub struct Array {
    pub contents: Option<Vec<Symbol>>,
    pub array_type: Box<SymbolContent>,
}

/// a symbol in the symbol table. It could be either a primitive, a function, a struct ...
// I've tried multiple times to use references with lifetimes annotations but it's too hard in graph structures
// I had to annotate all the nodes and structs with lifetimes and it didn't even work :S
// this is why I'm using Rc as it eases the process a lot.
// If you have an idea for a more elegant implementation I'm all ears !
pub type Symbol = Rc<RefCell<SymbolContent>>;

#[derive(Clone, Debug)]
/// the symbol contents
pub enum SymbolContent {
    /// a primitive such as 3 or false
    Primitive(LiteralEnum),

    /// an array such as [1, 2, 3]
    Array(Array),

    // a key-value map such as { "hello" => 5, "bonjour" => 4 }
    // Map(Map),

    // a pointer to a function such as let "a = b" where b is a FunctionDecl
    // Function(FunctionPointer)

    // a function, local to this scope (will be useful for struct methods)
    // FunctionClosure(FunctionDecl)
    /// a symbol reference  
    Ref(Symbol),

    // a structure built from a StructDecl such as "let a = b {}"
    Struct(Struct),
}

impl SymbolContent {
    pub fn into_primitive(self) -> Result<LiteralEnum, String> {
        match self {
            SymbolContent::Primitive(p) => Ok(p),
            _ => Err("expected a primitive".to_owned()),
        }
    }

    pub fn into_struct(self) -> Result<Struct, String> {
        match self {
            SymbolContent::Struct(s) => Ok(s),
            _ => Err("expected a struct".to_owned()),
        }
    }

    pub fn into_array(self) -> Result<Array, String> {
        match self {
            SymbolContent::Array(a) => Ok(a),
            _ => Err("expected an array".to_owned()),
        }
    }

    /// dereferences a symbol
    pub fn into_ref(self) -> Result<Symbol, String> {
        match self {
            SymbolContent::Ref(r) => Ok(r),
            _ => Err("expected a reference".to_owned()),
        }
    }

    pub fn is_void(&self) -> bool {
        match self {
            SymbolContent::Primitive(LiteralEnum::Void) => true,
            _ => false,
        }
    }
}

/// a top-level declaration such as a function declaration or a struct declaration
#[derive(Clone, Debug)]
pub enum Decl {
    // the blueprint of a function such as "fn a {}"
    FunctionDecl(FunctionDecl),

    // the blueprint of a struct such as "struct a {}"
    StructDecl(HashMap<String, Symbol>),
}

/// compare if two symbols are of the same type
pub fn symbol_eq(a: &SymbolContent, b: &SymbolContent) -> bool {
    match (a, b) {
        (SymbolContent::Primitive(a), SymbolContent::Primitive(b)) => literal_eq(a, b),
        (SymbolContent::Struct(a), SymbolContent::Struct(b)) => a.struct_type == b.struct_type,
        (SymbolContent::Array(a), SymbolContent::Array(b)) => {
            symbol_eq(&*a.array_type, &*b.array_type)
        }
        (SymbolContent::Ref(a), SymbolContent::Ref(b)) => {
            symbol_eq(&*a.borrow(), &*b.borrow())
        }
        _ => false,
    }
}

// returns the type of a symbol
pub fn get_symbol_type(symbol: Symbol) -> SymbolContent {
    match &*symbol.borrow() {
        SymbolContent::Primitive(LiteralEnum::Bool(_)) => {
            SymbolContent::Primitive(LiteralEnum::Bool(None))
        }
        SymbolContent::Primitive(LiteralEnum::Num(_)) => {
            SymbolContent::Primitive(LiteralEnum::Num(None))
        }
        SymbolContent::Primitive(LiteralEnum::Str(_)) => {
            SymbolContent::Primitive(LiteralEnum::Str(None))
        }
        SymbolContent::Primitive(LiteralEnum::Void) => SymbolContent::Primitive(LiteralEnum::Void),
        SymbolContent::Array(arr) => SymbolContent::Array(Array {
            contents: None,
            array_type: arr.array_type.clone(),
        }),
        SymbolContent::Struct(s) => SymbolContent::Struct(Struct {
            fields: None,
            struct_type: s.struct_type.clone(),
        }),
        SymbolContent::Ref(r) => {
            SymbolContent::Ref(Rc::new(RefCell::new(get_symbol_type(r.clone()))))
        }
    }
}

/// SymTable represents symbol tables where all the variables are stored.
/// The Vec represents the different scopes of variables, introduced by BlockNodes
/// The Hashmap stores variables by name, and bind them to a value.
/// Top level contains all struct and function declarations.
#[derive(Clone, Default, Debug)]
pub struct SymTable {
    symbols: Vec<HashMap<String, Symbol>>,
    top_level: HashMap<String, Decl>,
}

impl SymTable {
    pub fn new() -> Self {
        SymTable {
            symbols: vec![HashMap::new()],
            top_level: HashMap::new(),
        }
    }

    /// return the desired symbol starting from the inner scope
    pub fn get_symbol(&mut self, var_name: &str) -> Result<Symbol, String> {
        for table in self.symbols.iter().rev() {
            if let Some(symbol) = table.get(var_name) {
                return Ok(symbol.clone());
            }
        }

        // the variable doesn't exist
        Err(format!("variable {} has not been declared", var_name))
    }

    /// return the desired function declaration starting from the inner scope
    pub fn get_function_decl(&mut self, fn_name: &str) -> Result<&mut FunctionDecl, String> {
        match self.top_level.get_mut(fn_name) {
            Some(Decl::FunctionDecl(ref mut function)) => return Ok(function),
            Some(_) => {
                return Err(format!(
                    "trying to get {} as a function but it's not",
                    fn_name
                ))
            }
            None => (),
        }

        // the function doesn't exist
        Err(format!("function {} has not been declared", fn_name))
    }

    /// return the desired struct declaration starting from the inner scope
    pub fn get_struct_decl(
        &mut self,
        struct_type: &str,
    ) -> Result<&mut HashMap<String, Symbol>, String> {
        match self.top_level.get_mut(struct_type) {
            Some(Decl::StructDecl(ref mut struct_decl)) => return Ok(struct_decl),
            Some(_) => {
                return Err(format!(
                    "trying to get {} as a struct but it's not",
                    struct_type
                ))
            }
            None => (),
        }

        // the function doesn't exist
        Err(format!("struct {} has not been declared", struct_type))
    }

    /// insert to the closest scope
    pub fn insert_symbol(&mut self, name: &str, symbol: Symbol) -> Result<(), String> {
        if self
            .symbols
            .last_mut()
            .unwrap()
            .insert(name.to_owned(), symbol)
            .is_none()
        {
            Ok(())
        } else {
            Err(format!(
                "variable already declared with name {} in this scope",
                name
            ))
        }
    }

    /// insert to the global scope
    pub fn insert_global_symbol(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        if self
            .symbols
            .first_mut()
            .unwrap()
            .insert(name.to_owned(), symbol)
            .is_none()
        {
            Ok(())
        } else {
            Err(format!(
                "variable already declared with name {} in this scope",
                name
            ))
        }
    }

    /// modify a symbol already present in the symbol table
    pub fn modify_symbol(&mut self, name: &str, symbol: Symbol) -> Result<(), String> {
        for table in self.symbols.iter_mut().rev() {
            if let Some(old_symbol) = table.get_mut(name) {
                
                let old_symbol_borrow = &mut *old_symbol.borrow_mut();
                let new_symbol_borrow = &*symbol.borrow();

                if symbol_eq(old_symbol_borrow, new_symbol_borrow) {
                    *old_symbol_borrow = new_symbol_borrow.clone();
                    return Ok(());
                } else {
                    return Err(format!(
                        "Cannot assign another type to the variable {}",
                        name
                    ));
                }
            }
        }
        Err(format!("Can't assign to undeclared variable {}", name))
    }

    /// register a function or struct declaration
    pub fn register_decl(&mut self, var_name: String, decl: Decl) -> Result<(), String> {
        let res = self.top_level.insert(var_name, decl);

        if res.is_some() {
            Err("symbol already declared in this scope".to_owned())
        } else {
            Ok(())
        }
    }

    pub fn import_builtin_module(&mut self, name: &str) -> bool {
        if let Some(module) = get_module(name) {
            // the global module doesn't have any namespace
            let namespace = if name == "global" { "" } else { name };

            for function in module.functions {
                self.register_builtin_function(function, namespace);
            }

            for var in module.vars {
                self.register_builtin_var(var, namespace);
            }
            true
        } else {
            false
        }
    }

    pub fn register_builtin_var(&mut self, var: BuiltinVar, module_name: &str) {
        let namespaced_name =
            Identifier::new(var.name, module_name.to_owned()).get_namespaced_name();
        self.insert_global_symbol(namespaced_name, Rc::new(RefCell::new(var.value)))
            .unwrap();
    }

    pub fn register_builtin_function(&mut self, function: BuiltinFunction, module_name: &str) {
        // for the builtin functions we don't care about the variable name
        let mut typed_args = Vec::new();

        for el in function.args.into_iter() {
            typed_args.push(TypedArg {
                arg_name: String::new(),
                arg_type: el,
            });
        }

        let builtin = FunctionDecl::new(
            typed_args,
            function.return_type,
            FunctionKind::Builtin(function.pointer),
        );

        let namespaced_name =
            Identifier::new(function.name, module_name.to_owned()).get_namespaced_name();

        self.register_decl(namespaced_name, Decl::FunctionDecl(builtin))
            .unwrap();
    }

    pub fn add_scope(&mut self) {
        self.symbols.push(HashMap::new());
    }

    pub fn drop_scope(&mut self) {
        self.symbols.pop();
    }
}
