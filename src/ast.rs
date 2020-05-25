use dyn_clone::DynClone;
use std::{fmt, fs};
use unicode_segmentation::UnicodeSegmentation;

use crate::lexer::Lexer;
use crate::parser::{Parser, TypedArg};

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
    pub fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String>;
}
dyn_clone::clone_trait_object!(LeafNodeTrait);

pub trait UnaryNodeTrait: DynClone {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String>;
    fn set_bottom(&mut self, _node: AstNode) {}
}
dyn_clone::clone_trait_object!(UnaryNodeTrait);

pub trait BinaryNodeTrait: DynClone {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String>;
    fn set_left(&mut self, _node: AstNode) {}
    fn set_right(&mut self, _node: AstNode) {}
}
dyn_clone::clone_trait_object!(BinaryNodeTrait);

pub trait NaryNodeTrait: DynClone {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String>;
    fn prepend_child(&mut self, node: AstNode);
    fn add_child(&mut self, node: AstNode);
}
dyn_clone::clone_trait_object!(NaryNodeTrait);

/// returns the LiteralEnum associated to a node
fn get_value(
    opt_node: &mut Option<AstNode>,
    symtable: &mut SymTable,
) -> Result<LiteralEnum, String> {
    match opt_node {
        Some(node) => {
            let visited = node.visit(symtable)?.into_literal()?;

            if visited.is_void() {
                panic!("should have got a value there !!");
            }
            Ok(visited)
        }
        None => Err("One variable hasn't been initialized !".to_owned()),
    }
}

/// returns the number value of a node
fn get_number_value(
    opt_node: &mut Option<AstNode>,
    symtable: &mut SymTable,
) -> Result<f32, String> {
    let node = get_value(opt_node, symtable)?;
    match node {
        LiteralEnum::Num(x) => Ok(x.unwrap()),
        _ => Err("Performing a math operation on a wrong variable type !".to_owned()),
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

impl NaryNodeTrait for FunctionCallNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // resolve the function arguments
        let mut visited_args = Vec::new();
        for arg in self.fn_args.iter_mut() {
            let value = arg.visit(symtable)?.into_literal()?;
            if value.is_void() {
                return Err(format!(
                    "Empty value in {} function parameter",
                    self.fn_name
                ));
            }
            visited_args.push(value);
        }
        // this clone call is taking 30-50% of the execution time in fib.croco >:(
        let fn_decl = symtable.get_function(&self.fn_name)?.clone();
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
                    ))));
                }

                return_value = match block_node.visit(symtable)? {
                    NodeResult::Return(ret) => ret,
                    NodeResult::Break => {
                        return Err("cannot exit a function with a break".to_owned())
                    }
                    NodeResult::Continue => {
                        return Err("cannot use continue in a function".to_owned())
                    }
                    // this must be void if it's returned by a block node
                    NodeResult::Literal(l) => l,
                }
            }

            FunctionKind::Builtin(builtin_call) => {
                return_value = builtin_call(visited_args);
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
    return_type: Option<LiteralEnum>,
    args: Option<Vec<TypedArg>>,
    body: Option<AstNode>,
}

impl FunctionDeclNode {
    pub fn new(name: String, return_type: LiteralEnum, args: Vec<TypedArg>) -> Self {
        FunctionDeclNode {
            name,
            return_type: Some(return_type),
            args: Some(args),
            body: Some(AstNode::NaryNode(Box::new(BlockNode::new(BlockScope::New)))),
        }
    }
}

impl UnaryNodeTrait for FunctionDeclNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // check if the function has aready been defined
        if symtable.get_function(&self.name).is_ok() {
            return Err(format!("{} function name already used", self.name));
        }

        // once the function is declared we can move out its content since this node is not going to be used again
        let body = std::mem::replace(&mut self.body, None).unwrap();
        let args = std::mem::replace(&mut self.args, None).unwrap();
        let name = std::mem::replace(&mut self.name, String::new());
        let return_type = std::mem::replace(&mut self.return_type, None).unwrap();

        let fn_call = FunctionCall::new(args, return_type, FunctionKind::Regular(body));

        symtable.register_fn(name, Symbol::Function(fn_call));
        Ok(NodeResult::Literal(LiteralEnum::Void))
    }

    fn set_bottom(&mut self, node: AstNode) {
        self.body = Some(node);
    }
}

#[derive(Clone)]
pub enum BlockScope {
    New,
    Keep,
}

impl Default for BlockScope {
    fn default() -> Self {
        BlockScope::New
    }
}

/// node containing multiple instructions
/// creates a new scope, or not
/// e.g: if body, function body, etc.
#[derive(Clone)]
pub struct BlockNode {
    // all instructions of the block node
    body: Vec<AstNode>,
    scope: BlockScope,
    // instructions that get prepended, e.g variables in fn calls
    // prepended: Vec<AstNode>,
    // same as previous, useful for future defer calls
    // appended: Vec<AstNode>
}

impl BlockNode {
    pub fn new(scope: BlockScope) -> Self {
        BlockNode {
            body: Vec::new(),
            scope
            // prepended: Vec::new(),
            // appended: Vec::new(),
        }
    }
}

impl NaryNodeTrait for BlockNode {
    fn prepend_child(&mut self, node: AstNode) {
        self.body.insert(0, node);
    }

    fn add_child(&mut self, node: AstNode) {
        self.body.push(node);
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        
        // push a new scope if needed
        match self.scope {
            BlockScope::New => symtable.add_scope(),
            BlockScope::Keep => ()
        }

        // early return from the block
        let mut value = NodeResult::Literal(LiteralEnum::Void);
        // iterate over all nodes in the body
        for node in self.body.iter_mut()
        // .chain(self.prepended.iter_mut())
        // .chain(self.appended.iter_mut())
        {
            value = node.visit(symtable)?;

            match value {
                // propagate the early-returns until something catches it
                NodeResult::Return(_) | NodeResult::Break | NodeResult::Continue => break,
                _ => (),
            }
        }

        // clean up the injected statements
        // self.prepended.clear();
        // self.appended.clear();

        // return void if there is no return value
        if let NodeResult::Literal(_) = value {
            value = NodeResult::Literal(LiteralEnum::Void)
        }

        // we're done with this scope, drop it
        match self.scope {
            BlockScope::New => symtable.drop_scope(),
            BlockScope::Keep => ()
        }

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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        if symtable.same_scope_symbol(&self.left) {
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
        symtable.insert_symbol(&self.left, var_value);
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

impl AssignmentNode {
    pub fn new(var_name: String, expr: AstNode) -> Self {
        AssignmentNode {
            left: var_name,
            right: expr,
        }
    }
}

impl BinaryNodeTrait for AssignmentNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = symtable.get_literal(&self.name)?;
        Ok(NodeResult::Literal(value.clone()))
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

// actually we can't move out as a node can be visited multiple times in a loop
impl LeafNodeTrait for LiteralNode {
    fn visit(&mut self, _symtable: &mut SymTable) -> Result<NodeResult, String> {
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
// TODO: put all math nodes together ?
/// node handling additions and concatenations
impl BinaryNodeTrait for PlusNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let left_val = get_value(&mut self.left, symtable)?;
        let right_val = get_value(&mut self.right, symtable)?;

        // different kinds of additions can happen
        // the PlusNode also works for concatenation.

        let txt_and_txt = |txt1: Option<String>, txt2: Option<String>| -> LiteralEnum {
            let mut left_str = txt1.unwrap();
            let right_str = txt2.unwrap();
            left_str.push_str(&right_str);
            LiteralEnum::Str(Some(left_str))
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

                LiteralEnum::Str(Some(txt_str))
            };

        let num_and_num = |num1: Option<f32>, num2: Option<f32>| -> LiteralEnum {
            let num1_val = num1.unwrap();
            let num2_val = num2.unwrap();

            LiteralEnum::Num(Some(num1_val + num2_val))
        };

        match left_val {
            LiteralEnum::Str(txt1) => match right_val {
                LiteralEnum::Str(txt2) => Ok(NodeResult::Literal(txt_and_txt(txt1, txt2))),

                LiteralEnum::Num(num) => Ok(NodeResult::Literal(txt_and_num(txt1, num, false))),

                LiteralEnum::Bool(_) => Err("cannot add booleans".to_string()),
                LiteralEnum::Void => unreachable!(),
            },

            LiteralEnum::Num(num1) => match right_val {
                LiteralEnum::Str(txt) => Ok(NodeResult::Literal(txt_and_num(txt, num1, true))),

                LiteralEnum::Num(num2) => {
                    // self.value = num_and_num(num1, num2);
                    Ok(NodeResult::Literal(num_and_num(num1, num2)))
                }
                LiteralEnum::Bool(_) => Err("cannot add booleans".to_string()),
                LiteralEnum::Void => unreachable!(),
            },

            LiteralEnum::Bool(_) => Err("cannot add booleans".to_string()),
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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Num(Some(
            get_number_value(&mut self.left, symtable)?
                - get_number_value(&mut self.right, symtable)?,
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
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Num(Some(
            get_number_value(&mut self.left, symtable)?
                * get_number_value(&mut self.right, symtable)?,
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

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Num(Some(
            get_number_value(&mut self.left, symtable)?
                / get_number_value(&mut self.right, symtable)?,
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

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let value = LiteralEnum::Num(Some(
            get_number_value(&mut self.left, symtable)?
                .powf(get_number_value(&mut self.right, symtable)?),
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

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        let left_val = get_value(&mut self.left, symtable)?;
        let right_val = get_value(&mut self.right, symtable)?;

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

        Ok(NodeResult::Literal(LiteralEnum::Bool(Some(value))))
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
        IfNode { left, right }
    }
}

impl BinaryNodeTrait for IfNode {
    fn set_left(&mut self, node: AstNode) {
        self.left = node;
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = node;
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // there should always be a boolean condition, check if it's fullfilled
        let cond_ok = self.left.visit(symtable)?.into_literal()?.into_bool();

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

/// a node representing a while statement
#[derive(Clone)]
pub struct WhileNode {
    // comparison value (a CompareNode)
    left: AstNode,
    // while body (a BlockNode)
    right: AstNode,
}

impl WhileNode {
    pub fn new(left: AstNode, right: AstNode) -> Self {
        WhileNode { left, right }
    }
}

impl BinaryNodeTrait for WhileNode {
    fn set_left(&mut self, node: AstNode) {
        self.left = node;
    }

    fn set_right(&mut self, node: AstNode) {
        self.right = node;
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // loop while the condition is ok
        while self.left.visit(symtable)?.into_literal()?.into_bool() {
            let value = self.right.visit(symtable)?;
            match value {
                // propagate the early-return
                NodeResult::Return(_) => return Ok(value),
                NodeResult::Break => return Ok(NodeResult::Literal(LiteralEnum::Void)),
                NodeResult::Literal(_) | NodeResult::Continue => (),
            }
        }

        Ok(NodeResult::Literal(LiteralEnum::Void))
    }
}

/// a node representing a break statement
#[derive(Clone)]
pub struct BreakNode {}

impl BreakNode {
    pub fn new() -> Self {
        BreakNode {}
    }
}

impl LeafNodeTrait for BreakNode {
    fn visit(&mut self, _symtable: &mut SymTable) -> Result<NodeResult, String> {
        Ok(NodeResult::Break)
    }
}

/// a node representing a continue statement
#[derive(Clone)]
pub struct ContinueNode {}

impl ContinueNode {
    pub fn new() -> Self {
        ContinueNode {}
    }
}

impl LeafNodeTrait for ContinueNode {
    fn visit(&mut self, _symtable: &mut SymTable) -> Result<NodeResult, String> {
        Ok(NodeResult::Continue)
    }
}

#[derive(Clone)]
pub struct ImportNode {
    name: String,
    bottom: Option<AstNode>,
}

// imports code from another module, at runtime.
impl ImportNode {
    pub fn new(name: String) -> Self {
        ImportNode { name, bottom: None }
    }
}

impl UnaryNodeTrait for ImportNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, String> {
        // we have a relative path e.g import "./my_module"
        // look for a file with this name
        if self.name.contains('.') {
            let file_contents = fs::read_to_string(format!("{}.croco", self.name))
                .map_err(|_| format!("cannot find the file {}.croco", self.name))?;

            // lex the new import
            // namespace everything created there with the import name
            let mut lexer = Lexer::new();

            // find the real import name
            // e.g "./module/me/love" => "love"
            let iter = self.name.split_word_bounds().rev();
            let mut import_name = "";
            for word in iter {
                if word == "/" {
                    break;
                }

                import_name = word;
            }

            // import name should be the real import name now.
            println!("{}", import_name);
            lexer.set_namespace(import_name.to_owned());
            let tokens = lexer.process(&file_contents)?;

            // .. and resolve to an AST the import
            // TODO: export only when pub is used
            let mut parser = Parser::new();

            // we can now add the import as a closure:
            // a block node which doesn't introduce a new scope
            parser.set_scope(BlockScope::Keep);
            let mut bottom = parser.process(tokens)?;
            bottom.visit(symtable)?;
            self.bottom = Some(bottom);

            Ok(NodeResult::Literal(LiteralEnum::Void))

        // we have an absolute path e.g import "math"
        // we are looking for a builtin module with this name
        } else {
            // check if the module part of the std library
            if symtable.import_builtin_module(&self.name) {
                Ok(NodeResult::Literal(LiteralEnum::Void))
            } else {
                Err(format!(
                    "{} module not found in the builtin library",
                    self.name
                ))
            }
        }
    }
}
