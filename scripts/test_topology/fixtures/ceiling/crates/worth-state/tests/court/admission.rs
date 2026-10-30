#[test]
fn admission_advances_generation() {
    assert_eq!(worth_state::next(0), 1);
}
