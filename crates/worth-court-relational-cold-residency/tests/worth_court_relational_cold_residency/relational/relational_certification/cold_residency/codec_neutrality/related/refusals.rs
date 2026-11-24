//! Unowned internal edges, rewritten owned edges and changed ordering keys stay fenced.
use super::super::publication_fixture;
use super::incoming_reference::relation_payload;
use super::{
    super::fixture::{basis, capture, locator, protected_payload},
    fixture::*,
};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::mvcc::RelationalOperationControl;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::mvcc::{
    RelationalInstalledApplicationPublicationAuthority, RequiredOutputRolePosture,
};
use worth_relational::facade::transactions::*;

#[test]
fn unowned_internal_and_rewritten_owned_relations_refuse_without_any_retirement() {
    let request = super::super::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    for variant in 0..3 {
        let (runtime, receipt) = installed(1);
        let (world, publisher) = world_with_publication(execution, runtime, receipt);
        complete_association(execution, &world, &publisher.installed).unwrap();
        let intent = if variant == 0 {
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(4),
                client_key: worth_relational::facade::symbols::ClientKey::raw("unowned-internal"),
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
        let source_before = capture(execution, &world.runtime);
        let source_basis = basis(&world.runtime);
        let denied = complete_association(execution, &world, &publisher.installed).unwrap_err();
        assert!(
            format!("{denied:?}").contains("incident relation"),
            "{denied:?}"
        );
        assert!(world
            .runtime
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&source_basis)
            .is_ok());
        assert_eq!(capture(execution, &world.runtime), source_before);
        let (mut target, mut recovered, _) = recover(execution, &world);
        let before = capture(execution, &target);
        let selected = basis(&target);
        let source = protected_payload(execution, &target, world.authored);
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
            worth_relational::facade::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
        );
        assert!(denied.detail().contains("incident relation"), "{denied:?}");
        assert!(target
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&selected)
            .is_ok());
        assert!(recovered.codec_transition_pending());
        assert_eq!(capture(execution, &target), before);
        assert_eq!(
            protected_payload(execution, &target, world.authored),
            source
        );
        // The rewire has already lawfully changed ordinary ordering. Refusal must
        // preserve that exact committed graph (including its index), not undo it.
        if variant != 2 {
            assert_order(execution, &target, &world.outputs, &world.relations);
        }
        assert_eq!(
            world
                .relations
                .map(|id| relation_payload(execution, &target, id)),
            world
                .relations
                .map(|id| relation_payload(execution, &world.runtime, id)),
        );
    }
}

pub(super) fn complete_association(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &RelatedWorld,
    installed: &RelationalInstalledApplicationPublicationAuthority,
) -> Result<(), TransactionCommitError> {
    world
        .runtime
        .readmit_required_output_publication(
            execution,
            installed,
            &basis(&world.runtime),
            "native.closed.producer",
            &[19; 32],
            &publication_fixture::meaning("native.closed.producer"),
            &publication_fixture::roles(
                &world.outputs.map(|id| (id, KindId(3))),
                RequiredOutputRolePosture::Create,
            ),
        )
        .map(|_| ())
}
