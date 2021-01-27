use crate::{test_file_err, test_file_ok, ALL_BACKENDS};

// Struct tests

#[test]
fn it_is_declared_correctly() {
    test_file_ok(
        "tests/structs/it_is_declared_correctly/declaration.croco",
        ALL_BACKENDS,
    );

    test_file_err(
        "tests/structs/it_is_declared_correctly/declaration_err.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_is_instanciated_correctly() {
    test_file_ok(
        "tests/structs/it_is_instanciated_correctly/instanciation.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/structs/it_is_instanciated_correctly/nested_instanciation.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_returns_correct_field_values() {
    test_file_ok(
        "tests/structs/it_returns_correct_field_values/field_value.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/structs/it_returns_correct_field_values/default_value.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_sets_correct_field_values() {
    test_file_ok(
        "tests/structs/it_sets_correct_field_values/set_field_value.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/structs/it_sets_correct_field_values/set_multiple_field_values.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_does_not_change_field_type() {
    test_file_err(
        "tests/structs/it_does_not_change_field_type/num_to_str_err.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_calls_methods_correctly() {
    test_file_ok(
        "tests/structs/it_calls_methods_correctly/basic_method.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/structs/it_calls_methods_correctly/self_method.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/structs/it_calls_methods_correctly/self_mut_method.croco",
        ALL_BACKENDS,
    );
}
