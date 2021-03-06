use crate::{
    ast::{BackendNode, NodeResult},
    crocoi::stdlib::BuiltinCallback,
    parser::TypedArg,
    symbol::FunctionDecl,
};
use crate::{error::CrocoError, token::CodePos};
use crate::{symbol::Decl, symbol::SymTable, token::Identifier};
use crate::{symbol_type::SymbolType, token::LiteralEnum};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::stdlib::{get_module, BuiltinFunction};

/// Struct representation in the crocoi backend
#[derive(Clone, Debug)]
pub struct Struct {
    // the fields, populated with values
    // we use an option because in declarations we don't want the overhead of an HashMap allocation
    pub fields: HashMap<String, Rc<RefCell<ISymbol>>>,

    // the corresponding type of the struct, as as StructDecl
    pub struct_type: String,
}

/// Function representation in the crocoi backend
#[derive(Clone)]
pub enum Function {
    Regular(Box<dyn BackendNode>),
    Builtin(BuiltinCallback),
}

/// Map representation in the crocoi backend
#[derive(Clone)]
pub struct Map {
    pub contents: HashMap<ISymbol, ISymbol>,
    pub key_type: Box<SymbolType>,
    pub value_type: Box<SymbolType>,
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map<{:?}, {:?}>", self.key_type, self.value_type)
    }
}

/// Array repressentation in the crocoi backend
#[derive(Clone, Debug)]
pub struct Array {
    pub contents: Vec<Rc<RefCell<ISymbol>>>,
    pub array_type: Box<SymbolType>,
}

#[derive(Clone, Debug)]
/// the symbol contents
pub enum ISymbol {
    /// a primitive such as 3 or false
    Primitive(LiteralEnum),

    /// an array such as [1, 2, 3]
    Array(Array),

    // a key-value map such as { "hello" => 5, "bonjour" => 4 }
    // Map(Map),

    // a pointer to a function such as let "a = b" where b is a FunctionDecl
    // Function(FunctionPointer),
    /// a symbol reference  
    Ref(Rc<RefCell<ISymbol>>),

    // a structure built from a StructDecl such as "let a = B {}"
    Struct(Struct),

    // a croco type such as [num]
    CrocoType(SymbolType),
}

impl ISymbol {
    /// force cast into a primitive
    pub fn into_primitive(self) -> Result<LiteralEnum, &'static str> {
        match self {
            ISymbol::Primitive(p) => Ok(p),
            _ => Err("expected a primitive"),
        }
    }

    /// force cast into a struct
    pub fn into_struct(self) -> Result<Struct, &'static str> {
        match self {
            ISymbol::Struct(s) => Ok(s),
            _ => Err("expected a struct"),
        }
    }

    /// force cast into an array
    pub fn into_array(self) -> Result<Array, &'static str> {
        match self {
            ISymbol::Array(a) => Ok(a),
            _ => Err("expected an array"),
        }
    }

    /// force cast into a symbol reference
    pub fn into_ref(self) -> Result<Rc<RefCell<ISymbol>>, &'static str> {
        match self {
            ISymbol::Ref(r) => Ok(r),
            _ => Err("expected a reference"),
        }
    }

    /// force cast into a croco type
    pub fn into_croco_type(self) -> Result<SymbolType, &'static str> {
        match self {
            ISymbol::CrocoType(t) => Ok(t),
            _ => Err("expected a type"),
        }
    }

    pub fn get_ref(&self) -> Rc<RefCell<ISymbol>> {
        match self {
            ISymbol::Ref(r) => r.clone(),
            _ => panic!("expected a reference"),
        }
    }
}

/// convenience type
pub type ISymTable = SymTable<Rc<RefCell<ISymbol>>>;

pub struct ICodegen {
    pub symtable: ISymTable,
    pub functions: HashMap<String, Function>,
}

impl ICodegen {
    /// Registers a built-in function
    pub fn register_builtin_function(&mut self, function: BuiltinFunction, module_name: &str) {
        // for the builtin functions we don't care about the variable name
        let typed_args = function
            .args
            .into_iter()
            .map(|arg| TypedArg {
                arg_name: String::new(),
                arg_type: arg,
            })
            .collect();

        let builtin = FunctionDecl {
            args: typed_args,
            return_type: function.return_type,
        };

        let namespaced_name =
            Identifier::new(function.name.clone(), module_name.to_owned()).get_namespaced_name();

        self.symtable
            .register_decl(namespaced_name, Decl::FunctionDecl(builtin))
            .unwrap();

        self.functions
            .insert(function.name, Function::Builtin(function.pointer));
    }
}

/// The result returned by a node.  
/// A symbol value is a ISymbol.  
/// A symbol in the symtable is RefCell'd so it can be mutated easely.
pub type INodeResult = NodeResult<ISymbol, Rc<RefCell<ISymbol>>>;

impl INodeResult {
    /// Clones a variable contents into a symbol, or return the symbol value directly
    pub fn into_symbol(self, code_pos: &CodePos) -> Result<ISymbol, CrocoError> {
        match self {
            INodeResult::Variable(var) => Ok(var.borrow().clone()),
            INodeResult::Value(val) => Ok(val),
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }

    /// Returns a reference of a value, or transforms a variable into a reference value if needed
    pub fn into_var_ref(self, code_pos: &CodePos) -> Result<ISymbol, CrocoError> {
        match self {
            INodeResult::Variable(var) => match &*var.borrow() {
                r @ ISymbol::Ref(_) => Ok(r.clone()),
                _ => Ok(ISymbol::Ref(var.clone())),
            },
            INodeResult::Value(val) => Ok(ISymbol::Ref(Rc::new(RefCell::new(val)))),
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }
}

/// returns the type of a symbol
pub fn get_symbol_type(symbol: &ISymbol) -> SymbolType {
    match symbol {
        ISymbol::Primitive(LiteralEnum::Bool(_)) => SymbolType::Bool,
        ISymbol::Primitive(LiteralEnum::Num(_)) => SymbolType::Num,
        ISymbol::Primitive(LiteralEnum::Fnum(_)) => SymbolType::Fnum,
        ISymbol::Primitive(LiteralEnum::Str(_)) => SymbolType::Str,
        ISymbol::Array(arr) => SymbolType::Array(arr.array_type.clone()),
        ISymbol::Struct(s) => SymbolType::Struct(s.struct_type.clone()),
        ISymbol::Ref(r) => SymbolType::Ref(Box::new(get_symbol_type(&*r.borrow()))),
        ISymbol::CrocoType(_) => SymbolType::CrocoType,
    }
}

/// imports a builtin module in the symtable
pub fn import_builtin_module(codegen: &mut ICodegen, name: &str) -> bool {
    if let Some(module) = get_module(name) {
        // the global module doesn't have any namespace
        let namespace = if name == "global" { "" } else { name };

        for function in module.functions {
            codegen.register_builtin_function(function, namespace);
        }

        for var in module.vars {
            let namespaced_name = Identifier::new(var.name, name.to_owned()).get_namespaced_name();
            codegen
                .symtable
                .register_decl(
                    namespaced_name,
                    Decl::GlobalVariable(Rc::new(RefCell::new(var.value))),
                )
                .unwrap();
        }
        true
    } else {
        false
    }
}
