// If tests

use croco::interpreter::Interpreter;

#[test]
fn it_matches_if_properly() {
    let mut interpreter = Interpreter::new();

    assert!(interpreter
        .exec_file("tests/conditions/it_matches_if_properly/if.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/conditions/it_matches_if_properly/elif.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/conditions/it_matches_if_properly/else.croco")
        .is_ok());
}
