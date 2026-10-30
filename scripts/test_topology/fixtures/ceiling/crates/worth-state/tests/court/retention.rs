#[test]
fn retention_advances_generation() {
    assert_eq!(worth_state::next(1), 2);
}
