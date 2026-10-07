//! tree identity evidence for inherited retained state.
use super::*;
#[test]
fn a_child_edit_shares_every_tree_node_outside_its_edit_path() {
    let _guard = checkpoint_recovery_test_guard();
    const SIZE: usize = 32;
    let app = install(|graph| {
        facts::seed_set(graph, "even", -0.0);
        for number in 0..SIZE {
            seed_entry(
                graph,
                &["even"],
                number,
                RegionEntry {
                    id: number as u64,
                    region: number as u32,
                    value: 1.0,
                    work: 1,
                    fault: None,
                },
            );
        }
    });
    let parent = app.current_world();
    let parent_run = run(&app, parent).pop().unwrap();
    let child = fork(&app, parent);
    let advance = at_demand_scope(EntryEdit::new(
        "even",
        26,
        super::super::super::entry_edit::EntryFact::Value,
        3.25_f64.to_bits(),
    ));
    apply(&app, parent, vec![Change::Entry(advance)], &mut 0x6118);
    assert_eq!(run(&app, parent).last().unwrap().runs, [Run::Incremental]);
    let edit = at_demand_scope(EntryEdit::new(
        "even",
        7,
        super::super::super::entry_edit::EntryFact::Value,
        2.5_f64.to_bits(),
    ));
    apply(&app, child, vec![Change::Entry(edit)], &mut 0x6119);
    let child_run = run(&app, child).pop().unwrap();
    assert_eq!(child_run.runs, [Run::Incremental]);
    assert_eq!(
        child_run.calls,
        OwnerCalls {
            plans: 0,
            keys: 0,
            gathers: 1,
            kernels: 1
        }
    );
    let parent_state = parent_run.published.last().unwrap();
    let child_state = child_run.published.last().unwrap();
    let shared = parent_state.tree_node_sharing_with::<RegionKey, Entry, f64>(child_state);
    assert_eq!(shared.len(), SIZE);
    assert!(
        shared.iter().any(|(_, shared)| *shared),
        "a rebuild shares no node"
    );
    assert!(
        shared.iter().any(|(_, shared)| !*shared),
        "the changed leaf copies its root path"
    );
    let keys = shared.iter().map(|(key, _)| *key).collect::<Vec<_>>();
    let edited = worth_query_decl::facade::application_operation::application_computation_partition_identity(
        &RegionKey(7), &mut |_| Ok::<(), ()>(())).unwrap().partition();
    let path = super::super::tree_work::update_path(&keys, edited);
    for (key, shared) in shared {
        assert_eq!(
            shared,
            !path.contains(&key),
            "only the independently computed path changes: {key:?}"
        );
    }
    assert_eq!(
        run(&app, parent).len(),
        0,
        "the child's edit does not invalidate its parent"
    );
}
