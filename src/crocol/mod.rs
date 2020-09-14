pub mod symbol;

pub use self::symbol::Codegen;
pub use self::symbol::LNodeResult;
pub use self::symbol::LSymbol;

pub mod utils;

use std::{cell::RefCell, fs};
use std::{path::Path, rc::Rc};

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::{
    error::{CrocoError, CrocoErrorKind},
    linker::Linker,
};
use crate::{symbol::SymTable, token::CodePos};
use inkwell::{
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};

#[derive(Default)]
pub struct Crocol {
    file_path: String,
    asm_flag: bool,
    object_flag: bool,
    output_flag: String,
    verbose_flag: bool,
}

impl Crocol {
    pub fn new() -> Self {
        Crocol {
            file_path: String::new(),
            asm_flag: false,
            object_flag: false,
            output_flag: String::new(),
            verbose_flag: false,
        }
    }

    pub fn emit_assembly(&mut self) {
        self.asm_flag = true;
    }

    pub fn emit_object_file(&mut self) {
        self.object_flag = true;
    }

    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose_flag = verbose;
    }

    pub fn set_output(&mut self, output: String) {
        self.output_flag = output;
    }

    pub fn exec_file(&mut self, file_path: &str) -> Result<(), CrocoError> {
        let contents = fs::read_to_string(file_path).map_err(|_| {
            let mut err = CrocoError::new(
                &CodePos {
                    file: Rc::from(file_path),
                    line: 0,
                    word: 0,
                },
                &format!("file not found: {}", file_path),
            );
            err.set_kind(CrocoErrorKind::IO);
            err
        })?;

        self.file_path = file_path.to_owned();
        self.exec(&contents)
    }

    pub fn exec(&mut self, code: &str) -> Result<(), CrocoError> {
        let tokens;
        let mut tree;

        let mut lexer = Lexer::new();
        lexer.set_file(&self.file_path);
        match lexer.process(code) {
            Ok(t) => tokens = t,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Syntax);
                return Err(e);
            }
        }

        // println!("tokens: {:?}", &tokens);
        let mut parser = Parser::new();
        match parser.process(tokens) {
            Ok(root_node) => tree = root_node,
            Err(mut e) => {
                e.set_kind(CrocoErrorKind::Parse);
                return Err(e);
            }
        }

        // create the context and the default module
        let context = Context::create();
        let module = context.create_module("main");

        // initialize the target from this machine's specs
        // https://gitter.im/inkwell-rs/Lobby?at=5ef469846c06cd1bf452d3d3
        Target::initialize_all(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).map_err(|_| {
            CrocoError::from_type("cannot create target triple", CrocoErrorKind::CompileTarget)
        })?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| {
                CrocoError::from_type(
                    "cannot create target machine",
                    CrocoErrorKind::CompileTarget,
                )
            })?;

        // append the basic entry block
        let fn_return = context.void_type().fn_type(&[], false);
        let main_fn = module.add_function("main", fn_return, None);

        let codegen = Codegen {
            context: &context,
            module,
            builder: context.create_builder(),
            symtable: RefCell::new(SymTable::new()),
            ptr_size: context.ptr_sized_int_type(&target_machine.get_target_data(), None),
            current_fn: RefCell::new(main_fn),
        };

        if let Err(mut e) = tree.crocol(&codegen) {
            e.set_kind(CrocoErrorKind::Runtime);
            return Err(e);
        }

        // this should never fail if our nodes are right
        codegen.module.verify().unwrap();

        // emit an executable if we don't specifically want to emit assembly and object files
        let exe_flag = !self.asm_flag && !self.object_flag;

        // get the llvm file output name
        let llvm_output_filename = if !self.output_flag.is_empty() && !exe_flag {
            self.output_flag.clone()
        } else {
            let ext = if self.asm_flag { "asm" } else { "o" };
            format!("{}.{}", strip_ext(&self.file_path), ext)
        };

        let emit_method = if self.asm_flag {
            FileType::Assembly
        } else {
            FileType::Object
        };

        target_machine
            .write_to_file(&codegen.module, emit_method, llvm_output_filename.as_ref())
            .map_err(|e| {
                CrocoError::from_type(e.to_str().unwrap(), CrocoErrorKind::CompileTarget)
            })?;

        // if the user specified -S or -c, we're done here
        if self.asm_flag || self.object_flag {
            return Ok(());
        }

        // else we need to link the object file into an executable
        let mut linker = Linker::new();

        let linker_search = linker
            .find_linker()
            .map_err(|e| CrocoError::from_type(e, CrocoErrorKind::Linker))?;

        if self.verbose_flag {
            println!("{}", linker_search);
        }

        let exe_output_filename = if !self.output_flag.is_empty() {
            self.output_flag.clone()
        } else {
            let ext = if cfg!(windows) { "exe" } else { "" };
            format!("{}.{}", strip_ext(&self.file_path), ext)
        };

        let link_stage = linker
            .link(&llvm_output_filename, &exe_output_filename)
            .map_err(|e| CrocoError::from_type(e, CrocoErrorKind::Linker))?;

        if self.verbose_flag {
            println!("{}", link_stage);
        }

        // we can now remove the unwanted object file
        fs::remove_file(llvm_output_filename).map_err(|_| {
            CrocoError::from_type("cannot remove temporary object file", CrocoErrorKind::IO)
        })?;

        Ok(())
    }
}

fn strip_ext(file: &str) -> &str {
    Path::new(file)
        .file_stem()
        .unwrap_or_else(|| file.as_ref())
        .to_str()
        .unwrap()
}
