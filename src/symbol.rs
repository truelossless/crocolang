use crate::parser::TypedArg;
use crate::{ast::AstNode, crocoi::ISymbol};
use crate::{
    builtin::{get_module, BuiltinCallback, BuiltinFunction},
    symbol_type::type_eq,
};
use crate::{
    crocoi::symbol::SymbolContent,
    symbol_type::{FunctionType, SymbolType},
    token::{Identifier, LiteralEnum},
};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
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
    pub body: Option<FunctionKind>,
    pub return_type: SymbolType,
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

/// the abstract trait representing a symbol on any backend.
// implementations among backends are prefixed with the backend name,
// such as LSymbol or ISymbol. Tecnhically we could just use namespaces,
// but it's both easier and shorter like that.
pub trait Symbol {
    /// returns the type of a Symbol
    fn to_type(&self) -> SymbolType;
}

/// a top-level declaration such as a function declaration or a struct declaration
#[derive(Clone, Debug)]
pub enum Decl {
    // the blueprint of a function such as "fn a { .. }"
    FunctionDecl(FunctionDecl),

    // the blueprint of a struct such as "struct A { .. }"
    StructDecl(StructDecl),
}

/// returns the type of a symbol
pub fn get_symbol_type(symbol: &SymbolContent) -> SymbolType {
    match symbol {
        SymbolContent::Primitive(LiteralEnum::Bool(_)) => SymbolType::Bool,
        SymbolContent::Primitive(LiteralEnum::Num(_)) => SymbolType::Num,
        SymbolContent::Primitive(LiteralEnum::Str(_)) => SymbolType::Str,
        SymbolContent::Primitive(LiteralEnum::Void) => SymbolType::Void,
        SymbolContent::Array(arr) => SymbolType::Array(arr.array_type),
        SymbolContent::Struct(s) => SymbolType::Struct(s.struct_type),
        SymbolContent::Ref(r) => SymbolType::Ref(Box::new(get_symbol_type(&*r.borrow()))),
        SymbolContent::Function(func) => SymbolType::Function(FunctionType {
            args: func.args.clone(),
            return_type: Box::new(func.return_type.clone()),
        }),
        SymbolContent::CrocoType(_) => SymbolType::CrocoType,
    }
}

/// SymTable represents the symbol tables where all the variables are stored.
/// The Vec represents the different scopes of variables, introduced by BlockNodes
/// The Hashmap stores variables by name, and bind them to a value.
/// Top level contains all struct and function declarations.

/// The SymTable is a generic struct that can accept any Symbol type, coming from any backend.
#[derive(Clone, Default, Debug)]
pub struct SymTable<T: Symbol + Clone> {
    symbols: Vec<HashMap<String, T>>,
    top_level: HashMap<String, Decl>,
}

impl<T: Symbol + Clone> SymTable<T> {
    pub fn new() -> Self {
        SymTable {
            symbols: vec![HashMap::new()],
            top_level: HashMap::new(),
        }
    }

    /// return the desired symbol starting from the inner scope
    pub fn get_symbol(&mut self, var_name: &str) -> Result<T, &'static str> {
        for table in self.symbols.iter().rev() {
            if let Some(symbol) = table.get(var_name) {
                return Ok(symbol.clone());
            }
        }

        // the variable doesn't exist
        Err(&format!("variable {} has not been declared", var_name))
    }

    /// return the desired function declaration starting from the inner scope
    pub fn get_function_decl(&mut self, fn_name: &str) -> Result<&mut FunctionDecl, &'static str> {
        match self.top_level.get_mut(fn_name) {
            Some(Decl::FunctionDecl(ref mut function)) => return Ok(function),
            Some(_) => {
                return Err(&format!(
                    "trying to get {} as a function but it's not",
                    fn_name
                ))
            }
            None => (),
        }

        // the function doesn't exist
        Err(&format!("function {} has not been declared", fn_name))
    }

    /// return the desired struct declaration starting from the inner scope
    pub fn get_struct_decl(&mut self, struct_type: &str) -> Result<&StructDecl, &'static str> {
        match self.top_level.get(struct_type) {
            Some(Decl::StructDecl(ref struct_decl)) => return Ok(struct_decl),
            Some(_) => {
                return Err(&format!(
                    "trying to get {} as a struct but it's not",
                    struct_type
                ))
            }
            None => (),
        }

        // the function doesn't exist
        Err(&format!("struct {} has not been declared", struct_type))
    }

    /// insert to the closest scope
    pub fn insert_symbol(&mut self, name: &str, symbol: T) -> Result<(), &'static str> {
        if self
            .symbols
            .last_mut()
            .unwrap()
            .insert(name.to_owned(), symbol)
            .is_none()
        {
            Ok(())
        } else {
            Err(&format!(
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

    /// modify a symbol already present in the symbol table
    pub fn modify_symbol(&mut self, name: &str, symbol: T) -> Result<(), String> {
        for table in self.symbols.iter_mut().rev() {
            if let Some(old_symbol) = table.get_mut(name) {
                let old_symbol_type = old_symbol.to_type();
                let new_symbol_type = symbol.to_type();

                if type_eq(&old_symbol_type, &new_symbol_type) {
                    *old_symbol = symbol;
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

    pub fn import_builtin_module(symtable: &mut SymTable<ISymbol>, name: &str) -> bool {
        if let Some(module) = get_module(name) {
            // the global module doesn't have any namespace
            let namespace = if name == "global" { "" } else { name };

            for function in module.functions {
                symtable.register_builtin_function(function, namespace);
            }

            for var in module.vars {
                let namespaced_name =
                    Identifier::new(var.name, name.to_owned()).get_namespaced_name();
                symtable
                    .insert_global_symbol(namespaced_name, Rc::new(RefCell::new(var.value)))
                    .unwrap();
            }
            true
        } else {
            false
        }
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

        let builtin = FunctionDecl {
            args: typed_args,
            return_type: function.return_type,
            body: Some(FunctionKind::Builtin(function.pointer)),
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
