// Array tests

use croco::interpreter::Interpreter;
#[test]
fn it_can_be_created() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter
        .exec_file("tests/arrays/it_can_be_created/basic_creation.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/arrays/it_can_be_created/struct_creation.croco")
        .is_ok());
}

#[test]
fn it_does_not_change_type() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter
        .exec_file("tests/arrays/it_does_not_change_type/num_and_str_err.croco")
        .is_err());

    assert!(interpreter
        .exec_file("tests/arrays/it_does_not_change_type/struct_err.croco")
        .is_err());
}

#[test]
fn it_is_indexable() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter
        .exec_file("tests/arrays/it_is_indexable/basic_index.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/arrays/it_is_indexable/out_of_bounds_err.croco")
        .is_err());
}
