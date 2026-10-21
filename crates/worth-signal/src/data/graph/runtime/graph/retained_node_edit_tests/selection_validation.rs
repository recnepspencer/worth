use super::*;

#[test]
fn cumulative_selection_validates_every_node_before_editing() {
    let arena = prepared_arena();
    let maximum = representation_charge(&arena);
    for indices in [&[][..], &[0, 0], &[1, 0], &[0, 2]] {
        assert!(arena
            .prepare_retained_node_edits(
                &test_ledger(),
                indices,
                Some(maximum),
                &mut Work::new(Some(10_000)),
                |_| panic!("invalid selection must fail before callback")
            )
            .is_err());
    }
}
