//! Independent authored incoming references survive association, never destructive retirement.
use super::{
    super::fixture::{basis, capture, create, installed_with, locator, protected_payload},
    fixture::*,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::mvcc::{
    RelationalInstalledApplicationPublicationAuthority, RelationalPublicationOutcome,
    RelationalRequiredOutputPublication, RelationalTransactionIntent, RequiredOutputPublication,
    RequiredOutputRolePosture,
};
use crate::schema::data::*;
use crate::transactions::data::*;
use worth_execution::ExecutionRequest;

fn installed_references(
    version: u8,
) -> (
    crate::runtime::RelationalRuntime,
    crate::runtime::RelationalInitialSchemaInstallationReceipt,
) {
    let mut schema = registry(version);
    let relation = schema.relation_kinds.get_mut(&KindId(4)).unwrap();
    relation.relation_integrity = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: ContractId::new("native.reference.endpoints"),
            allowed_source_kinds: vec![KindId(1), KindId(3)],
            allowed_target_kinds: vec![KindId(3)],
            self_edges_allowed: false,
            cross_context_policy: crate::config::data::CrossContextPolicy::Forbid,
        }],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    installed_with(schema, Vec::new())
}

#[test]
fn independent_incoming_reference_survives_cold_association_and_preserve_but_blocks_retirement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_references(1);
    let world = world_with_installation(execution, runtime, receipt);
    let mut canonical_relations = world.relations;
    canonical_relations.sort_unstable();
    let payloads = world
        .outputs
        .map(|id| protected_payload(&world.runtime, id));
    let edges = world
        .relations
        .map(|id| relation_payload(&world.runtime, id));
    // This ordinary owner is created after the geometry publication, not adopted by it.
    let external = create(
        execution,
        &world.runtime,
        1,
        "independent-placement",
        "authored placement",
        None,
    );
    let reference = add_reference(execution, &world, external);
    let external_payload = protected_payload(&world.runtime, external);
    let reference_payload = relation_payload(&world.runtime, reference);
    super::refusals::complete_association(execution, &world).unwrap();
    let saved = capture(&world.runtime);
    let (mut cold, receipt) = installed_references(1);
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let previous = readmit(execution, &cold, &installed, &world.outputs, false);
    let original = previous.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(original.outputs.len(), 3);
    assert_eq!(
        original
            .relations
            .iter()
            .map(|edge| edge.id)
            .collect::<Vec<_>>(),
        canonical_relations
    );
    assert!(!original.outputs.contains(&external));
    assert!(!original.relations.iter().any(|edge| edge.id == reference));
    let born_at = original.birth;
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("native.closed.preserver", [20; 32]),
            previous.authority(),
        )
        .unwrap();
    let meaning = original.meaning.clone();
    let mut transaction = cold
        .begin_branch_transaction(&basis(&cold), RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("genuine-executed-no-data-preserve"),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction
        .validate(execution, &cold)
        .unwrap()
        .bind_required_output_publication(
            &installed,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor: previous,
                meaning,
                roles: crate::mvcc::native_roles_fixture(
                    &world.outputs.map(|id| (id, KindId(3))),
                    RequiredOutputRolePosture::Preserve,
                ),
            },
        )
        .unwrap();
    let candidate = cold
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = cold
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the genuine no-data Preserve must publish");
    };
    let committed = cold.settle_performed_publication(performed).unwrap();
    assert!(committed.changed_records.is_empty());
    assert_ne!(committed.commit.commit_id, born_at);
    let preserved = committed.required_output_publication(&installed).unwrap();
    let origin = preserved.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.birth, born_at);
    assert_eq!(origin.outputs, original_ids(&world.outputs));
    assert_eq!(
        origin
            .relations
            .iter()
            .map(|edge| edge.id)
            .collect::<Vec<_>>(),
        canonical_relations
    );
    cold.snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    readmit(execution, &cold, &installed, &world.outputs, true);
    for (id, payload) in world.outputs.iter().zip(&payloads) {
        assert_eq!(protected_payload(&cold, *id), *payload);
    }
    for (id, payload) in world.relations.iter().zip(&edges) {
        assert_eq!(relation_payload(&cold, *id), *payload);
    }
    assert_eq!(protected_payload(&cold, external), external_payload);
    assert_eq!(relation_payload(&cold, reference), reference_payload);

    // Cold readmission also authenticates the Preserve envelope, not merely the birth.
    let saved = capture(&cold);
    let (mut reopened, receipt) = installed_references(1);
    reopened
        .durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = reopened
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    readmit(execution, &reopened, &installed, &world.outputs, true);
    assert_eq!(protected_payload(&reopened, external), external_payload);
    assert_eq!(relation_payload(&reopened, reference), reference_payload);
    assert_retirement_refused(execution, &reopened, world.activation, external, reference);
}

fn original_ids(ids: &[EntityId; 3]) -> Vec<EntityId> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids
}

fn readmit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    ids: &[EntityId; 3],
    preserved: bool,
) -> RelationalRequiredOutputPublication {
    runtime
        .readmit_required_output_publication(
            execution,
            installed,
            &basis(runtime),
            if preserved {
                "native.closed.preserver"
            } else {
                "native.closed.producer"
            },
            if preserved { &[20; 32] } else { &[19; 32] },
            &crate::mvcc::native_meaning_fixture("native.closed.producer"),
            &crate::mvcc::native_roles_fixture(
                &ids.map(|id| (id, KindId(3))),
                if preserved {
                    RequiredOutputRolePosture::Preserve
                } else {
                    RequiredOutputRolePosture::Create
                },
            ),
        )
        .unwrap()
}

fn add_reference(
    execution: &ExecutionRequest<'_, '_>,
    world: &RelatedWorld,
    external: EntityId,
) -> RelationId {
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(4),
        client_key: crate::symbols::data::ClientKey::raw("independent-incoming-reference"),
        source: EntityReference::Existing(external),
        target: EntityReference::Existing(world.outputs[1]),
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
            WorkerIntentBatch::new("ordinary authored reference").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: edge.partition_id,
                    kind_id: edge.kind_id,
                    client_key: edge.client_key.clone(),
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    fields: AspectFieldPatch::from_locator(
                        locator("edge"),
                        worth_foundational::facade::AspectValue::String(
                            worth_foundational::facade::InternedString::from(
                                "independent incoming placement",
                            ),
                        ),
                    ),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &world.runtime).unwrap();
    assert!(committed
        .publication()
        .envelope
        .required_output_origin
        .is_none());
    let id = committed.created_relation(&edge).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    id
}

fn assert_retirement_refused(
    execution: &ExecutionRequest<'_, '_>,
    source: &crate::runtime::RelationalRuntime,
    activation: EntityId,
    external: EntityId,
    reference: RelationId,
) {
    let (mut target, _) = installed_references(2);
    let (_, mut recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(execution, &capture(source))
        .unwrap();
    let before = capture(&target);
    let selected = basis(&target);
    let external_payload = protected_payload(&target, external);
    let reference_payload = relation_payload(&target, reference);
    let denied = candidate(
        execution,
        &mut target,
        &mut recovered,
        activation,
        crate::mvcc::RelationalOperationControl::uninterrupted(),
        None,
    )
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::CodecClosureRejected
    );
    assert!(
        denied.detail().contains("purpose=codec-retirement"),
        "{denied:?}"
    );
    assert!(selected.is_current());
    assert!(recovered.codec_transition_pending());
    assert_eq!(capture(&target), before);
    assert_eq!(protected_payload(&target, external), external_payload);
    assert_eq!(relation_payload(&target, reference), reference_payload);
}
