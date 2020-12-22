use crate::ast::node::StructDeclNode;
use crate::symbol::{Decl, StructDecl};
use crate::{crocoi, parser::TypedArg};
use crate::{crocoi::CrocoiNode, error::CrocoError};
use crate::{
    crocoi::{ICodegen, INodeResult},
    symbol_type::SymbolType,
};

impl CrocoiNode for StructDeclNode {
    fn crocoi(&mut self, codegen: &mut ICodegen) -> Result<INodeResult, CrocoError> {
        // this node is not going to be called again, we can take the value
        let struct_decl = StructDecl {
            fields: self.fields.take().unwrap(),
        };

        let self_obj = SymbolType::Struct(self.name.to_owned());

        // inject in the method declaration the self argument
        for (method_name, (mut method_decl, method_body)) in self.methods.take().unwrap() {
            method_decl.args.insert(
                0,
                TypedArg {
                    arg_name: "self".to_owned(),
                    arg_type: self_obj.clone(),
                },
            );

            // foo.call() -> _Foo_call(&foo)
            let fn_name = format!("_{}_{}", self.name, method_name);

            codegen
                .symtable
                .register_decl(fn_name.clone(), Decl::FunctionDecl(method_decl))
                .map_err(|_| {
                    CrocoError::new(&self.code_pos, "method already existing with the same name")
                })?;

            // register the function body
            codegen
                .functions
                .insert(fn_name, crocoi::symbol::Function::Regular(method_body));
        }

        // register the struct declaration
        codegen
            .symtable
            .register_decl(self.name.clone(), Decl::StructDecl(struct_decl))
            .map_err(|e| CrocoError::new(&self.code_pos, &e))?;

        Ok(INodeResult::Void)
    }
}
