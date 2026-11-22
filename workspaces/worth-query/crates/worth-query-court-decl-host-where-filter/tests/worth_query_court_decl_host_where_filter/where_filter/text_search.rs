//! Journey 1: case-insensitive search across two text fields.
use super::*;

#[test]
fn text_search_matches_either_field_ignoring_ascii_case_within_the_scope_project() {
    let host = install(&board());
    // "flange" is in task 2's title and in tasks 1 ("FLANGE") and 5's notes.
    assert_eq!(
        keys(world::search(&host, search_text(1, "flange"), None)),
        [1, 2, 5]
    );
    assert_eq!(
        keys(world::search(&host, search_text(1, "MATTE"), None)),
        [3]
    );
    // Project 2's matching task never leaks into project 1 and vice versa.
    assert_eq!(
        keys(world::search(&host, search_text(2, "FLANGE"), None)),
        [6]
    );
    assert!(keys(world::search(&host, search_text(1, "gearbox"), None)).is_empty());
}
