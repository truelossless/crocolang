// References tests

use croco::interpreter::Interpreter;

#[test]
fn it_assigns_correctly() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter
        .exec_file("tests/references/it_is_assigned_correctly/basic_reference.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/references/it_is_assigned_correctly/mut_reference.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/references/it_is_assigned_correctly/deref_ref.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/references/it_is_assigned_correctly/deref_ref_err.croco")
        .is_err());
}
