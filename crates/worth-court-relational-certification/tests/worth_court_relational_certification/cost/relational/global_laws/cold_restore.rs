//! An indirect source in another cold region must never become "no source".
use super::super::cold_checkpoint::{reopen, ColdCheckpointSource};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RelationalCheckpointArtifactPolicy,
    inspection::observe_relation_law_sources,
    transactions::{EntityReference, TransactionCommitError, WorkerIntentBatch},
};
use worth_testkit_relational::maintenance::relation_laws::{request, World};

#[test]
fn cold_cross_region_predecessor_is_demanded_before_a_disconnect_is_judged() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let mut world = World::across_partitions(&execution, 2);
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let source = ColdCheckpointSource::capture(&world.runtime, &execution, &checkpoint);
    world.runtime = reopen(
        &execution,
        &checkpoint,
        source.clone(),
        World::empty_runtime(0),
    );
    assert!(
        source.region_read_count() == 0,
        "setup must leave record regions cold"
    );
    let before = world
        .runtime
        .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
        .unwrap();
    let (result, observed) = observe_relation_law_sources(1024, || {
        world.commit(
            &execution,
            WorkerIntentBatch::new("cold-disconnect").push(world.delete(1)),
        )
    });
    let error = result.unwrap_err();
    match error {
        TransactionCommitError::Publication { error, .. } => {
            assert!(error.detail.contains("source_reaches_target"), "{error:?}")
        }
        other => panic!("a load/resource refusal cannot substitute for the law: {other:?}"),
    }
    assert_eq!(observed.omitted, 0);
    assert!(
        observed.witnesses.iter().any(|witness| witness
            .sources
            .contains(&EntityReference::Existing(world.nodes[0]))),
        "the cold indirect source must participate in the actual judged witness"
    );
    assert!(
        source.region_read_count() > 0,
        "the original request must demand the relevant regions"
    );
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
            .unwrap(),
        before
    );
}
