use crate::{parser::TypedArg, symbol_type::SymbolType};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

/// A function declaration
#[derive(Clone, Debug)]
pub struct FunctionDecl {
    pub args: Vec<TypedArg>,
    pub return_type: Option<SymbolType>,
}

/// A struct declaration
#[derive(Clone)]
pub struct StructDecl {
    // in crocol struct fields are order dependant.
    // to guarentee that our map is sorted, use a BTreeMap
    pub fields: BTreeMap<String, SymbolType>,
}

impl fmt::Debug for StructDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StructDecl, fields: {:?}", self.fields)
    }
}

/// A top-level declaration such as a function declaration or a struct declaration
#[derive(Clone, Debug)]
pub enum Decl<U: Clone + fmt::Debug> {
    /// The blueprint of a function such as "fn a { .. }"
    FunctionDecl(FunctionDecl),

    /// The blueprint of a struct such as "struct A { .. }"
    StructDecl(StructDecl),

    /// A global variable
    GlobalVariable(U),
}

/// SymTable represents the symbol tables where all the variables are stored.
/// The SymTable is a generic struct that can accept any Symbol type, coming from any backend.
/// T is the type of a variable
#[derive(Clone, Default, Debug)]
pub struct SymTable<T: Clone + fmt::Debug> {
    /// All symbols stored in the SymTable.
    /// The Vec represents the different scopes of variables, introduced by BlockNodes
    /// The Hashmap stores variables by name, and bind them to a value.
    symbols: Vec<HashMap<String, T>>,
    /// Contains all struct and function declarations
    top_level: HashMap<String, Decl<T>>,
}

impl<T: Clone + fmt::Debug> SymTable<T> {
    pub fn new() -> Self {
        SymTable {
            symbols: vec![],
            top_level: HashMap::new(),
        }
    }

    /// Clears all symbols and returns them to restore them later
    pub fn pop_symbols(&mut self) -> Vec<HashMap<String, T>> {
        std::mem::replace(&mut self.symbols, vec![HashMap::new()])
    }

    /// Restores the symbols passed as argmuent and erases the current ones
    pub fn push_symbols(&mut self, symbols: Vec<HashMap<String, T>>) {
        self.symbols = symbols;
    }

    /// Returns the desired symbol starting from the inner scope, and ending with the global scope
    pub fn get_symbol<'a>(&'a self, var_name: &str) -> Result<&'a T, String> {
        for table in self.symbols.iter().rev() {
            if let Some(symbol) = table.get(var_name) {
                return Ok(symbol);
            }
        }

        if let Some(Decl::GlobalVariable(symbol)) = self.top_level.get(var_name) {
            return Ok(symbol);
        }

        // the variable doesn't exist
        Err(format!("variable {} has not been declared", var_name))
    }

    /// Returns the desired function declaration starting from the inner scope
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

    /// Returns the desired struct declaration starting from the inner scope
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

    /// Inserts to the closest scope if possible, or to the global scope
    pub fn insert_symbol(&mut self, name: &str, symbol: T) -> Result<(), String> {
        let var_already_declared;

        if let Some(scope) = self.symbols.last_mut() {
            var_already_declared = scope.insert(name.to_owned(), symbol).is_some();
        } else {
            var_already_declared = self
                .top_level
                .insert(name.to_owned(), Decl::GlobalVariable(symbol))
                .is_some();
        }

        if var_already_declared {
            Err(format!(
                "variable already declared with name {} in this scope",
                name
            ))
        } else {
            Ok(())
        }
    }

    /// Registers a function or struct declaration
    pub fn register_decl(&mut self, var_name: String, decl: Decl<T>) -> Result<(), String> {
        let res = self.top_level.insert(var_name, decl);

        if res.is_some() {
            Err("struct or function already declared with the same name".to_owned())
        } else {
            Ok(())
        }
    }

    pub fn add_scope(&mut self) {
        self.symbols.push(HashMap::new());
    }

    pub fn drop_scope(&mut self) {
        self.symbols.pop();
    }
}
