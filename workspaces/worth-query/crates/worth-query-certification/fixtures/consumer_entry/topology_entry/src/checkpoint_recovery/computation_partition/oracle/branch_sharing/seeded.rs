//! seeded evidence for inherited retained state.
use super::*;
#[test]
fn seeded_forks_switches_and_deleted_ancestors_equal_full_computation() {
    let _guard = checkpoint_recovery_test_guard();
    let model = prefix::model();
    let kept =
        installation::install_variant::<false, TOTALS_WORK, 1, 0>(None, Default::default(), |g| {
            model.seed(g)
        });
    let fresh =
        installation::install_variant::<false, TOTALS_WORK, 1, 3>(None, Default::default(), |g| {
            model.seed(g)
        });
    let (mut parent_model, parent_runs) = prefix::run(&kept);
    let (fresh_model, full_parent) = prefix::run(&fresh);
    assert!(parent_model == fresh_model);
    judge(&parent_runs, &full_parent, None);
    // The ancestor we later close owns a real forked Native branch. The
    // installed default remains available for application authentication.
    let parent = fork(&kept, kept.current_world());
    let full_parent = fork(&fresh, fresh.current_world());
    let step = parent_model.step(Kind::Value, &mut Lcg(0x6110));
    apply(
        &kept,
        parent,
        step.changes.iter().map(clone_change).collect(),
        &mut 0x610f,
    );
    apply(&fresh, full_parent, step.changes, &mut 0x610f);
    judge(
        &run(&kept, parent),
        &run(&fresh, full_parent),
        Some(Run::Incremental),
    );
    let child = fork(&kept, parent);
    let full_child = fork(&fresh, full_parent);
    let nested = fork(&kept, child);
    let full_nested = fork(&fresh, full_child);
    let mut models = [parent_model.clone(), parent_model.clone(), parent_model];
    let mut rng = Lcg(0x611_ba51_5eed);
    let mut command = [0x6110, 0x6110];
    let branches = [parent, child, nested];
    let full_branches = [full_parent, full_child, full_nested];
    // The parent advances before either child runs. Local-only lookup and
    // a parent's later head both lose the inherited unchanged calls here.
    for (index, kind) in [
        (0, Kind::Value),
        (1, Kind::SharedWeight),
        (2, Kind::ItemKey),
    ] {
        let step = models[index].step(kind, &mut rng);
        apply(
            &kept,
            branches[index],
            step.changes.iter().map(clone_change).collect(),
            &mut command[0],
        );
        apply(&fresh, full_branches[index], step.changes, &mut command[1]);
        judge(
            &run(&kept, branches[index]),
            &run(&fresh, full_branches[index]),
            Some(Run::Incremental),
        );
    }
    // Every semantic input of the existing edit alphabet is exercised on
    // independently evolving branch snapshots, including absence/failure,
    // membership and item digests, full keys, routing, and work ceilings.
    let kinds = [
        Kind::Input,
        Kind::Value,
        Kind::SharedWeight,
        Kind::ItemKey,
        Kind::Create,
        Kind::Delete,
        Kind::NewKey,
        Kind::EmptyKey,
        Kind::Swap,
        Kind::Fault,
        Kind::Repair,
        Kind::Ceiling,
        Kind::Relief,
        Kind::NoOp,
        Kind::OwnWrite,
        Kind::BlindWrite,
        Kind::DeleteThenCreate,
    ];
    for round in 0..2 {
        let mut order = kinds;
        // Fault/repair and ceiling/relief remain adjacent so the exact named
        // stop is tested, and the next unrelated edit has completed state.
        for place in (1..8).rev() {
            let other = rng.below(place + 1);
            order.swap(place, other);
        }
        for kind in order {
            let index = rng.below(branches.len());
            let index = if matches!(kind, Kind::Repair | Kind::Relief) {
                0
            } else {
                index
            };
            let index = if matches!(kind, Kind::Fault | Kind::Ceiling) {
                0
            } else {
                index
            };
            let step = models[index].step(kind, &mut rng);
            apply(
                &kept,
                branches[index],
                step.changes.iter().map(clone_change).collect(),
                &mut command[0],
            );
            apply(&fresh, full_branches[index], step.changes, &mut command[1]);
            let a = with_own_write(step.own_write, || run(&kept, branches[index]));
            let b = with_own_write(step.own_write, || run(&fresh, full_branches[index]));
            judge(&a, &b, None);
            if let Some(write) = step.own_write {
                models[index].written(write);
            }
            // Switching to an unedited sibling answers from that sibling's
            // current output, without importing this branch's latest head.
            let sibling = (index + 1) % branches.len();
            judge(
                &run(&kept, branches[sibling]),
                &run(&fresh, full_branches[sibling]),
                None,
            );
        }
        assert_eq!(
            command[0], command[1],
            "round {round}: both worlds received each edit"
        );
    }
    let _child_cleanup = close_ancestor(&kept, child);
    let _full_child_cleanup = close_ancestor(&fresh, full_child);
    let step = models[2].step(Kind::Value, &mut rng);
    apply(
        &kept,
        nested,
        step.changes.iter().map(clone_change).collect(),
        &mut command[0],
    );
    apply(&fresh, full_nested, step.changes, &mut command[1]);
    judge(&run(&kept, nested), &run(&fresh, full_nested), None);
    let _parent_cleanup = close_ancestor(&kept, parent);
    let _full_parent_cleanup = close_ancestor(&fresh, full_parent);
    let step = models[2].step(Kind::Value, &mut rng);
    apply(
        &kept,
        nested,
        step.changes.iter().map(clone_change).collect(),
        &mut command[0],
    );
    apply(&fresh, full_nested, step.changes, &mut command[1]);
    judge(&run(&kept, nested), &run(&fresh, full_nested), None);
}
