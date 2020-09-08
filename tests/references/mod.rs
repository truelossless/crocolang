// References tests

use croco::Crocoi;

#[test]
fn it_assigns_correctly() {
    let mut interpreter = Crocoi::new();

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
