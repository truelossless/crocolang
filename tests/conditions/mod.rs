// If tests

use croco::Crocoi;

#[test]
fn it_matches_if_properly() {
    let mut interpreter = Crocoi::new();

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
