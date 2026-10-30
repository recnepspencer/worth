#[test]
fn settlement_advances_generation() {
    assert_eq!(worth_state::next(2), 3);
}
