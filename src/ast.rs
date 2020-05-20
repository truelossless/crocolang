use crate::parser::TypedArg;
use dyn_clone::DynClone;
use std::fmt;

use crate::symbol::{FunctionCall, FunctionKind, SymTable, Symbol};
use crate::token::{literal_eq, LiteralEnum, OperatorEnum};

#[derive(Clone)]
pub enum AstNode {
    LeafNode(Box<dyn LeafNodeTrait>),
    UnaryNode(Box<dyn UnaryNodeTrait>),
    BinaryNode(Box<dyn BinaryNodeTrait>),
    NaryNode(Box<dyn NaryNodeTrait>),
}

impl AstNode {
    pub fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // yeah, it's big brain time !
        match self {
            AstNode::LeafNode(node) => node.visit(symtable),
            AstNode::UnaryNode(node) => node.visit(symtable),
            AstNode::BinaryNode(node) => node.visit(symtable),
            AstNode::NaryNode(node) => node.visit(symtable),
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

/*
    I'm not sure of the right design here:
    It's either a NodeResult enum, or a string stating  the type
    of the node to add AstNode.
*/

/// The type of value returned by a node
#[derive(Clone, Debug)]
pub enum NodeResult {
    /// a break statement
    Break,
    /// a continue statement
    Continue,
    /// a return statement
    /// e.g return 3
    Return(LiteralEnum),
    /// a return value
    /// e.g pi() or 42
    Literal(LiteralEnum),
}

impl NodeResult {
    pub fn into_literal(self) -> Result<LiteralEnum, String> {
        match self {
            NodeResult::Literal(l) => Ok(l),
            _ => Err("Expected a value but got an early-return keyword".to_owned()),
        }
    }

    pub fn into_return(self) -> Result<LiteralEnum, String> {
        match self {
            NodeResult::Return(l) => Ok(l),
            _ => panic!("Expected a return value but got an early-return keyword !!"),
        }
    }
}

pub trait LeafNodeTrait: DynClone {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String>;
}
dyn_clone::clone_trait_object!(LeafNodeTrait);

pub trait UnaryNodeTrait: DynClone {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String>;
    fn set_bottom(&mut self, _node: AstNode) {}
}
dyn_clone::clone_trait_object!(UnaryNodeTrait);

pub trait BinaryNodeTrait: DynClone {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String>;
    fn set_left(&mut self, _node: AstNode) {}
    fn set_right(&mut self, _node: AstNode) {}
}
dyn_clone::clone_trait_object!(BinaryNodeTrait);

pub trait NaryNodeTrait: DynClone {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String>;
    fn prepend_child(&mut self, node: AstNode);
    fn add_child(&mut self, node: AstNode);
}
dyn_clone::clone_trait_object!(NaryNodeTrait);

/// returns the LiteralEnum associated to a node
fn get_value(opt_node: &Option<AstNode>, symtable: &mut SymTable) -> Result<LiteralEnum, String> {
    match opt_node {
        Some(node) => {
            let visited = node.visit(symtable)?.into_literal()?;

            if visited.is_void() {
                panic!("should have got a value there !!");
            }
            Ok(visited)
        }
        None => Err("One variable hasn't been initialized !".to_string()),
    }
}

/// returns the number value of a node
fn get_number_value(opt_node: &Option<AstNode>, symtable: &mut SymTable) -> Result<f32, String> {
    let node = get_value(opt_node, symtable)?;
    match node {
        LiteralEnum::Number(x) => Ok(x.unwrap()),
        _ => Err("Performing a math operation on a wrong variable type !".to_string()),
    }
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
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // ensure that we're calling a function
        let fn_decl = match symtable.get_symbol(&self.fn_name) {
            Ok(Symbol::Function(fn_decl)) => fn_decl,
            Ok(Symbol::Literal(_)) => {
                return Err(format!(
                    "can't call {} as it's a variable and not a function",
                    self.fn_name
                ))
            }
            Err(_) => {
                return Err(format!(
                    "can't call {} as it hasn't been declared before",
                    self.fn_name
                ))
            }
        };

        // resolve the function arguments
        let mut visited_args = Vec::new();
        for arg in self.fn_args.iter() {
            let value = arg.visit(symtable)?.into_literal()?;
            if value.is_void() {
                return Err(format!(
                    "Empty value in {} function parameter",
                    self.fn_name
                ));
            }

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
            if !literal_eq(arg, &fn_decl.args[i].arg_type) {
                return Err(format!(
                    "parameter {} type doesn't match {} function definition",
                    i + 1,
                    self.fn_name
                ));
            }
        }

        let return_value: LiteralEnum;

        match fn_decl.body {
            FunctionKind::Regular(func_call) => {
                // get the block node of the function
                let mut block_node = match func_call {
                    AstNode::NaryNode(node) => node,
                    _ => unreachable!(),
                };

                // inject the function arguments
                for (i, arg) in visited_args.into_iter().enumerate() {
                    let resolved_literal = AstNode::LeafNode(Box::new(LiteralNode::new(arg)));

                    block_node.prepend_child(AstNode::BinaryNode(Box::new(DeclNode::new(
                        fn_decl.args[i].arg_name.clone(),
                        resolved_literal,
                        fn_decl.args[i].arg_type.clone(),
                    ))))
                }

                return_value = block_node.visit(symtable)?.into_return()?;
            }

            FunctionKind::Builtin(builtin_call) => {
                return_value = builtin_call(visited_args)?;
            }
        }

        if !literal_eq(&fn_decl.return_type, &return_value) {
            return Err(format!("function {} returned a wrong type", self.fn_name));
        }

        Ok(NodeResult::Literal(return_value))
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
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // check if the function has aready been defined
        if symtable.get_symbol(&self.name).is_ok() {
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

        symtable.register_fn(&self.name, Symbol::Function(fn_call))?;
        Ok(NodeResult::Literal(LiteralEnum::Void))
    }

    fn set_bottom(&mut self, node: AstNode) {
        self.body = node;
    }
}

/// node containing multiple instructions
/// creates a new scope
/// e.g: if body, function body, etc.
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

    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // push a new scope
        symtable.add_scope()?;

        // early return from the block
        let mut value = NodeResult::Literal(LiteralEnum::Void);

        for node in &self.body {
            value = node.visit(symtable)?;

            match value {
                // propagate the early-returns until something catches it
                NodeResult::Return(_) | NodeResult::Break | NodeResult::Continue => break,
                _ => (),
            }
        }

        // return void if the return value of the block is a literal
        if let NodeResult::Literal(_) = value {
            value = NodeResult::Literal(LiteralEnum::Void)
        }

        // we're done with this scope, drop it
        symtable.drop_scope()?;
        Ok(value)
    }
}

/// A node returning a value from a block
#[derive(Clone)]
pub struct ReturnNode {
    bottom: AstNode,
}

impl ReturnNode {
    pub fn new(bottom: AstNode) -> Self {
        ReturnNode { bottom }
    }
}

impl UnaryNodeTrait for ReturnNode {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        Ok(NodeResult::Return(
            self.bottom.visit(symtable)?.into_literal()?,
        ))
    }
}

/// a node to declare a new variable (declared variable are initialized by default)
#[derive(Clone)]
pub struct DeclNode {
    // the var_name
    left: String,
    // the variable Assignement
    right: AstNode,
    // the type of the variable
    var_type: LiteralEnum,
}

impl DeclNode {
    pub fn new(var_name: String, expr: AstNode, var_type: LiteralEnum) -> Self {
        DeclNode {
            left: var_name,
            right: expr,
            var_type,
        }
    }
}

impl BinaryNodeTrait for DeclNode {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        if symtable.same_scope_symbol(&self.left)? {
            return Err(format!(
                "The variable {} has already been declared",
                self.left
            ));
        }

        let var_value = self.right.visit(symtable)?.into_literal()?;

        if !self.var_type.is_void() && !literal_eq(&var_value, &self.var_type) {
            return Err(format!(
                "variable {} has been explicitely given a type but is declared with another one",
                &self.left
            ));
        }

        if var_value.is_void() && self.var_type.is_void() {
            return Err(format!("Unable to infer the type of {}", self.left));
        }

        symtable.insert_symbol(&self.left, var_value)?;
        Ok(NodeResult::Literal(LiteralEnum::Void))
    }
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
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let right_val = self.right.visit(symtable)?.into_literal()?;

        if right_val.is_void() {
            return Err(format!("Cannot assign {} to a void expression", &self.left));
        }
        symtable.modify_symbol(&self.left, right_val)?;
        Ok(NodeResult::Literal(LiteralEnum::Void))
    }
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
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let var = symtable.get_symbol(&self.name)?;
        let value = match var {
            Symbol::Function(_) => {
                return Err(format!(
                    "Trying to use {} as a variable but it's a function",
                    self.name
                ))
            }
            Symbol::Literal(lit) => lit,
        };
        Ok(NodeResult::Literal(value))
    }
}

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
    fn visit(&self, _symtable: &mut SymTable) -> Result<NodeResult, String> {
        // TODO: mut visit() so we can std::swap ?
        Ok(NodeResult::Literal(self.value.clone()))
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

// TODO: remove implicit cast and introduce as keyword
/// node handling additions and concatenations
impl BinaryNodeTrait for PlusNode {
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let left_val = get_value(&self.left, symtable)?;
        let right_val = get_value(&self.right, symtable)?;

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
                    Ok(NodeResult::Literal(txt_and_txt(txt1, txt2)))
                }

                LiteralEnum::Number(num) => {
                    // self.value = txt_and_num(txt1.clone(), num, false);
                    Ok(NodeResult::Literal(txt_and_num(txt1, num, false)))
                }

                LiteralEnum::Boolean(_) => Err("cannot add booleans".to_string()),
                LiteralEnum::Void => unreachable!(),
            },

            LiteralEnum::Number(num1) => match right_val {
                LiteralEnum::Text(txt) => {
                    // self.value = txt_and_num(txt.clone(), num1, true);
                    Ok(NodeResult::Literal(txt_and_num(txt, num1, true)))
                }

                LiteralEnum::Number(num2) => {
                    // self.value = num_and_num(num1, num2);
                    Ok(NodeResult::Literal(num_and_num(num1, num2)))
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
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symtable)?
                - get_number_value(&self.right, symtable)?,
        ));
        Ok(NodeResult::Literal(value))
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
    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symtable)?
                * get_number_value(&self.right, symtable)?,
        ));
        Ok(NodeResult::Literal(value))
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

    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symtable)?
                / get_number_value(&self.right, symtable)?,
        ));
        Ok(NodeResult::Literal(value))
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

    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Number(Some(
            get_number_value(&self.left, symtable)?
                .powf(get_number_value(&self.right, symtable)?),
        ));
        Ok(NodeResult::Literal(value))
    }
}

#[derive(Clone)]
/// A node used to compare two values, returns a boolean
pub struct CompareNode {
    left: Option<AstNode>,
    right: Option<AstNode>,
    compare_kind: OperatorEnum,
}

impl CompareNode {
    pub fn new(compare_kind: OperatorEnum) -> Self {
        CompareNode {
            left: None,
            right: None,
            compare_kind,
        }
    }
}

impl BinaryNodeTrait for CompareNode {
    fn set_left(&mut self, node: AstNode) {
        self.left = Some(node);
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = Some(node);
    }

    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let left_val = get_value(&self.left, symtable)?;
        let right_val = get_value(&self.right, symtable)?;

        if !literal_eq(&left_val, &right_val) {
            return Err("cannot compare different types".to_owned());
        }

        if (self.compare_kind != OperatorEnum::Equals
            || self.compare_kind == OperatorEnum::NotEquals)
            && !left_val.is_num()
        {
            return Err("can compare only numbers".to_owned());
        }

        let value = match self.compare_kind {
            OperatorEnum::Equals => left_val == right_val,
            OperatorEnum::NotEquals => left_val != right_val,
            OperatorEnum::GreaterOrEqual => left_val >= right_val,
            OperatorEnum::GreaterThan => left_val > right_val,
            OperatorEnum::LowerOrEqual => left_val <= right_val,
            OperatorEnum::LowerThan => left_val < right_val,
            _ => unreachable!(),
        };

        Ok(NodeResult::Literal(LiteralEnum::Boolean(Some(value))))
    }
}

/// a node representing an if statement
#[derive(Clone)]
pub struct IfNode {
    // comparison value (a CompareNode)
    left: AstNode,
    // if body (a BlockNode)
    right: AstNode,
}

impl IfNode {
    pub fn new(left: AstNode, right: AstNode) -> Self {
        IfNode {
            left,
            right
        }
    }
}

impl BinaryNodeTrait for IfNode {
    fn set_left(&mut self, node: AstNode) {
        self.left = node;
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = node;
    }

    fn visit(&self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // there should always be a boolean condition, check if it's fullfilled
        let cond_ok = self
            .left
            .visit(symtable)?
            .into_literal()?
            .into_bool();

        if cond_ok {
            let value = self.right.visit(symtable)?;
            match value {
                // propagate the early-return
                NodeResult::Return(_) | NodeResult::Break | NodeResult::Continue => {
                    return Ok(value)
                }
                _ => (),
            }
        }

        Ok(NodeResult::Literal(LiteralEnum::Void))
    }
}
