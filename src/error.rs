use crate::{symbol_type::SymbolType, token::CodePos};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
pub enum CrocoErrorKind {
    // global
    Unknown,
    /// Thrown when a file failed to open
    Io,
    /// Thrown by the lexer
    Syntax,
    /// Thrown by the parser
    Parse,
    /// Thrown when an error occurs at runtime
    Runtime,

    // crocol-specific
    /// Thrown when the compilation failed
    Compilation,
    /// Thrown when no compilation is possible on this target
    CompileTarget,
    /// Thrown when the OS has no default allocator  
    Malloc,
    /// Thrown when there isn't a linker available           
    Linker,
}

/// errors thrown by croco
pub struct CrocoError {
    kind: CrocoErrorKind,
    pub hint: Option<String>,
    pub pos: Option<CodePos>,
    pub message: String,
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
    pub fn add_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot add these two types together")
    }

    pub fn break_in_function_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot exit a function with a break")
    }

    pub fn cast_non_primitive_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "can only cast primitives together")
    }

    pub fn cast_redundant_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "redundant cast")
    }

    pub fn compare_different_types_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot compare different types")
    }

    pub fn compare_numbers_only_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "can compare only numbers")
    }

    pub fn condition_not_bool_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "expected a bool for the condition")
    }

    pub fn continue_in_function_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot use continue in a function")
    }

    pub fn dereference_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot dereference this variable")
    }

    pub fn divide_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot divide these two types together")
    }

    pub fn expected_value_got_early_return_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "expected a value but got an early-return keyword")
    }

    pub fn empty_array_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "do not use this syntax to declare empty arrays")
            .hint("use type annotations to declare empty arrays")
    }

    pub fn field_type_error(field_name: &str, code_pos: &CodePos) -> CrocoError {
        CrocoError::new(
            code_pos,
            format!("field {} is not of the right type", field_name),
        )
    }

    pub fn index_out_of_bounds_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "index out of bounds")
    }

    pub fn infer_error(code_pos: &CodePos, var_name: &str) -> CrocoError {
        CrocoError::new(
            code_pos,
            format!("cannot infer the type of the variable {}", var_name),
        )
    }

    pub fn invalid_return_value(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "expected a valid return value")
    }

    pub fn invert_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot invert something that isn't a boolean")
    }

    pub fn minus_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot substract these two types together")
    }

    pub fn mismatched_number_of_arguments_error(
        code_pos: &CodePos,
        decl_len: usize,
        args_len: usize,
    ) -> CrocoError {
        CrocoError::new(
            code_pos,
            format!(
                "mismatched number of arguments in function call\nExpected {} parameter{} but got {}",
                decl_len,
                if decl_len < 2 { "" } else { "s" },
                args_len
            ),
        )
    }

    pub fn mixed_type_array(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "array elements must be of the same type")
    }

    pub fn multiplicate_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot multiplicate these two types together")
    }

    pub fn negative_indexing_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot use a negative index")
    }

    pub fn no_field_error(field_name: &str, code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, format!("no field with the name {}", field_name))
    }

    pub fn parameter_error(code_pos: &CodePos, index: usize, is_method: bool) -> CrocoError {
        // if we have a method, we don't want to show the self parameter as a true parameter
        let errored_param = if is_method { index } else { index + 1 };

        CrocoError::new(
            code_pos,
            &format!(
                "parameter {} doesn't match function definition",
                errored_param,
            ),
        )
    }

    pub fn power_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot power these two types together")
    }

    pub fn tmp_value_borrow(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "trying to borrow a temporary value")
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

    pub fn unary_minus_error(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "cannot negate this type of variable")
    }

    pub fn wrong_return(
        fn_ty: Option<&SymbolType>,
        ret_ty: Option<&SymbolType>,
        code_pos: &CodePos,
    ) -> CrocoError {
        match (fn_ty, ret_ty) {
            (Some(fn_ty), Some(ret_ty)) => CrocoError::new(
                code_pos,
                format!("function should return {} but returned {}", fn_ty, ret_ty),
            ),

            (None, Some(fn_ty)) => CrocoError::new(
                code_pos,
                format!("function shouldn't return anything but returned {}", fn_ty),
            ),

            (Some(ret_ty), None) => CrocoError::new(
                code_pos,
                format!(
                    "function should return {} but didn't return anything",
                    ret_ty
                ),
            ),
            _ => unreachable!(),
        }
    }

    pub fn wrong_type_indexing(code_pos: &CodePos) -> CrocoError {
        CrocoError::new(code_pos, "only arrays are indexable")
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
            CrocoErrorKind::Compilation => "Compilation error",
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
        // in mymod.croco:45:6
        write!(
            f,
            "\n{errored_line}\n{indicator}\n\n{error_kind}: {error_message}{hint}\n\nIn file {file_name}:{line_number}:{col_number}\n",
            errored_line = errored_line,
            indicator = indicator,
            error_kind = error_kind,
            error_message = self.message,
            hint = hint,
            file_name = pos.file,
            line_number = pos.line + 1,         // lines start at 1
            col_number = errored_word.0 + 1     // cols start at 1
        )
    }
}

impl fmt::Debug for CrocoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
