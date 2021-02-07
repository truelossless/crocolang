use crate::ast::BackendNode;
use crate::crocoi::symbol::{Array, ICodegen, ISymbol, Struct};
use crate::error::CrocoError;
use crate::{
    symbol_type::SymbolType,
    token::{CodePos, LiteralEnum, LiteralEnum::*},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// returns the LiteralEnum associated to a node
pub fn get_value(
    opt_node: &mut Option<Box<dyn BackendNode>>,
    codegen: &mut ICodegen,
    code_pos: &CodePos,
) -> Result<LiteralEnum, CrocoError> {
    Ok(opt_node
        .as_mut()
        .ok_or_else(|| CrocoError::new(code_pos, "variable hasn't been initialized"))?
        .crocoi(codegen)?
        .into_symbol(code_pos)?
        .into_primitive()
        .map_err(|_| CrocoError::new(code_pos, "cannot use this type in an expression"))?)
}

/// Auto dereferences as many times as needed  
/// e.g (&a).foo -> a.foo
pub fn auto_deref(mut symbol: ISymbol) -> ISymbol {
    loop {
        let reference;

        if let ISymbol::Ref(r) = symbol {
            reference = r.borrow().clone();
        } else {
            break;
        }

        symbol = reference;
    }

    symbol
}

/// initializes recursively a symbol to its default value
pub fn init_default(
    symbol_type: &SymbolType,
    codegen: &mut ICodegen,
    code_pos: &CodePos,
) -> Result<ISymbol, CrocoError> {
    Ok(match symbol_type {
        SymbolType::Num => ISymbol::Primitive(Num(0)),
        SymbolType::Fnum => ISymbol::Primitive(Fnum(0.)),
        SymbolType::Bool => ISymbol::Primitive(LiteralEnum::Bool(false)),
        SymbolType::Str => ISymbol::Primitive(Str(String::new())),
        SymbolType::Array(array_type) => ISymbol::Array(Array {
            array_type: array_type.clone(),
            contents: Vec::new(),
        }),
        SymbolType::Ref(_) => return Err(CrocoError::new(code_pos, "dangling reference")),
        SymbolType::Struct(struct_type) => {
            let struct_decl = codegen
                .symtable
                .get_struct_decl(struct_type)
                .map_err(|e| CrocoError::new(&code_pos, e))?
                .clone();

            let rc_fields = struct_decl.fields.into_iter();

            let mut fields = HashMap::new();
            for (k, x) in rc_fields {
                fields.insert(
                    k,
                    Rc::new(RefCell::new(init_default(&x, codegen, code_pos)?)),
                );
            }

            ISymbol::Struct(Struct {
                fields,
                struct_type: struct_type.clone(),
            })
        }
        SymbolType::Map(_, _) => todo!(),
        SymbolType::Function(_) => {
            return Err(CrocoError::new(code_pos, "dangling function pointer"))
        }
        SymbolType::CrocoType => ISymbol::CrocoType(SymbolType::CrocoType),
    })
}
