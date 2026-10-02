use super::*;

#[test]
fn effect_witnesses_and_one_node_read_share_the_remaining_window() {
    let (_directory, _media, coordination) = super::tests::fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let page = 16 * 1024;
    let witnesses = 3 * std::mem::size_of::<SelectedArtifactSlice>() as u64;
    let required = witnesses + page;
    assert!(witnesses < required - 1 && page < required - 1);

    assert!(matches!(
        prepare_effect_slices(&window, 1, 2, page, required - 1),
        Err(Denial::BoundExceeded)
    ));

    let adequate = required + 128 * 1024;
    let slices = prepare_effect_slices(&window, 1, 2, page, adequate)
        .expect("same path and writes fit a funded resident window");
    assert!(slices.owned_heap_bytes().unwrap() >= witnesses);
    assert!(slices.owned_heap_bytes().unwrap() + page <= adequate);
}

#[test]
fn effect_witness_preflight_rejects_overflow_and_reserves_read_window_even_if_empty() {
    let (_directory, _media, coordination) = super::tests::fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    assert!(matches!(
        prepare_effect_slices(&window, usize::MAX, 1, 16 * 1024, u64::MAX),
        Err(Denial::BoundExceeded)
    ));

    // A V14 upsert has writes; zero count checks the read-window invariant.
    assert!(matches!(
        prepare_effect_slices(&window, 0, 0, 16 * 1024, 16 * 1024 - 1),
        Err(Denial::BoundExceeded)
    ));
    let empty = prepare_effect_slices(&window, 0, 0, 16 * 1024, 16 * 1024).unwrap();
    assert_eq!(empty.owned_heap_bytes(), Some(0));
}
