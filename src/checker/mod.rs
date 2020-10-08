use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::NodeResult, error::CrocoError, symbol::SymTable, symbol_type::SymbolType, token::CodePos,
};

/// The checker ensures for compiled backends that the croco code is valid.  
/// It is run before the backend, so the backend doesn't have to verify most of
/// the errors and can just unwrap() things.
/// It is however not suitable for interpreted backends as it can slow down execution.  
/// Particularly, type inferring can be much more accurate with compiled backends
/// as we "don't care" for compile time performance.

/// a symbol in the checker
#[derive(Clone)]
pub struct CheckerSymbol {
    /// the type of the symbol, None if it's unknown
    pub symbol_type: Option<SymbolType>,
    /// if it's a variable tracked store its name (used for errors pretty-print)
    pub var: Option<String>,
    /// if the symbol is ever read
    pub is_used: bool,
}

/// The result returned by a node.  
/// A symbol value is a CheckerSymbol.  
/// A symbol in the symtable is RefCell'd so it can be mutated easely.
pub type _CNodeResult = NodeResult<CheckerSymbol, Rc<RefCell<CheckerSymbol>>>;

impl CheckerSymbol {
    /// Tracks a symbol value
    pub fn new_value(symbol_type: SymbolType) -> Self {
        CheckerSymbol {
            var: None,
            symbol_type: Some(symbol_type),
            is_used: false,
        }
    }

    /// Tracks a symbol value with an unknown type
    pub fn new_unknown_value() -> Self {
        CheckerSymbol {
            var: None,
            symbol_type: None,
            is_used: false,
        }
    }

    /// Tracks a symbol variable
    pub fn new_variable(symbol_type: SymbolType, var_name: String) -> Self {
        CheckerSymbol {
            var: Some(var_name),
            symbol_type: Some(symbol_type),
            is_used: false,
        }
    }

    /// Tracks a symbol variable with an unknwon type
    pub fn new_unknown_variable(var_name: String) -> Self {
        CheckerSymbol {
            var: Some(var_name),
            symbol_type: None,
            is_used: false,
        }
    }

    pub fn into_value(self, code_pos: &CodePos) -> Result<SymbolType, CrocoError> {
        match self.symbol_type {
            Some(s) => Ok(s),
            None => Err(CrocoError::new(
                code_pos,
                format!(
                    "cannot infer a variable type for {}",
                    self.var.unwrap()
                ),
            )),
        }
    }

    pub fn as_symbol(&self) -> &SymbolType {
        match &self.symbol_type {
            Some(s) => s,
            None => panic!(),
        }
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        self.symbol_type.is_none()
    }

    #[inline]
    pub fn is_var(&self) -> bool {
        self.var.is_some()
    }

    /// set the variable as used in the symtable
    pub fn set_used(
        &mut self,
        _checker: &mut Checker,
        _code_pos: &CodePos,
    ) -> Result<(), CrocoError> {
        self.is_used = true;

        // TODO fix set_used
        // if let Some(var_name) = &self.var_name {
        //     checker
        //         .symtable
        //         .modify_symbol(&var_name, self.clone())
        //         .map_err(|e| CrocoError::new(code_pos, e))?;
        // }

        Ok(())
    }
}

pub struct Checker {
    // the symtable is just composed of the symbol type, we don't need
    // to keep track of the values.
    pub symtable: SymTable<Rc<RefCell<CheckerSymbol>>>,

    // collect recoverable warnings
    // this is too much pain to do the same with errors, though
    pub warnings: Vec<CrocoError>,
}
