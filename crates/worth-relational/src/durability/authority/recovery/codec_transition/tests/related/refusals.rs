//! Unowned internal edges, rewritten owned edges and changed ordering keys stay fenced.
use super::{
    super::fixture::{basis, capture, locator, protected_payload},
    fixture::*,
};
use crate::identity::data::{KindId, PartitionId};
use crate::mvcc::{RelationalOperationControl, RelationalTransactionIntent};
use crate::transactions::data::*;

#[test]
fn unowned_internal_and_rewritten_owned_relations_refuse_without_any_retirement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for variant in 0..3 {
        let world = world(execution);
        complete_association(execution, &world).unwrap();
        let none = crate::history::data::derive_codec_retirement_closure(
            execution,
            &world.runtime,
            &basis(&world.runtime),
            &[],
        )
        .unwrap();
        assert!(none.entities.is_empty());
        let intent = if variant == 0 {
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(4),
                client_key: crate::symbols::data::ClientKey::raw("unowned-internal"),
                source: EntityReference::Existing(world.outputs[1]),
                target: EntityReference::Existing(world.outputs[2]),
                fields: AspectFieldPatch::from_locator(
                    locator("edge"),
                    worth_foundational::facade::AspectValue::String(
                        worth_foundational::facade::InternedString::from("unowned internal edge"),
                    ),
                ),
            }))
        } else {
            MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                UpdateRelationEndpointsIntent {
                    relation_id: world.relations[0],
                    kind_id: KindId(4),
                    source: EntityReference::Existing(
                        world.outputs[if variant == 1 { 0 } else { 1 }],
                    ),
                    target: EntityReference::Existing(
                        world.outputs[if variant == 1 { 1 } else { 2 }],
                    ),
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
        let source_before = capture(&world.runtime);
        let source_basis = basis(&world.runtime);
        let denied = complete_association(execution, &world).unwrap_err();
        assert!(
            format!("{denied:?}").contains("incident relation"),
            "{denied:?}"
        );
        assert!(source_basis.is_current());
        assert_eq!(capture(&world.runtime), source_before);
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
            None,
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
        // The rewire has already lawfully changed ordinary ordering. Refusal must
        // preserve that exact committed graph (including its index), not undo it.
        if variant != 2 {
            assert_order(&target, &world.outputs, &world.relations);
        }
        assert_eq!(
            world.relations.map(|id| relation_payload(&target, id)),
            world
                .relations
                .map(|id| relation_payload(&world.runtime, id)),
        );
    }
}

pub(super) fn complete_association(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &RelatedWorld,
) -> Result<(), crate::history::data::CodecRetirementClosureStop> {
    crate::history::data::validate_codec_output_correspondence(
        execution,
        &world.runtime,
        &basis(&world.runtime),
        "native.closed.producer",
        &[19; 32],
        &crate::mvcc::native_meaning_fixture("native.closed.producer"),
        &crate::mvcc::native_roles_fixture(
            &world.outputs.map(|id| (id, KindId(3))),
            crate::mvcc::RequiredOutputRolePosture::Create,
        ),
    )
    .map(|_| ())
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
