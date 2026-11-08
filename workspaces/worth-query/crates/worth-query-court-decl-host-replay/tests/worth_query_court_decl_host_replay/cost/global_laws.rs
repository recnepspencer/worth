//! #97: the real Native law path is independent of unrelated N, H and catalogs.
use super::observation::{Dimensions, Phase, Sample};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    inspection::observe_maintenance_work, transactions::WorkerIntentBatch,
};
use worth_testkit_relational::maintenance::relation_laws::{request, World};

#[test]
fn global_law_fixed_edit_exposes_each_independent_growth_axis() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let samples = Dimensions {
        records: 128,
        history: 32,
        installed: 8,
    }
    .points()
    .map(|d| {
        let world = World::with_unrelated_kinds(&execution, d.records, d.installed as u32);
        world.add_hub_fanout(&execution, d.records);
        for step in 0..d.history {
            let committed = world
                .commit(
                    &execution,
                    WorkerIntentBatch::new("law-history")
                        .push(world.retarget(3, if step % 2 == 0 { 7 } else { 6 })),
                )
                .unwrap();
            world.release(&committed);
        }
        let (result, work) = observe_maintenance_work(64, || {
            world.commit(
                &execution,
                WorkerIntentBatch::new("fixed-law-edit").push(world.retarget(1, 3)),
            )
        });
        let result = result.unwrap();
        let mut sample = Sample::new(d);
        sample.observe_phase_work(&work);
        let counters = &result.execution().complexity_delta;
        assert!(
            counters.invariant_entity_slot_scans > 0,
            "the installed source law must run"
        );
        assert!(
            counters.invariant_relation_slot_scans > 0,
            "the law must traverse its actual successors"
        );
        sample.record(
            Phase::Invariants,
            "entity_scans",
            counters.invariant_entity_slot_scans as u64,
        );
        sample.record(
            Phase::Invariants,
            "relation_scans",
            counters.invariant_relation_slot_scans as u64,
        );
        sample.commits = 1;
        sample.touched.push(result.changed_records.len());
        world.release(&result);
        eprintln!("GLOBAL_LAW_SCALE {sample:?}");
        sample
    });
    super::ratchet::verify("global_laws", &samples);
}
