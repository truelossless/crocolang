use crate::parser::TypedArg;
use dyn_clone::DynClone;
use std::collections::HashMap;
use std::fmt;

use crate::interpreter::SymbolTable;
use crate::token::LiteralEnum;

#[derive(Clone)]
pub enum AstNode {
    LeafNode(Box<dyn LeafNodeTrait>),
    UnaryNode(Box<dyn UnaryNodeTrait>),
    BinaryNode(Box<dyn BinaryNodeTrait>),
    NaryNode(Box<dyn NaryNodeTrait>),
}

impl AstNode {
    pub fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        // yeah, it's big brain time !
        match self {
            AstNode::LeafNode(node) => node.visit(symbol_tables),
            AstNode::UnaryNode(node) => node.visit(symbol_tables),
            AstNode::BinaryNode(node) => node.visit(symbol_tables),
            AstNode::NaryNode(node) => node.visit(symbol_tables),
        }
    }
}

impl fmt::Debug for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = match &self {
            AstNode::LeafNode(_) => "LeafNode",
            AstNode::UnaryNode(_) => "UnaryNode",
            AstNode::BinaryNode(_) => "BinaryNode",
            AstNode::NaryNode(_) => "NaryNode",
        };

        write!(f, "AstNode {{ type: {} }}", ret)
    }
}

pub type BuiltinCallback = fn(Vec<LiteralEnum>) -> Result<Option<LiteralEnum>, String>;

/// Either if the function is a classic function or a built-in function
#[derive(Debug, Clone)]
pub enum FunctionKind {
    Regular(AstNode),
    Builtin(BuiltinCallback),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    args: Vec<TypedArg>,
    body: FunctionKind,
    return_type: LiteralEnum,
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

pub trait LeafNodeTrait: DynClone {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String>;
}
dyn_clone::clone_trait_object!(LeafNodeTrait);

pub trait UnaryNodeTrait: DynClone {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String>;
    fn set_bottom(&mut self, node: AstNode);
}
dyn_clone::clone_trait_object!(UnaryNodeTrait);

pub trait BinaryNodeTrait: DynClone {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String>;
    fn set_left(&mut self, node: AstNode);
    fn set_right(&mut self, node: AstNode);
}
dyn_clone::clone_trait_object!(BinaryNodeTrait);

pub trait NaryNodeTrait: DynClone {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String>;
    fn prepend_child(&mut self, node: AstNode);
    fn add_child(&mut self, node: AstNode);
}
dyn_clone::clone_trait_object!(NaryNodeTrait);

// TODO: move to SymbolTable.rs
// return the variable / function value, starting from the inner scope
fn symbol_tables_get(symbol_tables: &SymbolTable, var_name: &str) -> Result<Symbol, String> {
    let symtables_unlocked = symbol_tables.write().expect("Write lock already in use !");

    for table in symtables_unlocked.iter().rev() {
        if let Some(symbol_ref) = table.get(var_name) {
            return Ok(symbol_ref.clone());
        }
    }

    // the variable doesn't exist
    Err(format!("variable {} has not been declared", var_name))
}

/// modify a variable already present in the symbol table
fn symbol_tables_modify(
    symbol_tables: SymbolTable,
    var_name: &str,
    var_value: LiteralEnum,
) -> Result<(), String> {
    let mut symtables_unlocked = symbol_tables
        .write()
        .expect("The symbol tables have a write lock !");
    for table in symtables_unlocked.iter_mut().rev() {
        if let Some(old_symbol) = table.get_mut(var_name) {
            match old_symbol {
                Symbol::Function(_) => {
                    return Err(format!(
                        "Cannot change the {} function to a variable",
                        var_name
                    ))
                }
                Symbol::Literal(old_var_value) => {
                    if !variant_eq(old_var_value, &var_value) {
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
fn symbol_tables_insert(symbol_tables: SymbolTable, var_name: &str, var_value: LiteralEnum) {
    let mut symtables_unlocked = symbol_tables.write().expect("Write lock already in use !");

    // get the closest scope (which have to exist)
    symtables_unlocked
        .last_mut()
        .unwrap()
        .insert(var_name.to_owned(), Symbol::Literal(var_value));
}

fn symbol_tables_register_fn(symbol_tables: SymbolTable, fn_name: &str, fn_pointer: FunctionCall) {
    let mut symtables_unlocked = symbol_tables.write().expect("Write lock already in use !");
    symtables_unlocked
        .last_mut()
        .unwrap()
        .insert(fn_name.to_owned(), Symbol::Function(fn_pointer));
}

/// returns the LiteralEnum associated to a node
fn get_value(
    opt_node: &Option<AstNode>,
    symbol_tables: SymbolTable,
) -> Result<LiteralEnum, String> {
    match opt_node {
        Some(node) => match node.visit(symbol_tables)? {
            Some(x) => Ok(x),
            None => panic!("should have got a value there !!"),
        },
        None => Err("One variable hasn't been initialized !".to_string()),
    }
}

/// returns the number value of a node
fn get_number_value(opt_node: &Option<AstNode>, symbol_tables: SymbolTable) -> Result<f32, String> {
    match opt_node {
        Some(node) => match node.visit(symbol_tables)? {
            Some(LiteralEnum::Number(x)) => Ok(x.unwrap()),
            _ => Err("Performing a math operation on a wrong variable type !".to_string()),
        },
        None => panic!("Node not initialized !"),
    }
}

// https://stackoverflow.com/questions/32554285/compare-enums-only-by-variant-not-value
#[allow(clippy::mem_discriminant_non_enum)]
fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[derive(Clone)]
pub struct FunctionCallNode {
    fn_name: String,
    fn_args: Vec<AstNode>,
}

impl FunctionCallNode {
    pub fn new(fn_name: String, fn_args: Vec<AstNode>) -> Self {
        FunctionCallNode { fn_name, fn_args }
    }
}

// TODO: return fn value
impl NaryNodeTrait for FunctionCallNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let fn_symbol = symbol_tables_get(&symbol_tables, &self.fn_name)?;

        // ensure that we're calling a function
        let fn_decl = match fn_symbol {
            Symbol::Literal(_) => {
                return Err(format!(
                    "Can't call {} as it's a variable and not a function",
                    &self.fn_name
                ))
            }
            Symbol::Function(func) => func,
        };

        // resolve the function arguments
        let mut visited_args = Vec::new();
        for arg in self.fn_args.iter() {
            let value = match arg.visit(symbol_tables.clone())? {
                None => {
                    return Err(format!(
                        "Empty value in {} function parameter",
                        self.fn_name
                    ))
                }
                Some(value) => value,
            };

            visited_args.push(value);
        }

        // ensure that the arguments provided and the arguments in the function call match
        if visited_args.len() != fn_decl.args.len() {
            return Err(format!(
                "mismatched number of arguments in function {}\n expected {} parameters but got {}",
                self.fn_name,
                fn_decl.args.len(),
                visited_args.len()
            ));
        }

        for (i, arg) in visited_args.iter().enumerate() {
            if !variant_eq(arg, &fn_decl.args[i].arg_type) {
                return Err(format!(
                    "parameter {} type doesn't match {} function definition",
                    i + 1,
                    self.fn_name
                ));
            }
        }

        match fn_decl.body {
            
            FunctionKind::Regular(func_call) => {
                // get the block node of the function
                let mut block_node = match func_call {
                    AstNode::NaryNode(mut node) => node,
                    _ => unreachable!(),
                };

                // inject the function arguments
                for (i, arg) in visited_args.into_iter().enumerate() {
                    let resolved_literal = AstNode::LeafNode(Box::new(LiteralNode::new(arg)));

                    block_node.prepend_child(AstNode::BinaryNode(Box::new(DeclNode::new(
                        fn_decl.args[i].arg_name.clone(),
                        resolved_literal,
                        Some(fn_decl.args[i].arg_type.clone()),
                    ))))
                }

                block_node.visit(symbol_tables)?;
            }

            FunctionKind::Builtin(builtin_call) => {
                let callback = builtin_call;
                callback(visited_args)?;
            }
        }

        Ok(None)
    }

    fn prepend_child(&mut self, node: AstNode) {
        self.fn_args.insert(0, node);
    }

    fn add_child(&mut self, node: AstNode) {
        self.fn_args.push(node);
    }
}

/// function declaration node
#[derive(Clone)]
pub struct FunctionDeclNode {
    name: String,
    return_type: LiteralEnum,
    args: Vec<TypedArg>,
    body: AstNode,
}

impl FunctionDeclNode {
    pub fn new(name: String, return_type: LiteralEnum, args: Vec<TypedArg>) -> Self {
        FunctionDeclNode {
            name,
            return_type,
            args,
            body: AstNode::NaryNode(Box::new(BlockNode::new())),
        }
    }
}

impl UnaryNodeTrait for FunctionDeclNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        // check if the function has aready been defined
        if symbol_tables_get(&symbol_tables, &self.name).is_ok() {
            return Err(format!("{} function name already used", self.name));
        }

        // TODO: move out (&mut visit ?)
        // move out the body node
        // let body = std::mem::replace(&self.body, AstNode::LeafNode(Box::new(VoidNode::new())));

        let fn_call = FunctionCall::new(
            self.args.clone(),
            self.return_type.clone(),
            // TODO: figure out how to limit the clone() calls for each function call
            FunctionKind::Regular(self.body.clone()),
        );

        symbol_tables_register_fn(symbol_tables, &self.name, fn_call);
        Ok(None)
    }

    fn set_bottom(&mut self, node: AstNode) {
        self.body = node;
    }
}

// node containing multiple instructions
// creates a new scope
// e.g: if body, function body, etc.
#[derive(Clone)]
pub struct BlockNode {
    body: Vec<AstNode>,
}

impl BlockNode {
    pub fn new() -> Self {
        BlockNode { body: Vec::new() }
    }
}

impl NaryNodeTrait for BlockNode {
    fn prepend_child(&mut self, node: AstNode) {
        self.body.insert(0, node);
    }

    fn add_child(&mut self, node: AstNode) {
        self.body.push(node);
    }

    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        // push a new scope
        // SEUM
        let mut symtables_unlocked = symbol_tables.write().expect("Write lock already in use !");
        symtables_unlocked.push(HashMap::new());
        drop(symtables_unlocked);

        for node in &self.body {
            node.visit(symbol_tables.clone())?;
        }
        // we're done with this scope, pop it
        let mut symtables_unlocked = symbol_tables.write().expect("Write lock already in use !");
        symtables_unlocked.pop();

        Ok(None)
    }
}

// a node to declare a new variable (declared variable are initialized by default)
#[derive(Clone)]
pub struct DeclNode {
    // the var_name
    left: String,
    // the variable Assignement
    right: AstNode,
    // the type of the variable
    var_type: Option<LiteralEnum>,
}

impl DeclNode {
    pub fn new(var_name: String, expr: AstNode, var_type: Option<LiteralEnum>) -> Self {
        DeclNode {
            left: var_name,
            right: expr,
            var_type,
        }
    }
}

impl BinaryNodeTrait for DeclNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        if symbol_tables_get(&symbol_tables, &self.left).is_ok() {
            return Err(format!(
                "The variable {} has already been declared",
                self.left
            ));
        }

        let var_value = self.right.visit(symbol_tables.clone())?;

        if self.var_type.is_some() && !variant_eq(&var_value, &self.var_type) {
            return Err(format!(
                "variable {} has been explicitely given a type but is declared with another one",
                &self.left
            ));
        }

        // this should not happen as it's already checked by the parser.
        // TODO: check edge cases and eventually remove.
        if var_value.is_none() && self.var_type.is_none() {
            return Err(format!("Unable to infer the type of {}", self.left));
        }

        symbol_tables_insert(symbol_tables, &self.left, var_value.unwrap());
        Ok(None)
    }

    fn set_left(&mut self, node: AstNode) {}
    fn set_right(&mut self, node: AstNode) {}
}

// a node to assign a variable to a certain value
#[derive(Clone)]
pub struct AssignmentNode {
    // variable to assign to
    left: String,
    // expr assigned
    right: AstNode,
}

// TODO: check if variable isn't assigned multiple times
// check if variable is of the good type
impl AssignmentNode {
    pub fn new(var_name: String, expr: AstNode) -> Self {
        AssignmentNode {
            left: var_name,
            right: expr,
        }
    }
}

impl BinaryNodeTrait for AssignmentNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let right_val = self.right.visit(symbol_tables.clone())?;

        match right_val {
            Some(val) => {
                symbol_tables_modify(symbol_tables, &self.left, val)?;
                Ok(None)
            }
            None => panic!("Cannot assign a variable to a void expression"),
        }
    }

    fn set_left(&mut self, node: AstNode) {
        panic!("Can't set left val for Assignment Node !")
    }
    fn set_right(&mut self, node: AstNode) {}
}

// a node holding a variable
#[derive(Clone)]
pub struct VarNode {
    name: String,
}

impl VarNode {
    pub fn new(name: String) -> Self {
        VarNode { name }
    }
}

impl LeafNodeTrait for VarNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let var = symbol_tables_get(&symbol_tables, &self.name)?;
        let value = match var {
            Symbol::Function(_) => {
                return Err(format!(
                    "Trying to use {} as a variable but it's a function",
                    self.name
                ))
            }
            Symbol::Literal(lit) => lit,
        };
        Ok(Some(value))
    }
}

// a node ... doing nothing. THE VOID.
// It's separated from a literal node to avoid the billion dollar mistake.
// #[derive(Clone)]
// pub struct VoidNode {}

// impl VoidNode {
//     pub fn new() -> Self {
//         VoidNode {}
//     }
// }

// impl LeafNodeTrait for VoidNode {
//     fn visit(&self, _symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
//         Ok(None)
//     }
// }

// a node holding a literal
#[derive(Clone)]
pub struct LiteralNode {
    value: LiteralEnum,
}

impl LiteralNode {
    pub fn new(value: LiteralEnum) -> Self {
        LiteralNode { value }
    }
}

impl LeafNodeTrait for LiteralNode {
    fn visit(&self, _symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        Ok(Some(self.value.clone()))
    }
}

#[derive(Clone)]
pub struct PlusNode {
    left: Option<AstNode>,
    right: Option<AstNode>,
}

impl PlusNode {
    pub fn new() -> Self {
        PlusNode {
            left: None,
            right: None,
        }
    }
}

/// node handling additions and concatenations
impl BinaryNodeTrait for PlusNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let left_val = get_value(&self.left, symbol_tables.clone())?;
        let right_val = get_value(&self.right, symbol_tables)?;

        // different kinds of additions can happen
        // the PlusNode also works for concatenation.

        let txt_and_txt = |txt1: Option<String>, txt2: Option<String>| -> LiteralEnum {
            let mut left_str = txt1.unwrap();
            let right_str = txt2.unwrap();
            left_str.push_str(&right_str);
            LiteralEnum::Text(Some(left_str))
        };

        let txt_and_num =
            |txt: Option<String>, num: Option<f32>, number_first: bool| -> LiteralEnum {
                let mut txt_str = txt.unwrap();
                let mut num_str = num.unwrap().to_string();

                if number_first {
                    num_str.push_str(&txt_str);
                } else {
                    txt_str.push_str(&num_str);
                }

                LiteralEnum::Text(Some(txt_str))
            };

        let num_and_num = |num1: Option<f32>, num2: Option<f32>| -> LiteralEnum {
            let num1_val = num1.unwrap();
            let num2_val = num2.unwrap();

            LiteralEnum::Number(Some(num1_val + num2_val))
        };

        match left_val {
            LiteralEnum::Text(txt1) => match right_val {
                LiteralEnum::Text(txt2) => {
                    // self.value = txt_and_txt(txt1.clone(), txt2.clone());
                    Ok(Some(txt_and_txt(txt1, txt2)))
                }

                LiteralEnum::Number(num) => {
                    // self.value = txt_and_num(txt1.clone(), num, false);
                    Ok(Some(txt_and_num(txt1, num, false)))
                }

                LiteralEnum::Boolean(_) => Err("cannot add booleans".to_string()),
                LiteralEnum::Void => unreachable!(),
            },

            LiteralEnum::Number(num1) => match right_val {
                LiteralEnum::Text(txt) => {
                    // self.value = txt_and_num(txt.clone(), num1, true);
                    Ok(Some(txt_and_num(txt, num1, true)))
                }

                LiteralEnum::Number(num2) => {
                    // self.value = num_and_num(num1, num2);
                    Ok(Some(num_and_num(num1, num2)))
                }
                LiteralEnum::Boolean(_) => Err("cannot add booleans".to_string()),
                LiteralEnum::Void => unreachable!(),
            },

            LiteralEnum::Boolean(_) => Err("cannot add booleans".to_string()),
            LiteralEnum::Void => unreachable!(),
        }
    }
    fn set_left(&mut self, node: AstNode) {
        self.left = Some(node);
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = Some(node);
    }
}

#[derive(Clone)]
pub struct MinusNode {
    left: Option<AstNode>,
    right: Option<AstNode>,
}

impl MinusNode {
    pub fn new() -> Self {
        MinusNode {
            left: None,
            right: None,
        }
    }
}

impl BinaryNodeTrait for MinusNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symbol_tables.clone())?
                - get_number_value(&self.right, symbol_tables)?,
        ));
        Ok(Some(value))
    }
    fn set_left(&mut self, node: AstNode) {
        self.left = Some(node);
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = Some(node);
    }
}

#[derive(Clone)]
pub struct MultiplicateNode {
    left: Option<AstNode>,
    right: Option<AstNode>,
}

impl MultiplicateNode {
    pub fn new() -> Self {
        MultiplicateNode {
            left: None,
            right: None,
        }
    }
}

impl BinaryNodeTrait for MultiplicateNode {
    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symbol_tables.clone())?
                * get_number_value(&self.right, symbol_tables)?,
        ));
        Ok(Some(value))
    }

    fn set_left(&mut self, node: AstNode) {
        self.left = Some(node);
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = Some(node);
    }
}

#[derive(Clone)]
pub struct DivideNode {
    left: Option<AstNode>,
    right: Option<AstNode>,
}

impl DivideNode {
    pub fn new() -> Self {
        DivideNode {
            left: None,
            right: None,
        }
    }
}

impl BinaryNodeTrait for DivideNode {
    fn set_left(&mut self, node: AstNode) {
        self.left = Some(node);
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = Some(node);
    }

    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symbol_tables.clone())?
                / get_number_value(&self.right, symbol_tables)?,
        ));
        Ok(Some(value))
    }
}

#[derive(Clone)]
pub struct PowerNode {
    left: Option<AstNode>,
    right: Option<AstNode>,
}

impl PowerNode {
    pub fn new() -> Self {
        PowerNode {
            left: None,
            right: None,
        }
    }
}

impl BinaryNodeTrait for PowerNode {
    fn set_left(&mut self, node: AstNode) {
        self.left = Some(node);
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = Some(node);
    }

    fn visit(&self, symbol_tables: SymbolTable) -> Result<Option<LiteralEnum>, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symbol_tables.clone())?
                .powf(get_number_value(&self.right, symbol_tables)?),
        ));
        Ok(Some(value))
    }
}
