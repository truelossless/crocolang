// If tests

use crate::{test_file_ok, ALL_BACKENDS};

#[test]
fn it_matches_if_properly() {
    test_file_ok(
        "tests/conditions/it_matches_if_properly/if.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/conditions/it_matches_if_properly/elif.croco",
        ALL_BACKENDS,
    );

    test_file_ok(
        "tests/conditions/it_matches_if_properly/else.croco",
        ALL_BACKENDS,
    )
}
