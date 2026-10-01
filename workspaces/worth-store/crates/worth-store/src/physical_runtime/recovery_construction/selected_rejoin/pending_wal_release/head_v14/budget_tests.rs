use super::*;

#[test]
fn effect_witnesses_and_one_node_read_share_the_remaining_window() {
    let page = 16 * 1024;
    let witnesses = 3 * std::mem::size_of::<SelectedArtifactSlice>() as u64;
    let required = witnesses + page;
    assert!(witnesses < required - 1 && page < required - 1);

    assert!(matches!(
        prepare_effect_slices(1, 2, page, required - 1),
        Err(Denial::BoundExceeded)
    ));

    let adequate = required + 128 * 1024;
    let slices = prepare_effect_slices(1, 2, page, adequate)
        .expect("same path and writes fit a combined resident window");
    assert!(slices.capacity() >= 3);
    assert!(effect_slice_heap(&slices).unwrap() + page <= adequate);
}

#[test]
fn effect_witness_preflight_rejects_overflow_and_reserves_read_window_even_if_empty() {
    assert!(matches!(
        prepare_effect_slices(usize::MAX, 1, 16 * 1024, u64::MAX),
        Err(Denial::BoundExceeded)
    ));

    // A V14 upsert has writes; this zero-count case checks the helper's
    // read-window invariant independently of a mutation's authority shape.
    assert!(matches!(
        prepare_effect_slices(0, 0, 16 * 1024, 16 * 1024 - 1),
        Err(Denial::BoundExceeded)
    ));
    let empty = prepare_effect_slices(0, 0, 16 * 1024, 16 * 1024).unwrap();
    assert!(empty.is_empty());
}
