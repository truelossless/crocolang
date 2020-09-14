use crate::token::CodePos;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub enum CrocoErrorKind {
    // global
    Unknown,
    IO,     // when a file failed to open
    Syntax, // thrown by the lexer
    Parse,  // thrown by the parser

    // crocoi-specific
    Runtime, // thrown by the interpreter

    // crocol-specific
    CompileTarget, // thrown when no compilation is possible on this target
    Malloc,        // thrown when the OS has no default allocator
    Linker,        // thrown when there isn't a linker available
}

/// errors thrown by croco
pub struct CrocoError {
    kind: CrocoErrorKind,
    pos: Option<CodePos>,
    message: String,
}

impl CrocoError {
    pub fn new(pos: &CodePos, message: impl AsRef<str>) -> Self {
        CrocoError {
            kind: CrocoErrorKind::Unknown,
            pos: Some(pos.clone()),
            message: message.as_ref().to_owned(),
        }
    }

    pub fn from_type(message: impl AsRef<str>, error_type: CrocoErrorKind) -> Self {
        CrocoError {
            kind: error_type,
            pos: None,
            message: message.as_ref().to_owned(),
        }
    }

    /// sets the kind of error, ONLY IF IT HASN'T BEEN SET BEFORE.
    pub fn set_kind(&mut self, kind: CrocoErrorKind) {
        if let CrocoErrorKind::Unknown = &self.kind {
            self.kind = kind;
        }
    }
}

impl fmt::Display for CrocoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_kind = match &self.kind {
            CrocoErrorKind::Syntax => "Syntax error",
            CrocoErrorKind::Parse => "Parse error",
            CrocoErrorKind::Runtime => "Runtime error",
            CrocoErrorKind::IO => "File error",
            CrocoErrorKind::CompileTarget => "Compile error",
            CrocoErrorKind::Malloc => "Allocation error",
            CrocoErrorKind::Linker => "Linker error",
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

        // format the errror message like this:
        // while (a)
        //
        // ^^^^^^^^
        // Syntax Error: unexpected token after while keyword
        // in mymod.croco at line 45
        write!(
            f,
            "\n{}\n{}\n\n{}: {}\nin {} at line {}, position {}\n",
            errored_line,
            indicator,
            error_kind,
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
