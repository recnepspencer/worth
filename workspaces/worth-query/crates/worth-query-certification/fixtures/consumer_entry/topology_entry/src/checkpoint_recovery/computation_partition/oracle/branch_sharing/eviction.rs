//! eviction evidence for inherited retained state.
use super::*;
#[test]
fn a_child_eviction_is_local_and_does_not_release_its_parents_state() {
    let _guard = checkpoint_recovery_test_guard();
    let mut rng = Lcg(0x611_e71c_5eed);
    let model = Model::eviction(&mut rng, TOTALS_WORK);
    let kept =
        installation::install_variant::<false, TOTALS_WORK, 1, 0>(None, Default::default(), |g| {
            model.seed(g)
        });
    let fresh =
        installation::install_variant::<false, TOTALS_WORK, 1, 3>(None, Default::default(), |g| {
            model.seed(g)
        });
    let parent = kept.current_world();
    let full_parent = fresh.current_world();
    judge(&run(&kept, parent), &run(&fresh, full_parent), None);
    let child = fork(&kept, parent);
    let full_child = fork(&fresh, full_parent);
    let extra = LARGEST_SET - 16 - model.len();
    let prime = EntryEdit::create(&["even"], 1000, 1000, 1.0_f64.to_bits(), 1).batch(extra);
    let mut commands = [0x611e, 0x611e];
    apply(
        &kept,
        child,
        vec![Change::Entry(prime.clone())],
        &mut commands[0],
    );
    apply(
        &fresh,
        full_child,
        vec![Change::Entry(prime)],
        &mut commands[1],
    );
    let a = kept
        .with_available_lineage_bytes_for_test(256 * 1024, || run(&kept, child))
        .unwrap();
    let b = fresh
        .with_available_lineage_bytes_for_test(256 * 1024, || run(&fresh, full_child))
        .unwrap();
    judge(&a, &b, Some(Run::Incremental));
    let mut child_model = model.clone();
    let step = child_model.step(Kind::Value, &mut rng);
    let mut changes = vec![Change::Entry(EntryEdit::delete(1000).batch(extra))];
    changes.extend(step.changes);
    apply(
        &kept,
        child,
        changes.iter().map(clone_change).collect(),
        &mut commands[0],
    );
    apply(&fresh, full_child, changes, &mut commands[1]);
    judge(
        &run(&kept, child),
        &run(&fresh, full_child),
        Some(Run::Full(Cause::Evicted)),
    );
    let mut parent_model = model;
    let step = parent_model.step(Kind::Value, &mut rng);
    apply(
        &kept,
        parent,
        step.changes.iter().map(clone_change).collect(),
        &mut commands[0],
    );
    apply(&fresh, full_parent, step.changes, &mut commands[1]);
    judge(
        &run(&kept, parent),
        &run(&fresh, full_parent),
        Some(Run::Incremental),
    );
}
