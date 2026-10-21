//! Closed performed native graph retirement precedes changed-codec consumers.
mod codec_disposition;
mod consequences;
mod fixture;
mod incoming_reference;
mod refusals;
mod settlement;
use super::fixture::{basis, capture, protected_payload};
use crate::indexes::data::DerivedIndexEntries;
use crate::mvcc::RelationalOperationControl;
use fixture::*;

#[test]
fn closed_three_outputs_two_relations_retire_atomically_and_regenerate_ordered_v2() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    assert_order(&world.runtime, &world.outputs, &world.relations);
    assert_values(
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
        .indexes
        .latest_generation(ORDER, Some(old_basis.descriptor().branch_id()))
        .unwrap();
    let source = protected_payload(&world.runtime, world.authored);
    assert!(!source.is_empty());
    let payloads = world
        .outputs
        .map(|id| protected_payload(&world.runtime, id));
    let edges = world
        .relations
        .map(|id| relation_payload(&world.runtime, id));
    let checkpoint = capture(&world.runtime);
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
        None,
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
    for (position, id) in world.outputs.iter().enumerate() {
        assert!(recovered.codec_transition_retires(*id));
        assert_eq!(protected_payload(&target, *id), payloads[position]);
        assert!(!current
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .entity_arena
            .get(id)
            .unwrap()
            .is_live());
        assert!(old_basis
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .entity_arena
            .get(id)
            .unwrap()
            .is_live());
    }
    for (position, id) in world.relations.iter().enumerate() {
        assert_eq!(relation_payload(&target, *id), edges[position]);
        assert!(!current
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .relation_arena
            .get(id)
            .unwrap()
            .is_live());
        assert!(old_basis
            .inner
            .root
            .partition_state(id.partition_id)
            .unwrap()
            .relation_arena
            .get(id)
            .unwrap()
            .is_live());
    }
    assert_eq!(protected_payload(&target, world.authored), source);
    let retired_index = target
        .indexes
        .latest_generation(ORDER, Some(current.descriptor().branch_id()))
        .unwrap();
    assert_eq!(retired_index.source_commit_id, commit.commit_id);
    assert_ne!(retired_index.generation_id, old_index.generation_id);
    assert!(
        matches!(&retired_index.entries, DerivedIndexEntries::RelatedEntityOrdering(entries) if entries.is_empty())
    );
    recovered.admit_basis(current).unwrap();
    codec_disposition::cold_retired_route(execution, &target);
    let (outputs, relations) = produce(execution, &target, &target_receipt, 2);
    for id in outputs {
        assert!(!world.outputs.contains(&id));
    }
    super::fixture::build_index(execution, &target);
    assert_order(&target, &outputs, &relations);
    assert_values(
        &target,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    let v2_payloads = outputs.map(|id| protected_payload(&target, id));
    assert_ne!(v2_payloads, payloads);
    let saved = capture(&target);
    let (mut reopened, receipt) = installed(2);
    reopened
        .durability_recovery()
        .restore_native_checkpoint_with_authority(execution, &saved)
        .unwrap();
    reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    assert_order(&reopened, &outputs, &relations);
    assert_values(
        &reopened,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    assert_eq!(protected_payload(&reopened, world.authored), source);
    for (position, id) in world.outputs.iter().enumerate() {
        assert_eq!(protected_payload(&reopened, *id), payloads[position]);
    }
    for (position, id) in world.relations.iter().enumerate() {
        assert_eq!(relation_payload(&reopened, *id), edges[position]);
    }
    assert_eq!(checkpoint.bytes(), checkpoint_bytes);
    assert_order(&world.runtime, &world.outputs, &world.relations);
    assert_eq!(
        old_index.entries,
        world
            .runtime
            .indexes
            .latest_generation(ORDER, Some(old_basis.descriptor().branch_id()))
            .unwrap()
            .entries
    );
}
