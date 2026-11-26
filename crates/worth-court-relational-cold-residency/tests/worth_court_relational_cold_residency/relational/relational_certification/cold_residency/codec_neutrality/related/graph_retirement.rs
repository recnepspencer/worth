//! Native owners retire and regenerate an entire closed producer graph.
use super::{
    super::{
        fixture::{basis, capture, protected_payload},
        retained_records::RetainedRecords,
        serial_request,
    },
    fixture::*,
};
use worth_relational::facade::{indexes::DerivedIndexEntries, mvcc::RelationalOperationControl};

#[test]
fn closed_three_outputs_two_relations_retire_atomically_and_regenerate_ordered_v2() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    assert_order(execution, &world.runtime, &world.outputs, &world.relations);
    assert_values(
        execution,
        &world.runtime,
        &world.outputs,
        [
            "codec1:nondefault-0",
            "codec1:nondefault-1",
            "codec1:nondefault-2",
        ],
    );
    let old_basis = basis(&world.runtime);
    let old_index = world
        .runtime
        .index_access()
        .read_latest_generation(execution, ORDER, old_basis.descriptor().branch_id())
        .unwrap()
        .unwrap();
    let source = protected_payload(execution, &world.runtime, world.authored);
    assert!(!source.is_empty());
    let original = RetainedRecords::capture(execution, &world.runtime);
    let payloads = world.outputs.map(|id| original.entity(id).1);
    let edges = world.relations.map(|id| original.relation(id).1);
    let checkpoint = capture(execution, &world.runtime);
    let checkpoint_bytes = checkpoint.bytes().to_vec();
    let (mut target, mut recovered, receipt) = recover(execution, &world);
    assert!(target
        .readmit_recovered_initial_schema_installation(receipt.clone())
        .is_err());
    let candidate = candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        RelationalOperationControl::uninterrupted(),
    )
    .unwrap();
    let committed = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .unwrap();
    let (commit, recovered) = committed.into_parts();
    let target_receipt = target
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let current = basis(&target);
    let retired = RetainedRecords::capture(execution, &target);
    for (position, id) in world.outputs.iter().enumerate() {
        assert!(recovered.codec_transition_retires(*id));
        let (live, payload) = retired.entity(*id);
        assert_eq!(payload, payloads[position]);
        assert!(!live);
        assert!(original.entity(*id).0);
    }
    for (position, id) in world.relations.iter().enumerate() {
        let (live, payload) = retired.relation(*id);
        assert_eq!(payload, edges[position]);
        assert!(!live);
        assert!(original.relation(*id).0);
    }
    assert_eq!(
        protected_payload(execution, &target, world.authored),
        source
    );
    let retired_index = target
        .index_access()
        .read_latest_generation(execution, ORDER, current.descriptor().branch_id())
        .unwrap()
        .unwrap();
    assert_eq!(retired_index.source_commit_id, commit.commit_id);
    assert_ne!(retired_index.generation_id, old_index.generation_id);
    assert!(
        matches!(&retired_index.entries, DerivedIndexEntries::RelatedEntityOrdering(entries) if entries.is_empty())
    );
    recovered.admit_basis(current).unwrap();
    super::retired_route::cold_retired_route(execution, &target);
    let (outputs, relations) = produce(execution, &target, &target_receipt, 2);
    for id in outputs {
        assert!(!world.outputs.contains(&id));
    }
    super::super::fixture::build_index(execution, &target);
    assert_order(execution, &target, &outputs, &relations);
    assert_values(
        execution,
        &target,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    let regenerated = RetainedRecords::capture(execution, &target);
    let v2_payloads = outputs.map(|id| regenerated.entity(id).1);
    assert_ne!(v2_payloads, payloads);
    let saved = capture(execution, &target);
    let (mut reopened, receipt) = installed(2);
    reopened
        .durability_recovery()
        .restore_native_checkpoint_with_authority(
            execution,
            &saved,
            worth_testkit_relational::cold_recovery::members_source(&saved),
        )
        .unwrap();
    reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_order(execution, &reopened, &outputs, &relations);
    assert_values(
        execution,
        &reopened,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    assert_eq!(
        protected_payload(execution, &reopened, world.authored),
        source
    );
    let reopened_records = RetainedRecords::capture(execution, &reopened);
    for (position, id) in world.outputs.iter().enumerate() {
        assert_eq!(reopened_records.entity(*id).1, payloads[position]);
    }
    for (position, id) in world.relations.iter().enumerate() {
        assert_eq!(reopened_records.relation(*id).1, edges[position]);
    }
    assert_eq!(checkpoint.bytes(), checkpoint_bytes);
    assert_order(execution, &world.runtime, &world.outputs, &world.relations);
    assert_eq!(
        old_index.entries,
        world
            .runtime
            .index_access()
            .read_latest_generation(execution, ORDER, old_basis.descriptor().branch_id())
            .unwrap()
            .unwrap()
            .entries
    );
}
