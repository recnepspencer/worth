//! Real external edges, later equal writes and changed ordering keys stay fenced.
use super::{
    super::fixture::{basis, capture, locator, protected_payload},
    fixture::*,
};
use crate::identity::data::{KindId, PartitionId};
use crate::mvcc::{RelationalOperationControl, RelationalTransactionIntent};
use crate::transactions::data::*;

#[test]
fn external_endpoint_and_later_equal_relation_write_refuse_without_any_retirement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for external in [true, false] {
        let world = world(execution);
        let intent = if external {
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(4),
                client_key: crate::symbols::data::ClientKey::raw("external-source"),
                source: EntityReference::Existing(world.authored),
                target: EntityReference::Existing(world.outputs[1]),
                fields: AspectFieldPatch::from_locator(
                    locator("edge"),
                    worth_foundational::facade::AspectValue::String(
                        worth_foundational::facade::InternedString::from("protected external edge"),
                    ),
                ),
            }))
        } else {
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: world.relations[0],
                    kind_id: KindId(4),
                    source: EntityReference::Existing(world.outputs[0]),
                    target: EntityReference::Existing(world.outputs[1]),
                },
            ))
        };
        let mut transaction = world
            .runtime
            .begin_branch_transaction(
                &basis(&world.runtime),
                RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("unowned relation consequence").push(intent),
                execution.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(execution, &world.runtime).unwrap();
        world
            .runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
        let (mut target, mut recovered, _) = recover(execution, &world);
        let before = capture(&target);
        let selected = basis(&target);
        let source = protected_payload(&target, world.authored);
        let denied = candidate(
            execution,
            &mut target,
            &mut recovered,
            world.activation,
            RelationalOperationControl::uninterrupted(),
        )
        .unwrap_err();
        assert_eq!(
            denied.kind(),
            crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
        );
        assert!(denied.detail().contains("incident relation"), "{denied:?}");
        assert!(selected.is_current());
        assert!(recovered.codec_transition_pending());
        assert_eq!(capture(&target), before);
        assert_eq!(protected_payload(&target, world.authored), source);
        assert_order(&target, &world.outputs, &world.relations);
    }
}

#[test]
fn changed_ordering_key_meaning_refuses_before_destination_restore() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let world = world(execution);
    // Replace the actual registered definition with the changed payload key,
    // rather than manufacturing a transition-permission declaration.
    world
        .runtime
        .index_authority()
        .register(crate::indexes::data::DerivedIndexDefinition {
            index_id: crate::indexes::data::DerivedIndexId(99),
            name: "changed derived ordering".into(),
            branch_scoped: true,
            kind: crate::indexes::data::DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: KindId(4),
                parent_endpoint: crate::indexes::data::RelatedEntityEndpoint::SourceParent,
                child_kind: KindId(3),
                ordering: vec![crate::indexes::data::RelatedEntityOrderingField::new(
                    locator("derived"),
                    crate::indexes::data::RelatedEntityOrderingDirection::Ascending,
                )],
            },
        });
    let (mut target, _) = installed(2);
    let before = capture(&target);
    let owner = target.runtime_instance_id();
    let denial = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap_err();
    assert!(
        matches!(denial, crate::durability::data::RelationalRecoveryStop::Durability(error)
        if error.class == crate::durability::data::RecoveryFailureClass::SchemaMismatch && error.detail.contains("ordering"))
    );
    assert_eq!(target.runtime_instance_id(), owner);
    assert_eq!(capture(&target), before);
}
