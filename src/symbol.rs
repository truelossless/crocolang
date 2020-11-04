use crate::ast::AstNode;
use crate::builtin::{BuiltinCallback, BuiltinFunction};
use crate::parser::TypedArg;
use crate::{symbol_type::SymbolType, token::Identifier};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

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
    pub return_type: Option<SymbolType>,
}

#[derive(Clone)]
pub struct StructDecl {
    // in crocol struct fields are order dependant.
    // to guarentee that our map is sorted, use a BTreeMap
    pub fields: BTreeMap<String, SymbolType>,
    // TODO: generate global functions instead of this
    // and desugar struct.call(foo) to struct_call(&struct, foo)
    pub methods: HashMap<String, FunctionDecl>,
}

impl fmt::Debug for StructDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StructDecl, fields: {:?}", self.fields)
    }
}

/// a top-level declaration such as a function declaration or a struct declaration
#[derive(Clone, Debug)]
pub enum Decl {
    // the blueprint of a function such as "fn a { .. }"
    FunctionDecl(FunctionDecl),

    // the blueprint of a struct such as "struct A { .. }"
    StructDecl(StructDecl),
}

/// SymTable represents the symbol tables where all the variables are stored.
/// The Vec represents the different scopes of variables, introduced by BlockNodes
/// The Hashmap stores variables by name, and bind them to a value.
/// Top level contains all struct and function declarations.

/// The SymTable is a generic struct that can accept any Symbol type, coming from any backend.
#[derive(Clone, Default, Debug)]
pub struct SymTable<T: Clone> {
    symbols: Vec<HashMap<String, T>>,
    top_level: HashMap<String, Decl>,
}

impl<T: Clone> SymTable<T> {
    pub fn new() -> Self {
        SymTable {
            symbols: vec![HashMap::new()],
            top_level: HashMap::new(),
        }
    }

    /// return the desired symbol starting from the inner scope
    pub fn get_symbol<'a>(&'a self, var_name: &str) -> Result<&'a T, String> {
        for table in self.symbols.iter().rev() {
            if let Some(symbol) = table.get(var_name) {
                return Ok(symbol);
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
    pub fn get_struct_decl(&self, struct_type: &str) -> Result<&StructDecl, String> {
        match self.top_level.get(struct_type) {
            Some(Decl::StructDecl(struct_decl)) => Ok(struct_decl),
            Some(_) => Err(format!(
                "trying to get {} as a struct but it's not",
                struct_type
            )),
            None => Err(format!("struct {} has not been declared", struct_type)),
        }
    }

    /// insert to the closest scope
    pub fn insert_symbol(&mut self, name: &str, symbol: T) -> Result<(), String> {
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
    pub fn insert_global_symbol(&mut self, name: String, symbol: T) -> Result<(), String> {
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

    /// register a function or struct declaration
    pub fn register_decl(&mut self, var_name: String, decl: Decl) -> Result<(), String> {
        let res = self.top_level.insert(var_name, decl);

        if res.is_some() {
            Err("struct of function already declared with the same name".to_owned())
        } else {
            Ok(())
        }
    }

    pub fn register_builtin_function(&mut self, function: BuiltinFunction, module_name: &str) {
        // for the builtin functions we don't care about the variable name
        let mut typed_args = Vec::with_capacity(function.args.len());

        for el in function.args.into_iter() {
            typed_args.push(TypedArg {
                arg_name: String::new(),
                arg_type: el,
            });
        }

        let builtin = FunctionDecl {
            args: typed_args,
            return_type: function.return_type,
            body: FunctionKind::Builtin(function.pointer),
        };

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
