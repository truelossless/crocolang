use crate::builtin::BuiltinCallback;
use crate::parser::TypedArg;
use crate::{ast::AstNode, builtin::get_module, symbol::SymTable, token::Identifier};
use crate::{
    error::CrocoError,
    symbol::Symbol,
    symbol_type::{FunctionType, SymbolType},
    token::{CodePos, LiteralEnum},
};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::rc::Rc;

//// The type of value returned by a node
// TODO: in the long run we could probably merge this with LNodeResult
#[derive(Clone)]
pub enum INodeResult {
    /// a break statement
    Break,
    /// a continue statement
    Continue,
    /// a return statement
    /// e.g return 3
    Return(ISymbol),
    /// a symbol
    /// e.g a struct or a primitive
    Symbol(ISymbol),
    // empty value
}

impl INodeResult {
    /// convenience function to build a INodeResult
    pub fn construct_symbol(symbol_content: SymbolContent) -> INodeResult {
        INodeResult::Symbol(Rc::new(RefCell::new(symbol_content)))
    }

    pub fn into_symbol(self, pos: &CodePos) -> Result<ISymbol, CrocoError> {
        match self {
            INodeResult::Symbol(s) => Ok(s),
            _ => Err(CrocoError::new(
                pos,
                "Expected a value but got an early-return keyword",
            )),
        }
    }

    pub fn into_return(self) -> Result<ISymbol, CrocoError> {
        match self {
            INodeResult::Return(s) => Ok(s),
            _ => panic!("Expected a return value but got an early-return keyword !!"),
        }
    }
}

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
    pub fields: Option<HashMap<String, ISymbol>>,

    // the corresponding type of the struct, as as StructDecl
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
    pub contents: Option<Vec<ISymbol>>,
    pub array_type: Box<SymbolType>,
}

/// a symbol in the symbol table. It could be either a primitive, a function, a struct ...
// I've tried multiple times to use references with lifetimes annotations but it's too hard in graph structures
// I had to annotate all the nodes and structs with lifetimes and it didn't even work :S
// this is why I'm using Rc as it eases the process a lot.
// If you have an idea for a more elegant implementation I'm all ears !
pub type ISymbol = Rc<RefCell<SymbolContent>>;

impl Symbol for ISymbol {
    fn to_type(&self) -> SymbolType {
        get_symbol_type(&*self.borrow())
    }
}

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
    // Function(FunctionPointer),

    // a function, local to this scope (will be useful for struct methods)
    Function(Box<FunctionDecl>),

    /// a symbol reference  
    Ref(ISymbol),

    // a structure built from a StructDecl such as "let a = B {}"
    Struct(Struct),

    // a croco type such as [num]
    CrocoType(SymbolType),
}

impl SymbolContent {
    /// force cast into a primitive
    pub fn into_primitive(self) -> Result<LiteralEnum, &'static str> {
        match self {
            SymbolContent::Primitive(p) => Ok(p),
            _ => Err("expected a primitive"),
        }
    }

    // force cast into a struct
    pub fn into_struct(self) -> Result<Struct, &'static str> {
        match self {
            SymbolContent::Struct(s) => Ok(s),
            _ => Err("expected a struct"),
        }
    }

    // force cast into an array
    pub fn into_array(self) -> Result<Array, &'static str> {
        match self {
            SymbolContent::Array(a) => Ok(a),
            _ => Err("expected an array"),
        }
    }

    /// force cast into a function
    pub fn into_function(self) -> Result<Box<FunctionDecl>, &'static str> {
        match self {
            SymbolContent::Function(r) => Ok(r),
            _ => Err("expected a function declaration"),
        }
    }

    /// dereferences a symbol
    pub fn into_ref(self) -> Result<ISymbol, &'static str> {
        match self {
            SymbolContent::Ref(r) => Ok(r),
            _ => Err("expected a reference"),
        }
    }

    /// force cast into a croco type
    pub fn into_croco_type(self) -> Result<SymbolType, &'static str> {
        match self {
            SymbolContent::CrocoType(t) => Ok(t),
            _ => Err("expected a type"),
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

/// imports a builtin module in the symtable
pub fn import_builtin_module(symtable: &mut SymTable<ISymbol>, name: &str) -> bool {
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
