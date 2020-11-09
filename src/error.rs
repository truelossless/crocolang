use crate::token::CodePos;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub enum CrocoErrorKind {
    // global
    Unknown,
    Io,      // when a file failed to open
    Syntax,  // thrown by the lexer
    Parse,   // thrown by the parser
    Runtime, // thrown when an error occurs at runtime

    // crocol-specific
    CompilationError, // thrown when the compilation failed
    CompileTarget,    // thrown when no compilation is possible on this target
    Malloc,           // thrown when the OS has no default allocator
    Linker,           // thrown when there isn't a linker available
}

/// errors thrown by croco
pub struct CrocoError {
    kind: CrocoErrorKind,
    hint: Option<String>,
    pos: Option<CodePos>,
    message: String,
}

impl CrocoError {
    pub fn new(pos: &CodePos, message: impl AsRef<str>) -> Self {
        CrocoError {
            kind: CrocoErrorKind::Unknown,
            hint: None,
            pos: Some(pos.clone()),
            message: message.as_ref().to_owned(),
        }
    }

    pub fn from_type(message: impl AsRef<str>, error_type: CrocoErrorKind) -> Self {
        CrocoError {
            kind: error_type,
            hint: None,
            pos: None,
            message: message.as_ref().to_owned(),
        }
    }

    /// Sets the kind of error if it wasn't set before
    pub fn set_kind_if_unknown(&mut self, kind: CrocoErrorKind) {
        if let CrocoErrorKind::Unknown = &self.kind {
            self.kind = kind;
        }
    }

    /// Sets the kind of error
    pub fn set_kind(&mut self, kind: CrocoErrorKind) {
        self.kind = kind;
    }

    pub fn hint(mut self, hint: impl AsRef<str>) -> Self {
        self.hint = Some(hint.as_ref().to_owned());
        self
    }

    // convenient error constructors to avoid code reuse across backends
    pub fn cast_non_primitive_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "can only cast primitives together")
    }

    pub fn cast_redundant_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "redundant cast")
    }

    pub fn expected_value_got_early_return_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "expected a value but got an early-return keyword")
    }

    pub fn infer_error(code_pos: &CodePos, var_name: &str) -> CrocoError {
        CrocoError::new(
            code_pos,
            format!("cannot infer the type of the variable {}", var_name),
        )
    }

    pub fn type_annotation_error(code_pos: &CodePos, var_name: &str) -> CrocoError {
        CrocoError::new(
            code_pos,
            format!(
                "variable {} has been explicitely given a type but is declared with another one",
                var_name
            ),
        )
    }

    pub fn type_change_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot change the type of a variable")
    }
}

impl fmt::Display for CrocoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_kind = match &self.kind {
            CrocoErrorKind::Syntax => "Syntax error",
            CrocoErrorKind::Parse => "Parse error",
            CrocoErrorKind::Runtime => "Runtime error",
            CrocoErrorKind::Io => "File error",
            CrocoErrorKind::CompileTarget => "Compile error",
            CrocoErrorKind::Malloc => "Allocation error",
            CrocoErrorKind::Linker => "Linker error",
            CrocoErrorKind::CompilationError => "Compilation error",
            CrocoErrorKind::Unknown => unreachable!(),
        };

        // for some errors just print a minimal message
        if self.pos.is_none() {
            return write!(f, "\n{}: {}", error_kind, self.message);
        }

        let pos = self.pos.as_ref().unwrap();

        // get the line involved
        let mut lines = io::BufReader::new(File::open(&*pos.file).unwrap()).lines();
        let mut indicator = String::new();

        // we know that the line is present so just unwrap
        // we also have to unwrap the u32 -> usize conversion
        let mut errored_line = lines.nth(pos.line as usize).unwrap().unwrap();
        // newline are wrapped at the end of the line in our lexer
        errored_line += "\n";
        let errored_word = errored_line
            .split_word_bound_indices()
            .nth((pos.word) as usize)
            .unwrap();

        let lower_bound = errored_word.0;
        let upper_bound = errored_word.0 + errored_word.1.len();

        for _ in 0..lower_bound {
            indicator += " ";
        }

        for _ in lower_bound..upper_bound {
            indicator += "^";
        }

        // if we have only one character to highlight make an arrow to see it better
        if errored_word.1.len() == 1 {
            indicator += "---"
        }

        let hint = match &self.hint {
            Some(hint) => format!("\nHint: {}", hint),

            None => String::new(),
        };

        // format the errror message like this:
        // while (a)
        //
        // ^^^^^^^^
        // Syntax Error: unexpected token after while keyword
        // Hint: do not use parenthesis
        // in mymod.croco at line 45
        write!(
            f,
            "\n{}\n{}\n\n{}: {}{}\n\nIn file {}:{}:{}\n",
            errored_line,
            indicator,
            error_kind,
            hint,
            self.message,
            pos.file,
            pos.line + 1,       // lines start at 1
            errored_word.0 + 1  // cols start at 1
        )
    }
}

impl fmt::Debug for CrocoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
