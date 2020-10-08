// References tests

use croco::Crocoi;

#[test]
fn it_assigns_correctly() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/references/it_assigns_correctly/basic_reference.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/references/it_assigns_correctly/mut_reference.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/references/it_assigns_correctly/correct_mut.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/references/it_assigns_correctly/deref_ref.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/references/it_assigns_correctly/deref_ref_err.croco")
        .is_err());
}

#[test]
fn it_auto_dereferences() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/references/it_auto_dereferences/auto_dereference.croco")
        .is_ok());
}
