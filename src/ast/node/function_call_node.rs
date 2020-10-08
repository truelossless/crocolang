use crate::ast::node::*;
use crate::ast::{AstNode, INodeResult};
use crate::symbol::FunctionKind;
use crate::token::{CodePos, LiteralEnum};

use crate::error::CrocoError;

#[cfg(feature = "crocoi")]
use crate::crocoi::{self, symbol::get_symbol_type, utils::auto_deref, ISymTable, ISymbol};

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

    #[cfg(feature = "crocoi")]
    fn crocoi(&mut self, symtable: &mut ISymTable) -> Result<INodeResult, CrocoError> {
        // resolve the function arguments
        let mut visited_args = Vec::new();
        for arg in &mut self.args {
            let value = arg.crocoi(symtable)?.into_symbol(&self.code_pos)?;
            visited_args.push(value);
        }

        let mut self_arg = None;
        let fn_decl;

        // if we're dealing with a method, inject self as the first argument
        if let Some(method_self) = self.method.as_mut() {
            let mut method_symbol = method_self
                .crocoi(symtable)?
                .into_value_or_var_ref(&self.code_pos)?;

            // inject the "self" argument
            self_arg = Some(method_symbol.clone());

            method_symbol = auto_deref(method_symbol);

            match method_symbol {
                // struct methods
                ISymbol::Struct(s) => {
                    // look for the method declaration
                    fn_decl = symtable
                        .get_struct_decl(&s.struct_type)
                        .map_err(|e| CrocoError::new(&self.code_pos, e))?
                        .methods
                        .get(&self.fn_name)
                        .ok_or_else(|| {
                            CrocoError::new(
                                &self.code_pos,
                                &format!("no method called {}", self.fn_name),
                            )
                        })?
                        .clone()
                }

                // str methods
                ISymbol::Primitive(LiteralEnum::Str(_s)) => {
                    todo!();
                }

                // num methods
                ISymbol::Primitive(LiteralEnum::Num(_n)) => {
                    todo!();
                }

                // bool methods
                ISymbol::Primitive(LiteralEnum::Bool(_b)) => {
                    todo!();
                }

                // array methods
                ISymbol::Array(_arr) => {
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
                &format!(
                "mismatched number of arguments in function call\n expected {} parameters but got {}",
                fn_decl.args.len(),
                visited_args.len()
            ),
            ));
        }

        for (i, arg) in visited_args.iter().enumerate() {
            if !get_symbol_type(arg).eq(&fn_decl.args[i].arg_type) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    &format!("parameter {} doesn't match function definition", i + 1,),
                ));
            }
        }

        let return_value;

        match fn_decl.body {
            FunctionKind::Regular(mut block_node) => {
                // get the block node of the function

                // inject the function arguments
                // TODO: deep clone
                for (i, arg) in visited_args.into_iter().enumerate() {
                    let resolved_literal = Box::new(crocoi::node::SymbolNode::new(
                        arg.clone(),
                        self.code_pos.clone(),
                    ));

                    block_node.prepend_child(Box::new(VarDeclNode::new(
                        fn_decl.args[i].arg_name.clone(),
                        Some(resolved_literal),
                        Some(fn_decl.args[i].arg_type.clone()),
                        self.code_pos.clone(),
                    )));
                }

                if let Some(self_symbol) = self_arg {
                    let var_type = Some(get_symbol_type(&self_symbol));

                    block_node.prepend_child(Box::new(VarDeclNode::new(
                        "self".to_owned(),
                        Some(Box::new(crocoi::node::SymbolNode::new(
                            self_symbol,
                            self.code_pos.clone(),
                        ))),
                        var_type,
                        self.code_pos.clone(),
                    )));
                }

                return_value = match block_node.crocoi(symtable)? {
                    INodeResult::Return(ret) => ret,
                    INodeResult::Break => {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            "cannot exit a function with a break",
                        ))
                    }
                    INodeResult::Continue => {
                        return Err(CrocoError::new(
                            &self.code_pos,
                            "cannot use continue in a function",
                        ))
                    }
                    INodeResult::Value(val) => Some(val),
                    INodeResult::Variable(var) => Some(var.borrow().clone()),
                    // this must be void if it's returned by a block node
                    INodeResult::Void => None,
                }
            }

            FunctionKind::Builtin(builtin_call) => {
                return_value = builtin_call(visited_args);
            }
        }

        // if this is false then both return types are void
        if let (Some(ret_ty), Some(ret_val)) = (&fn_decl.return_type, &return_value) {
            if !ret_ty.eq(&get_symbol_type(ret_val)) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "function returned a wrong type",
                ));
            }
        }

        match return_value {
            None => Ok(INodeResult::Void),
            Some(val) => Ok(INodeResult::Value(val)),
        }
    }
}
