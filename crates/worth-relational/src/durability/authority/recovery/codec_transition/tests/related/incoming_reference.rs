//! Owned boundary edges and independent references preserve distinct native custody.
use super::{
    super::fixture::{basis, capture, create, locator, protected_payload},
    fixture::*,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::mvcc::{
    RelationalInstalledApplicationPublicationAuthority, RelationalPublicationOutcome,
    RelationalRequiredOutputPublication, RelationalTransactionIntent, RequiredOutputPublication,
    RequiredOutputRolePosture,
};
use crate::transactions::data::*;
use worth_execution::ExecutionRequest;
mod authoring;
use authoring::add_reference;
pub(super) use authoring::installed_references;

#[test]
fn owned_boundary_and_independent_references_survive_cold_preserve_but_block_retirement() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let (runtime, receipt) = installed_references(1);
    let (world, boundary) = world_with_boundary(execution, runtime, receipt);
    let (boundary, publisher) = boundary.unwrap();
    let warm_birth = publisher.custody;
    let mut canonical_relations = vec![world.relations[0], world.relations[1], boundary];
    canonical_relations.sort_unstable();
    assert_eq!(
        warm_birth
            .envelope
            .required_output_origin
            .as_ref()
            .unwrap()
            .relations
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        canonical_relations
    );
    let frame_payload = protected_payload(&world.runtime, world.authored);
    let boundary_payload = relation_payload(&world.runtime, boundary);
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
    let reference = add_reference(
        execution,
        &world,
        external,
        world.outputs[1],
        "independent-incoming",
    );
    let independent_out = add_reference(
        execution,
        &world,
        world.outputs[1],
        external,
        "independent-outgoing",
    );
    let independent_out_payload = relation_payload(&world.runtime, independent_out);
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
    assert!(!original.outputs.contains(&world.authored));
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
    let new_boundary = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(4),
        client_key: crate::symbols::data::ClientKey::raw("actual-new-boundary-during-preserve"),
        source: EntityReference::Existing(world.outputs[2]),
        target: EntityReference::Existing(world.authored),
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("genuine-preserve-with-boundary-birth").push(
                MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                    partition_id: new_boundary.partition_id,
                    kind_id: new_boundary.kind_id,
                    client_key: new_boundary.client_key.clone(),
                    source: new_boundary.source.clone(),
                    target: new_boundary.target.clone(),
                    fields: AspectFieldPatch::from_locator(
                        locator("edge"),
                        worth_foundational::facade::AspectValue::String(
                            worth_foundational::facade::InternedString::from(
                                "newly owned frame link",
                            ),
                        ),
                    ),
                })),
            ),
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
        panic!("the genuine Preserve with a new boundary birth must publish");
    };
    let committed = cold.settle_performed_publication(performed).unwrap();
    let new_boundary = committed.created_relation(&new_boundary).unwrap();
    assert_eq!(
        committed.changed_records,
        vec![RecordRef::Relation(new_boundary)]
    );
    canonical_relations.push(new_boundary);
    canonical_relations.sort_unstable();
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
    let new_entry = origin
        .relations
        .iter()
        .find(|entry| entry.id == new_boundary)
        .unwrap();
    assert_eq!(new_entry.birth, committed.commit.commit_id);
    assert!(new_entry.before_write.is_none());
    assert_eq!(new_entry.after_write, committed.commit.version_id);
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
    assert_eq!(protected_payload(&cold, world.authored), frame_payload);
    assert_eq!(relation_payload(&cold, boundary), boundary_payload);
    let new_boundary_payload = relation_payload(&cold, new_boundary);
    assert_eq!(protected_payload(&cold, external), external_payload);
    assert_eq!(relation_payload(&cold, reference), reference_payload);
    assert_eq!(
        relation_payload(&cold, independent_out),
        independent_out_payload
    );

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
    assert_eq!(protected_payload(&reopened, world.authored), frame_payload);
    assert_eq!(relation_payload(&reopened, boundary), boundary_payload);
    assert_eq!(
        relation_payload(&reopened, new_boundary),
        new_boundary_payload
    );
    let current = readmit(execution, &reopened, &installed, &world.outputs, true);
    assert_eq!(
        current
            .envelope
            .required_output_origin
            .as_ref()
            .unwrap()
            .relations
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        canonical_relations
    );
    assert_eq!(protected_payload(&reopened, external), external_payload);
    assert_eq!(relation_payload(&reopened, reference), reference_payload);
    assert_eq!(
        relation_payload(&reopened, independent_out),
        independent_out_payload
    );
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
