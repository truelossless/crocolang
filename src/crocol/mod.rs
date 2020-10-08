pub mod symbol;

pub use self::symbol::Codegen;
pub use self::symbol::LNodeResult;
pub use self::symbol::LSymbol;

pub mod utils;

use std::fs;
use std::rc::Rc;

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
use utils::{register_str_add_char, str_type, strip_ext};

#[derive(PartialEq)]
enum OutputFormat {
    LlvmIr,
    ObjectFile,
    Assembly,
    Executable,
}

pub struct Crocol {
    file_path: String,
    output_format: OutputFormat,
    output_flag: String,
    no_llvm_checks_flag: bool,
    verbose_flag: bool,
}

impl Default for Crocol {
    fn default() -> Self {
        Crocol::new()
    }
}

impl Crocol {
    pub fn new() -> Self {
        Crocol {
            file_path: String::new(),
            output_format: OutputFormat::Executable,
            output_flag: String::new(),
            no_llvm_checks_flag: false,
            verbose_flag: false,
        }
    }

    pub fn emit_assembly(&mut self) {
        self.output_format = OutputFormat::Assembly;
    }

    pub fn emit_object_file(&mut self) {
        self.output_format = OutputFormat::ObjectFile;
    }

    pub fn emit_llvm(&mut self) {
        self.output_format = OutputFormat::LlvmIr;
    }

    pub fn set_no_llvm_checks(&mut self, no_llvm_checks: bool) {
        self.no_llvm_checks_flag = no_llvm_checks;
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

        let ptr_size = context.ptr_sized_int_type(&target_machine.get_target_data(), None);

        let mut codegen = Codegen {
            context: &context,
            module,
            builder: context.create_builder(),
            symtable: SymTable::new(),
            str_type: str_type(&context, ptr_size),
            ptr_size,
            current_fn: main_fn,
        };

        // register built-in functions
        register_str_add_char(&codegen)?;

        if let Err(mut e) = tree.crocol(&mut codegen) {
            e.set_kind(CrocoErrorKind::Runtime);
            return Err(e);
        }

        // this should never fail if our nodes are right
        if !self.no_llvm_checks_flag {
            codegen.module.verify().unwrap();
        }

        // emit an executable if we don't specifically want to emit assembly, object files, llvm ir
        // get the llvm file output name
        let llvm_output_filename =
            if !self.output_flag.is_empty() && self.output_format != OutputFormat::Executable {
                self.output_flag.clone()
            } else {
                let ext = match self.output_format {
                    OutputFormat::Assembly | OutputFormat::Executable => "asm",
                    OutputFormat::ObjectFile => "o",
                    OutputFormat::LlvmIr => "ll",
                };
                format!("{}.{}", strip_ext(&self.file_path), ext)
            };

        // here we want to save the llvm ir
        if self.output_format == OutputFormat::LlvmIr {
            codegen
                .module
                .print_to_file(llvm_output_filename)
                .map_err(|_| {
                    CrocoError::from_type("cannot write llvm ir to disk", CrocoErrorKind::IO)
                })?;
            return Ok(());
        }

        let emit_method = match self.output_format {
            OutputFormat::Assembly => FileType::Assembly,
            _ => FileType::Object,
        };

        target_machine
            .write_to_file(&codegen.module, emit_method, llvm_output_filename.as_ref())
            .map_err(|e| {
                CrocoError::from_type(e.to_str().unwrap(), CrocoErrorKind::CompileTarget)
            })?;

        // if the user specified -S or -c, we're done here
        match self.output_format {
            OutputFormat::Assembly | OutputFormat::ObjectFile => return Ok(()),
            _ => (),
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
