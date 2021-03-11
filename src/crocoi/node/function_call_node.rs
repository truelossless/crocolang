use crate::crocoi::INodeResult;
use crate::token::LiteralEnum;
use crate::{ast::node::*, crocoi::CrocoiNode};
use crate::{crocoi::symbol::Function, error::CrocoError};

use crate::crocoi::{self, symbol::get_symbol_type, ICodegen, ISymbol};

impl CrocoiNode for FunctionCallNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        let fn_decl;
        let fn_code;
        let mut visited_args = Vec::with_capacity(self.args.len());

        // if we're dealing with a method, inject self as the first argument
        if let Some(method_self) = self.method.as_mut() {
            let method_symbol = method_self.crocoi(codegen)?.into_var_ref(&self.code_pos)?;

            let err_closure = || {
                CrocoError::new(
                    &self.code_pos,
                    &format!("no method called {}", self.fn_name),
                )
            };

            let fn_name = match dbg!(&*method_symbol.get_ref().borrow()) {
                ISymbol::Struct(s) => format!("_{}_{}", s.struct_type, self.fn_name),
                ISymbol::Primitive(LiteralEnum::Str(_)) => format!("_str_{}", &self.fn_name),
                ISymbol::Primitive(LiteralEnum::Fnum(_)) => format!("_fnum_{}", &self.fn_name),
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
            visited_args.push(method_symbol);

        // this is just a regular function
        } else {
            fn_decl = codegen
                .symtable
                .get_function_decl(&self.fn_name)
                .map_err(|e| CrocoError::new(&self.code_pos, e))?
                .clone();

            fn_code = codegen.functions.get(&self.fn_name).unwrap().clone();
        }

        // resolve the function arguments
        for arg in &mut self.args {
            let value = arg.crocoi(codegen)?.into_symbol(&self.code_pos)?;
            visited_args.push(value);
        }

        // ensure that the arguments provided and the arguments in the function call match
        if visited_args.len() != fn_decl.args.len() {
            return Err(CrocoError::mismatched_number_of_arguments_error(
                &self.code_pos,
                fn_decl.args.len(),
                visited_args.len(),
            ));
        }

        for (i, arg) in visited_args.iter().enumerate() {
            if get_symbol_type(arg) != fn_decl.args[i].arg_type {
                return Err(CrocoError::parameter_error(
                    &self.code_pos,
                    i,
                    self.method.is_some(),
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
