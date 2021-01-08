use crate::crocoi::INodeResult;
use crate::token::LiteralEnum;
use crate::{ast::node::*, crocoi::CrocoiNode};
use crate::{crocoi::symbol::Function, error::CrocoError};

use crate::crocoi::{self, symbol::get_symbol_type, utils::auto_deref, ICodegen, ISymbol};

impl CrocoiNode for FunctionCallNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // resolve the function arguments
        let mut visited_args = Vec::with_capacity(self.args.len());
        for arg in &mut self.args {
            let value = arg.crocoi(codegen)?.into_symbol(&self.code_pos)?;
            visited_args.push(value);
        }

        let fn_decl;
        let fn_code;

        // if we're dealing with a method, inject self as the first argument
        if let Some(method_self) = self.method.as_mut() {
            let mut method_symbol = method_self
                .crocoi(codegen)?
                .into_value_or_var_ref(&self.code_pos)?;

            method_symbol = auto_deref(method_symbol);

            let err_closure = || {
                CrocoError::new(
                    &self.code_pos,
                    &format!("no method called {}", self.fn_name),
                )
            };

            let fn_name = match &method_symbol {
                ISymbol::Struct(s) => format!("_{}_{}", s.struct_type, self.fn_name),
                ISymbol::Primitive(LiteralEnum::Str(_)) => format!("_str_{}", &self.fn_name),
                ISymbol::Primitive(LiteralEnum::Num(_)) => format!("_num_{}", &self.fn_name),
                ISymbol::Primitive(LiteralEnum::Bool(_)) => format!("_bool_{}", &self.fn_name),
                ISymbol::Array(_) => format!("_array_{}", &self.fn_name),
                _ => unreachable!(),
            };

            fn_decl = codegen
                .symtable
                .get_function_decl(&fn_name)
                .map_err(|_| err_closure())?
                .clone();

            fn_code = codegen.functions.get(&fn_name).unwrap().clone();

            // insert the self symbol in the args
            visited_args.insert(0, method_symbol);

        // this is just a regular function
        } else {
            fn_decl = codegen
                .symtable
                .get_function_decl(&self.fn_name)
                .map_err(|e| CrocoError::new(&self.code_pos, e))?
                .clone();

            fn_code = codegen.functions.get(&self.fn_name).unwrap().clone();
        }

        // ensure that the arguments provided and the arguments in the function call match
        if visited_args.len() != fn_decl.args.len() {
            return Err(CrocoError::new(
                &self.code_pos,
                format!(
                    "mismatched number of arguments in function call\nExpected {} parameter{} but got {}",
                    fn_decl.args.len(),
                    if fn_decl.args.len() < 2 { "" } else { "s" },
                    visited_args.len()
                ),
            ));
        }

        for (i, arg) in visited_args.iter().enumerate() {
            if get_symbol_type(arg) != fn_decl.args[i].arg_type {
                // if we have a method, we don't want to show the self parameter as
                // a true parameter
                let errored_param = if self.method.is_some() { i } else { i + 1 };

                return Err(CrocoError::new(
                    &self.code_pos,
                    &format!(
                        "parameter {} doesn't match function definition",
                        errored_param,
                    ),
                ));
            }
        }

        // clear the variables when evaluating the function
        let old_symbols = codegen.symtable.pop_symbols();

        let return_value;

        match fn_code {
            Function::Regular(mut body) => {
                // inject the function arguments
                // TODO: deep clone
                for (i, arg) in visited_args.into_iter().enumerate() {
                    let resolved_literal = Box::new(crocoi::node::SymbolNode::new(
                        arg.clone(),
                        self.code_pos.clone(),
                    ));

                    body.prepend_child(Box::new(VarDeclNode::new(
                        fn_decl.args[i].arg_name.clone(),
                        Some(resolved_literal),
                        Some(fn_decl.args[i].arg_type.clone()),
                        self.code_pos.clone(),
                    )));
                }

                return_value = match body.crocoi(codegen)? {
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

            Function::Builtin(callback) => {
                return_value = callback(visited_args);
            }
        }

        // if this is false then both return types are void
        if let (Some(ret_ty), Some(ret_val)) = (&fn_decl.return_type, &return_value) {
            if *ret_ty != get_symbol_type(ret_val) {
                return Err(CrocoError::new(
                    &self.code_pos,
                    "function returned a wrong type",
                ));
            }
        }

        // get back the old variables
        codegen.symtable.push_symbols(old_symbols);

        match return_value {
            None => Ok(INodeResult::Void),
            Some(val) => Ok(INodeResult::Value(val)),
        }
    }
}
