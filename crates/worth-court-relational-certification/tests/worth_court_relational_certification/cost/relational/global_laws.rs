//! Global laws consume the change-selected source witness, not a model scan.
mod cold_restore;
mod equality;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    inspection::observe_maintenance_work, transactions::WorkerIntentBatch,
};
use worth_testkit_relational::maintenance::{
    assert_change_bounded,
    relation_laws::{request, World},
};

#[test]
fn fixed_law_edit_is_flat_in_unrelated_components() {
    let small = measure(128, false);
    let large = measure(256, false);
    eprintln!("GLOBAL_LAWS components 128:{small:?} 256:{large:?}");
    for (name, a, b) in [
        ("source preparation charged work", small.0, large.0),
        ("law entity visits", small.1, large.1),
        ("law relation visits", small.2, large.2),
    ] {
        assert!(a > 0 && b > 0, "the real law must execute: {name}");
        assert_change_bounded(name, "unrelated components", a, b);
    }
}
fn measure(components: usize, hub_fanout: bool) -> (u64, u64, u64) {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let world = World::new(&execution, components);
    if hub_fanout {
        world.add_hub_fanout(&execution, components);
    }
    let (result, work) = observe_maintenance_work(64, || {
        world.commit(
            &execution,
            WorkerIntentBatch::new("fixed-law-edit").push(world.retarget(1, 3)),
        )
    });
    let result = result.unwrap();
    assert_eq!(work.omitted_phases, 0);
    let preparation = work
        .phases
        .iter()
        .filter(|row| row.phase == "relation_source_witness")
        .map(|row| {
            assert!(row.accounted);
            row.charged_work
        })
        .sum();
    let counters = &result.execution().complexity_delta;
    let visits = (
        preparation,
        counters.invariant_entity_slot_scans as u64,
        counters.invariant_relation_slot_scans as u64,
    );
    world.release(&result);
    visits
}

#[test]
fn fixed_law_edit_ignores_unrelated_kinds_sharing_its_endpoint() {
    let small = measure(512, true);
    let large = measure(1024, true);
    eprintln!("GLOBAL_LAW_SHARED_HUB 512:{small:?} 1024:{large:?}");
    assert!(small.0 > 0 && large.0 > 0);
    assert!(small.1 > 0 && large.1 > 0 && small.2 > 0 && large.2 > 0);
    assert_change_bounded(
        "shared-endpoint source preparation",
        "unrelated kind fanout",
        small.0,
        large.0,
    );
    assert_change_bounded(
        "shared-endpoint law entity visits",
        "unrelated kind fanout",
        small.1,
        large.1,
    );
    assert_change_bounded(
        "shared-endpoint law relation visits",
        "unrelated kind fanout",
        small.2,
        large.2,
    );
}
