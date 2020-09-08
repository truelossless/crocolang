// Primitive tests

use croco::Crocoi;

#[test]
fn it_assigns_correct_values() {
    let mut interpreter = Crocoi::new();

    // num assignment
    assert!(interpreter
        .exec_file("tests/primitives/it_assigns_correct_values/num_assignment.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/primitives/it_assigns_correct_values/num_default_assignment.croco")
        .is_ok());

    // str assignment
    assert!(interpreter
        .exec_file("tests/primitives/it_assigns_correct_values/str_assignment.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/primitives/it_assigns_correct_values/str_default_assignment.croco")
        .is_ok());

    // bool assignment
    assert!(interpreter
        .exec_file("tests/primitives/it_assigns_correct_values/bool_assignment.croco")
        .is_ok());
    assert!(interpreter
        .exec_file("tests/primitives/it_assigns_correct_values/bool_default_assignment.croco")
        .is_ok());
}

#[test]
fn it_shadows_correctly() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/primitives/it_shadows_correctly/shadow_err.croco")
        .is_err());

    assert!(interpreter
        .exec_file("tests/primitives/it_shadows_correctly/shadow_ok.croco")
        .is_ok());
}

#[test]
fn it_does_not_change_type() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/primitives/it_does_not_change_type/str_to_num_err.croco")
        .is_err());

    assert!(interpreter
        .exec_file("tests/primitives/it_does_not_change_type/num_to_bool_err.croco")
        .is_err());
}

#[test]
fn it_calculates_correctly() {
    let mut interpreter = Crocoi::new();

    assert!(interpreter
        .exec_file("tests/primitives/it_calculates_correctly/priority.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/primitives/it_calculates_correctly/floating_point.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/primitives/it_calculates_correctly/unary.croco")
        .is_ok());

    assert!(interpreter
        .exec_file("tests/primitives/it_calculates_correctly/parenthesis.croco")
        .is_ok());
}
