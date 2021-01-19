// Primitive tests

use crate::ALL_BACKENDS;

use super::{test_file_err, test_file_ok, CROCOI};

#[test]
fn it_assigns_correct_values() {
    // num assignment
    test_file_ok(
        "tests/primitives/it_assigns_correct_values/num_assignment.croco",
        ALL_BACKENDS,
    );
    test_file_ok(
        "tests/primitives/it_assigns_correct_values/num_default_assignment.croco",
        ALL_BACKENDS,
    );

    // str assignment
    test_file_ok(
        "tests/primitives/it_assigns_correct_values/str_assignment.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/primitives/it_assigns_correct_values/str_default_assignment.croco",
        ALL_BACKENDS,
    );

    // bool assignment
    test_file_ok(
        "tests/primitives/it_assigns_correct_values/bool_assignment.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/primitives/it_assigns_correct_values/bool_default_assignment.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_shadows_correctly() {
    test_file_err(
        "tests/primitives/it_shadows_correctly/shadow_err.croco",
        CROCOI,
    );

    test_file_ok(
        "tests/primitives/it_shadows_correctly/shadow_ok.croco",
        CROCOI,
    );
}

#[test]
fn it_does_not_change_type() {
    test_file_err(
        "tests/primitives/it_does_not_change_type/str_to_num_err.croco",
        ALL_BACKENDS,
    );

    test_file_err(
        "tests/primitives/it_does_not_change_type/num_to_bool_err.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_calculates_correctly() {
    test_file_ok(
        "tests/primitives/it_calculates_correctly/priority.croco",
        CROCOI,
    );
    test_file_ok(
        "tests/primitives/it_calculates_correctly/floating_point.croco",
        CROCOI,
    );
    test_file_ok(
        "tests/primitives/it_calculates_correctly/unary.croco",
        CROCOI,
    );
    test_file_ok(
        "tests/primitives/it_calculates_correctly/parenthesis.croco",
        CROCOI,
    );
}
