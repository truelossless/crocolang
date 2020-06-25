use crate::token::CodePos;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub enum CrocoErrorKind {
    Unknown,
    IO,      // when a file failed to open
    Syntax,  // trown by the lexer
    Parse,   // thrown by the parser
    Runtime, // thrown by the interpreter
}

/// errors thrown by croco
pub struct CrocoError {
    kind: CrocoErrorKind,
    pos: CodePos,
    message: String,
}

impl CrocoError {
    pub fn new(pos: &CodePos, message: String) -> Self {
        CrocoError {
            kind: CrocoErrorKind::Unknown,
            pos: pos.clone(),
            message,
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
            CrocoErrorKind::Unknown => unreachable!(),
        };

        // get the line involved
        let mut lines = io::BufReader::new(File::open(&*self.pos.file).unwrap()).lines();
        let mut indicator = String::new();

        // we know that the line is present so just unwrap
        // we also have to unwrap the u32 -> usize conversion
        let mut errored_line = lines.nth(self.pos.line as usize).unwrap().unwrap();
        // newline are wrapped at the end of the line in our lexer
        errored_line += "\n";
        let errored_word = errored_line
            .split_word_bound_indices()
            .nth((self.pos.word) as usize)
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
            self.pos.file,
            self.pos.line + 1,  // lines start at 1
            errored_word.0 + 1  // cols start at 1
        )
    }
}

impl fmt::Debug for CrocoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
