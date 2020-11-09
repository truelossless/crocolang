use crate::{ast::AstNode, builtin::get_module, symbol::SymTable, token::Identifier};
use crate::{ast::NodeResult, builtin::BuiltinCallback};
use crate::{error::CrocoError, parser::TypedArg, token::CodePos};
use crate::{
    symbol_type::{FunctionType, SymbolType},
    token::LiteralEnum,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::rc::Rc;
// Either the function is a classic function or a built-in function
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

/// representation of a built struct
#[derive(Clone, Debug)]
pub struct Struct {
    // the fields, populated with values
    // we use an option because in declarations we don't want the overhead of an HashMap allocation
    pub fields: HashMap<String, Rc<RefCell<ISymbol>>>,

    // the corresponding type of the struct, as as StructDecl
    pub struct_type: String,
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

    // a function, local to this scope (will be useful for struct methods)
    Function(Box<FunctionDecl>),

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

    pub fn mut_array(&mut self) -> Result<&mut Array, &'static str> {
        match self {
            ISymbol::Array(a) => Ok(a),
            _ => Err("expected an array"),
        }
    }

    /// force cast into a function
    pub fn into_function(self) -> Result<Box<FunctionDecl>, &'static str> {
        match self {
            ISymbol::Function(r) => Ok(r),
            _ => Err("expected a function declaration"),
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
}

/// convenience type
pub type ISymTable = SymTable<Rc<RefCell<ISymbol>>>;

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
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)) 
        }
    }

    /// Returns the ISymbol behind a Value, or a transforms a Variable into a Value(ISymbol::Ref())
    pub fn into_value_or_var_ref(self, code_pos: &CodePos) -> Result<ISymbol, CrocoError> {
        match self {
            INodeResult::Variable(var) => Ok(ISymbol::Ref(var)),
            INodeResult::Value(val) => Ok(val),
            _ => Err(CrocoError::expected_value_got_early_return_error(code_pos)),
        }
    }

    /// transforms a Variable to a Value(ISymbol::Ref())
    pub fn into_var_ref(self, code_pos: &CodePos) -> Result<ISymbol, CrocoError> {
        match self {
            INodeResult::Variable(var) => Ok(ISymbol::Ref(var)),
            _ => Err(CrocoError::new(
                code_pos,
                "expected a reference to a variable",
            )),
        }
    }
}

/// returns the type of a symbol
pub fn get_symbol_type(symbol: &ISymbol) -> SymbolType {
    match symbol {
        ISymbol::Primitive(LiteralEnum::Bool(_)) => SymbolType::Bool,
        ISymbol::Primitive(LiteralEnum::Num(_)) => SymbolType::Num,
        ISymbol::Primitive(LiteralEnum::Str(_)) => SymbolType::Str,
        ISymbol::Array(arr) => SymbolType::Array(arr.array_type.clone()),
        ISymbol::Struct(s) => SymbolType::Struct(s.struct_type.clone()),
        ISymbol::Ref(r) => SymbolType::Ref(Box::new(get_symbol_type(&*r.borrow()))),
        ISymbol::Function(func) => SymbolType::Function(FunctionType {
            args: func.args.clone(),
            return_type: Box::new(func.return_type.clone()),
        }),
        ISymbol::CrocoType(_) => SymbolType::CrocoType,
    }
}

/// imports a builtin module in the symtable
pub fn import_builtin_module(symtable: &mut ISymTable, name: &str) -> bool {
    if let Some(module) = get_module(name) {
        // the global module doesn't have any namespace
        let namespace = if name == "global" { "" } else { name };

        for function in module.functions {
            symtable.register_builtin_function(function, namespace);
        }

        for var in module.vars {
            let namespaced_name = Identifier::new(var.name, name.to_owned()).get_namespaced_name();
            symtable
                .insert_global_symbol(namespaced_name, Rc::new(RefCell::new(var.value)))
                .unwrap();
        }
        true
    } else {
        false
    }
}
