//! Distinct state tickets reconcile with independent full-lane metadata.
use super::*;
#[test]
fn child_publication_and_parent_deletion_keep_each_states_one_charge() {
    let _guard = checkpoint_recovery_test_guard();
    let model = Model::eviction(&mut Lcg(0x611_c057), TOTALS_WORK);
    let kept =
        installation::install_variant::<false, TOTALS_WORK, 1, 0>(None, Default::default(), |g| {
            model.seed(g)
        });
    let fresh =
        installation::install_variant::<false, TOTALS_WORK, 1, 3>(None, Default::default(), |g| {
            model.seed(g)
        });
    let initial = run(&kept, kept.current_world());
    let reference = run(&fresh, fresh.current_world());
    judge(&initial, &reference, None);
    let mut states = state_bytes(&initial);
    let mut reference_states = state_bytes(&reference);
    drop(reference);
    drop(initial);
    let parent = fork(&kept, kept.current_world());
    let full_parent = fork(&fresh, fresh.current_world());
    let value = |entry, bits| {
        at_demand_scope(EntryEdit::new(
            "even",
            entry,
            super::super::super::entry_edit::EntryFact::Value,
            bits,
        ))
    };
    apply(
        &kept,
        parent,
        vec![Change::Entry(value(0, 3.25_f64.to_bits()))],
        &mut 0x6111,
    );
    apply(
        &fresh,
        full_parent,
        vec![Change::Entry(value(0, 3.25_f64.to_bits()))],
        &mut 0x6111,
    );
    let retained = run(&kept, parent);
    let reference = run(&fresh, full_parent);
    judge(&retained, &reference, Some(Run::Incremental));
    reference_states += state_bytes(&reference);
    drop(reference);
    states += state_bytes(&retained);
    drop(retained);
    let child = fork(&kept, parent);
    let full_child = fork(&fresh, full_parent);
    apply(
        &kept,
        child,
        vec![Change::Entry(value(1, 2.5_f64.to_bits()))],
        &mut 0x6112,
    );
    apply(
        &fresh,
        full_child,
        vec![Change::Entry(value(1, 2.5_f64.to_bits()))],
        &mut 0x6112,
    );
    let retained = run(&kept, child);
    let reference = run(&fresh, full_child);
    judge(&retained, &reference, Some(Run::Incremental));
    reference_states += state_bytes(&reference);
    drop(reference);
    states += state_bytes(&retained);
    drop(retained);
    assert_eq!(
        kept.output_lineage_retained_bytes_for_test(),
        fresh.output_lineage_retained_bytes_for_test() - reference_states + states,
        "at child publication"
    );
    let _parent_cleanup = close_ancestor(&kept, parent);
    let _full_parent_cleanup = close_ancestor(&fresh, full_parent);
    assert_eq!(
        kept.output_lineage_retained_bytes_for_test(),
        fresh.output_lineage_retained_bytes_for_test() - reference_states + states,
        "deleted parent with a live child"
    );
}

pub(super) fn state_bytes(runs: &[OracleRun]) -> u64 {
    runs.iter()
        .flat_map(|run| &run.published)
        .map(|state| state.retained_bytes().unwrap())
        .sum()
}
