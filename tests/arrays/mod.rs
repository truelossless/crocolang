use crate::{test_file_err, test_file_ok, ALL_BACKENDS};

// Array tests

#[test]
fn it_can_be_created() {
    test_file_ok(
        "tests/arrays/it_can_be_created/basic_creation.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/arrays/it_can_be_created/struct_creation.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_does_not_change_type() {
    test_file_err(
        "tests/arrays/it_does_not_change_type/num_and_str_err.croco",
        ALL_BACKENDS,
    );

    test_file_err(
        "tests/arrays/it_does_not_change_type/struct_err.croco",
        ALL_BACKENDS,
    );
}

#[test]
fn it_is_indexable() {
    test_file_ok(
        "tests/arrays/it_is_indexable/basic_index.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/arrays/it_is_indexable/variable_index.croco",
        ALL_BACKENDS,
    );

    test_file_err(
        "tests/arrays/it_is_indexable/out_of_bounds_err.croco",
        ALL_BACKENDS,
    );

    test_file_err(
        "tests/arrays/it_is_indexable/negative_err.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/arrays/it_is_indexable/multi_dimensional.croco",
        ALL_BACKENDS,
    );
}
