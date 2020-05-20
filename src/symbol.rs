use std::collections::HashMap;

use crate::ast::AstNode;
use crate::parser::TypedArg;
use crate::token::{literal_eq, LiteralEnum};

/// A callback to a built-in function
pub type BuiltinCallback = fn(Vec<LiteralEnum>) -> Result<LiteralEnum, String>;

/// Either if the function is a classic function or a built-in function
#[derive(Debug, Clone)]
pub enum FunctionKind {
    Regular(AstNode),
    Builtin(BuiltinCallback),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub args: Vec<TypedArg>,
    pub body: FunctionKind,
    pub return_type: LiteralEnum,
}

impl<'a> FunctionCall {
    pub fn new(args: Vec<TypedArg>, return_type: LiteralEnum, body: FunctionKind) -> Self {
        FunctionCall {
            args,
            return_type,
            body,
        }
    }
}

/// a Symbol in the symbol table. Could be either a Literal or a function call
#[derive(Debug, Clone)]
pub enum Symbol {
    Literal(LiteralEnum),
    Function(FunctionCall),
}

/// SymTable represents symbol tables where all the variables are stored.
/// The Vec represents the different scopes of variables, introduced by BlockNodes
/// The Hashmap stores variables by name, and bind them to a value.
#[derive(Clone, Default, Debug)]
pub struct SymTable(Vec<HashMap<String, Symbol>>);

impl SymTable {
    pub fn new() -> Self {
        SymTable(vec![HashMap::new()])
    }

    /// returns wether or not a variable with the same name exists on this scope
    pub fn same_scope_symbol(&self, var_name: &str) -> Result<bool, String> {
        Ok(self.0.last().unwrap().get(var_name).is_some())
    }

    // return the variable / function value, starting from the inner scope
    pub fn get_symbol(&self, var_name: &str) -> Result<Symbol, String> {
        for table in self.0.iter().rev() {
            if let Some(symbol_ref) = table.get(var_name) {
                return Ok(symbol_ref.clone());
            }
        }

        // the variable doesn't exist
        Err(format!("variable {} has not been declared", var_name))
    }

    /// modify a variable already present in the symbol table
    pub fn modify_symbol(&mut self, var_name: &str, var_value: LiteralEnum) -> Result<(), String> {
        for table in self.0.iter_mut().rev() {
            if let Some(old_symbol) = table.get_mut(var_name) {
                match old_symbol {
                    Symbol::Function(_) => {
                        return Err(format!(
                            "Cannot change the {} function to a variable",
                            var_name
                        ))
                    }
                    Symbol::Literal(old_var_value) => {
                        if !literal_eq(old_var_value, &var_value) {
                            return Err(format!(
                                "Cannot assign another type to the variable {}",
                                var_name
                            ));
                        } else {
                            // update the value
                            *old_var_value = var_value;
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(format!("Can't assign to undeclared variable {}", var_name))
    }

    /// insert to the closest scope
    pub fn insert_symbol(&mut self, var_name: &str, var_value: LiteralEnum) -> Result<(), String> {
        self.0
            .last_mut()
            .unwrap()
            .insert(var_name.to_owned(), Symbol::Literal(var_value));

        Ok(())
    }

    pub fn register_fn(&mut self, fn_name: &str, fn_symbol: Symbol) -> Result<(), String> {
        let fn_pointer = match fn_symbol {
            Symbol::Function(fn_pointer) => fn_pointer,
            _ => return Err("expected function but got a variable".to_owned()),
        };
        self.0
            .first_mut()
            .unwrap()
            .insert(fn_name.to_owned(), Symbol::Function(fn_pointer));
        Ok(())
    }

    pub fn add_scope(&mut self) -> Result<(), String> {
        self.0.push(HashMap::new());
        Ok(())
    }

    pub fn drop_scope(&mut self) -> Result<(), String> {
        self.0.pop();
        Ok(())
    }
}
