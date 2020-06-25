use crate::ast::node::*;
use crate::ast::{AstNode, NodeResult};
use crate::symbol::{symbol_eq, FunctionKind, SymTable, Symbol};
use crate::token::CodePos;

use crate::error::CrocoError;

#[derive(Clone)]
pub struct FunctionCallNode {
    name: String,
    args: Vec<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl FunctionCallNode {
    pub fn new(name: String, args: Vec<Box<dyn AstNode>>, code_pos: CodePos) -> Self {
        FunctionCallNode {
            name,
            args,
            code_pos,
        }
    }
}

impl AstNode for FunctionCallNode {
    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // resolve the function arguments
        let mut visited_args = Vec::new();
        for arg in self.args.iter_mut() {
            let value = arg.visit(symtable)?.into_symbol(&self.code_pos)?;
            visited_args.push(value);
        }
        // this clone call is taking 30-50% of the execution time in fib.croco >:(
        let fn_decl = symtable
            .get_function_decl(&self.name)
            .map_err(|e| CrocoError::new(&self.code_pos, e))?
            .clone();
        // ensure that the arguments provided and the arguments in the function call match
        if visited_args.len() != fn_decl.args.len() {
            return Err(CrocoError::new(
                &self.code_pos,
                format!(
                "mismatched number of arguments in function {}\n expected {} parameters but got {}",
                self.name,
                fn_decl.args.len(),
                visited_args.len()
            ),
            ));
        }
        for (i, arg) in visited_args.iter().enumerate() {
            if !symbol_eq(arg, &fn_decl.args[i].arg_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    format!(
                        "parameter {} type doesn't match {} function definition",
                        i + 1,
                        self.name
                    ),
                ));
            }
        }

        let return_value: Symbol;

        match fn_decl.body {
            FunctionKind::Regular(mut block_node) => {
                // get the block node of the function

                // inject the function arguments
                for (i, arg) in visited_args.into_iter().enumerate() {
                    let resolved_literal = Box::new(SymbolNode::new(arg, self.code_pos.clone()));

                    block_node.prepend_child(Box::new(VarDeclNode::new(
                        fn_decl.args[i].arg_name.clone(),
                        Some(resolved_literal),
                        fn_decl.args[i].arg_type.clone(),
                        self.code_pos.clone(),
                    )));
                }

                return_value = match block_node.visit(symtable)? {
                    NodeResult::Return(ret) => ret,
                    NodeResult::Break => {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            "cannot exit a function with a break".to_owned(),
                        ))
                    }
                    NodeResult::Continue => {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            "cannot use continue in a function".to_owned(),
                        ))
                    }
                    // this must be void if it's returned by a block node
                    NodeResult::Symbol(s) => s,
                }
            }

            FunctionKind::Builtin(builtin_call) => {
                return_value = builtin_call(visited_args);
            }
        }

        if !symbol_eq(&fn_decl.return_type, &return_value) {
            return Err(CrocoError::new(
                &self.code_pos,
                format!("function {} returned a wrong type", self.name),
            ));
        }

        Ok(NodeResult::Symbol(return_value))
    }

    fn prepend_child(&mut self, node: Box<dyn AstNode>) {
        self.args.insert(0, node);
    }

    fn add_child(&mut self, node: Box<dyn AstNode>) {
        self.args.push(node);
    }
}
