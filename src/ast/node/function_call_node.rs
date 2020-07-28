use crate::ast::node::*;
use crate::ast::{AstNode, NodeResult};
use crate::symbol::{get_symbol_type, symbol_eq, FunctionKind, SymTable, Symbol, SymbolContent};
use crate::token::{CodePos, LiteralEnum};

use crate::error::CrocoError;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct FunctionCallNode {
    fn_name: String,
    args: Vec<Box<dyn AstNode>>,
    // wether or not this is a method (we have to inject "self" in this case)
    method: Option<Box<dyn AstNode>>,
    code_pos: CodePos,
}

impl FunctionCallNode {
    pub fn new(
        fn_name: String,
        args: Vec<Box<dyn AstNode>>,
        method: Option<Box<dyn AstNode>>,
        code_pos: CodePos,
    ) -> Self {
        FunctionCallNode {
            fn_name,
            args,
            method,
            code_pos,
        }
    }
}

impl AstNode for FunctionCallNode {
    fn add_child(&mut self, node: Box<dyn AstNode>) {
        if self.method.is_none() {
            self.method = Some(node);
        } else {
            unreachable!()
        }
    }

    fn visit(&mut self, symtable: &mut SymTable) -> Result<NodeResult, CrocoError> {
        // resolve the function arguments
        let mut visited_args = Vec::new();
        for arg in &mut self.args {
            let value = arg.visit(symtable)?.into_symbol(&self.code_pos)?;
            visited_args.push(value);
        }

        let mut self_arg = None;
        let fn_decl;

        // if we're dealing with a method, inject self as the first argument
        if let Some(method_self) = self.method.as_mut() {
            let mut method_symbol = method_self.visit(symtable)?.into_symbol(&self.code_pos)?;

            // auto deref if we have a Ref
            loop {
                let reference;

                if let SymbolContent::Ref(r) = &*method_symbol.borrow() {
                    reference = r.clone();
                } else {
                    break;
                }

                method_symbol = reference;
            }

            // inject the "self" argument
            self_arg = Some(method_symbol.clone());

            match &*method_symbol.borrow() {
                // struct methods
                SymbolContent::Struct(s) => {
                    // look for the method declaration
                    fn_decl = symtable
                        .get_struct_decl(&s.struct_type)
                        .map_err(|e| CrocoError::new(&self.code_pos, e))?
                        .methods
                        .get(&self.fn_name)
                        .ok_or_else(|| {
                            CrocoError::new(
                                &self.code_pos,
                                format!("no method called {}", self.fn_name),
                            )
                        })?
                        .clone()
                }

                // str methods
                SymbolContent::Primitive(LiteralEnum::Str(Some(_s))) => {
                    todo!();
                }

                // num methods
                SymbolContent::Primitive(LiteralEnum::Num(Some(_n))) => {
                    todo!();
                }

                // bool methods
                SymbolContent::Primitive(LiteralEnum::Bool(Some(_b))) => {
                    todo!();
                }

                // array methods
                SymbolContent::Array(_arr) => {
                    todo!();
                }

                _ => unreachable!(),
            };

        // this is just a regular function
        } else {
            fn_decl = symtable
                .get_function_decl(&self.fn_name)
                .map_err(|e| CrocoError::new(&self.code_pos, e))?
                .clone();
        }

        // ensure that the arguments provided and the arguments in the function call match
        if visited_args.len() != fn_decl.args.len() {
            return Err(CrocoError::new(
                &self.code_pos,
                format!(
                "mismatched number of arguments in function call\n expected {} parameters but got {}",
                fn_decl.args.len(),
                visited_args.len()
            ),
            ));
        }

        for (i, arg) in visited_args.iter().enumerate() {
            if !symbol_eq(&*arg.borrow(), &fn_decl.args[i].arg_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    format!("parameter {} doesn't match function definition", i + 1,),
                ));
            }
        }

        let return_value: Symbol;

        match fn_decl.body.unwrap() {
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

                if let Some(self_symbol) = self_arg {
                    block_node.prepend_child(Box::new(VarDeclNode::new(
                        "self".to_owned(),
                        Some(Box::new(SymbolNode::new(
                            self_symbol.clone(),
                            self.code_pos.clone(),
                        ))),
                        get_symbol_type(self_symbol),
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
                return_value = Rc::new(RefCell::new(builtin_call(visited_args)));
            }
        }

        if !symbol_eq(&fn_decl.return_type, &*return_value.borrow()) {
            return Err(CrocoError::new(
                &self.code_pos,
                "function returned a wrong type".to_owned(),
            ));
        }

        Ok(NodeResult::Symbol(return_value))
    }
}
