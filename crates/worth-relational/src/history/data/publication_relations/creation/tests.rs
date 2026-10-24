//! Real retention exercises Created-endpoint evidence without synthesizing a branch root.
use super::*;
use crate::mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent};
use crate::publication::patch::data::{
    PublishedAuthoritativePatch, PublishedAuthoritativePatchOperation as Op,
};
use crate::storage::substrate::RelationRecordKind;
use crate::symbols::data::ClientKey;
use crate::tests::support::{
    create_entity, relation_source_aspect, relation_target_aspect, release_test_commit_snapshot,
    test_owner_begin_transaction_for_main, AspectSchemaFixture,
};
use crate::transactions::data::*;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_foundational::facade::AspectBinding;

#[test]
fn trimmed_birth_uses_created_endpoints_and_refuses_forged_or_missing_evidence() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let schema = AspectSchemaFixture {
        relation_aspects: vec![relation_source_aspect(), relation_target_aspect()],
        ..AspectSchemaFixture::default()
    };
    let mut runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .profile(crate::config::data::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema.build_registry())
        .durability_mode(crate::durability::data::DurabilityMode::InMemoryCanonical)
        .build();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            crate::schema::data::RelationalSchemaRegistry::new(),
            Vec::new(),
        )
        .unwrap();
    let left = create_entity(execution, &runtime, "external-left");
    let right = create_entity(execution, &runtime, "external-right");
    let output = CreatedEntityRef {
        partition_id: left.partition_id,
        kind_id: KindId(1),
        client_key: ClientKey::raw("output"),
    };
    let incidental = CreatedRelationRef {
        partition_id: left.partition_id,
        kind_id: KindId(2),
        client_key: ClientKey::raw("incidental"),
        source: EntityReference::Existing(left),
        target: EntityReference::Existing(right),
    };
    let owned = CreatedRelationRef {
        client_key: ClientKey::raw("owned"),
        source: EntityReference::Created(output.clone()),
        target: EntityReference::Existing(left),
        ..incidental.clone()
    };
    let mut batch = WorkerIntentBatch::new("real producer with incidental reference").push(
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: output.partition_id,
            kind_id: output.kind_id,
            client_key: output.client_key.clone(),
            fields: AspectFieldPatch::default(),
        })),
    );
    for edge in [&incidental, &owned] {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: edge.partition_id,
                kind_id: edge.kind_id,
                client_key: edge.client_key.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                fields: AspectFieldPatch::default(),
            },
        )));
    }
    let (owner, installed) = receipt.issue_application_publication_owner();
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut publication = crate::mvcc::native_birth_fixture(
        owner.admit_required_output("native.trimmed.producer", [19; 32]),
        vec![output.clone()],
    );
    if let crate::mvcc::RequiredOutputPublication::Birth {
        absence, meaning, ..
    } = &mut publication
    {
        *absence = runtime
            .readmit_required_output_route_absence(execution, &installed, &basis, meaning)
            .unwrap();
        assert!(absence.is_some());
    }
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, &runtime)
        .unwrap()
        .bind_required_output_publication(&installed, publication)
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("real producer birth must publish");
    };
    let birth = runtime.settle_performed_publication(performed).unwrap();
    let id = birth.created_relation(&incidental).unwrap();
    let owned_id = birth.created_relation(&owned).unwrap();
    let output_id = birth.created_entity(&output).unwrap();
    let birth_envelope = Arc::clone(&birth.publication().envelope);
    let origin = birth_envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.relations.len(), 1);
    assert_eq!(origin.relations[0].id, owned_id);
    assert!(!origin.relations.iter().any(|row| row.id == id));

    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("real incidental endpoint rewrite").push(
                MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                    UpdateRelationEndpointsIntent {
                        relation_id: id,
                        kind_id: KindId(2),
                        source: EntityReference::Existing(output_id),
                        target: EntityReference::Existing(right),
                    },
                )),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let rewritten = transaction.commit(execution, &runtime).unwrap();
    let write_version = rewritten.commit.version_id;
    let write_envelope = Arc::clone(&rewritten.publication().envelope);
    assert!(write_envelope.required_output_origin.is_none());
    // Release snapshots before exercising the actual storage retention owner.
    release_test_commit_snapshot(&runtime, &rewritten);
    release_test_commit_snapshot(&runtime, &birth);
    runtime
        .storage_authority()
        .trim_live_history::<RelationRecordKind>(
            BTreeMap::from([(id.partition_id, BTreeSet::from([id.slot_index()]))]),
            write_version,
        );
    let edition = runtime.acquire_partition_edition();
    let partition = edition.partition(id.partition_id).unwrap();
    let history = partition
        .relation_arena
        .metadata_history_at(id.slot_index())
        .unwrap();
    assert!(
        history
            .iter()
            .all(|row| row.effective_at != birth_envelope.commit.version_id),
        "native birth metadata must actually be unavailable in the tested storage edition"
    );
    assert_eq!(
        history.get(history.len() - 1).unwrap().effective_at,
        write_version
    );
    assert_eq!(
        partition
            .relation_arena
            .created_at_for_slot(id.slot_index()),
        Some(birth_envelope.commit.version_id)
    );
    let row = partition.relation_arena.get(&id).unwrap();
    assert_eq!(row.extra().endpoints.as_ref().unwrap().source, output_id);
    assert_eq!(row.extra().endpoints.as_ref().unwrap().target, right);
    assert!(run(
        execution,
        &edition,
        id,
        origin,
        &birth_envelope,
        &write_envelope,
        false
    )
    .is_ok());

    // Hostile envelope twins are inputs to the owner court, never issued custody.
    let mut forged_origin: CanonicalRequiredOutputOrigin =
        rmp_serde::from_slice(&rmp_serde::to_vec(origin.as_ref()).unwrap()).unwrap();
    let mut claim = origin.relations[0].clone();
    claim.id = id;
    claim.source = output_id;
    claim.target = right;
    forged_origin.relations.push(claim);
    forged_origin.relations.sort_unstable_by_key(|row| row.id);
    let mut forged = birth_envelope.as_ref().clone();
    forged.required_output_origin = Some(Arc::new(forged_origin));
    let denied = run(
        execution,
        &edition,
        id,
        origin,
        &forged,
        &write_envelope,
        true,
    )
    .unwrap_err();
    assert!(
        matches!(denied, Stop::Payload(ref detail) if detail.contains("birth roster differs from native allocation/endpoints")),
        "{denied:?}"
    );

    let mut contradictory_endpoints = birth_envelope.as_ref().clone();
    contradictory_endpoints
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .relation_birth
        .as_mut()
        .unwrap()
        .source = output_id;
    assert!(
        run(
            execution,
            &edition,
            id,
            origin,
            &contradictory_endpoints,
            &write_envelope,
            false
        )
        .is_err(),
        "structural birth and exposed endpoint operations must agree"
    );
    let mut missing = birth_envelope.as_ref().clone();
    let patch = missing
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap();
    let operations = patch
        .authoritative_patch
        .full_grammar_operations()
        .iter()
        .filter(|operation| {
            !matches!(
                operation,
                Op::WholeAspectSet {
                    binding: AspectBinding::RelationSourceEndpoint
                        | AspectBinding::RelationTargetEndpoint,
                    ..
                }
            )
        })
        .cloned()
        .collect();
    patch.authoritative_patch = PublishedAuthoritativePatch::new(operations);
    assert!(
        patch.relation_birth.is_some(),
        "actual native creation publishes structural birth independently of optional aspects"
    );
    assert!(
        run(
            execution,
            &edition,
            id,
            origin,
            &missing,
            &write_envelope,
            false
        )
        .is_ok(),
        "retired metadata plus absent endpoint aspects still has the actual structural birth"
    );
    let mut contradictory = missing.clone();
    contradictory
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .relation_birth
        .as_mut()
        .unwrap()
        .kind = KindId(999);
    assert!(
        run(
            execution,
            &edition,
            id,
            origin,
            &contradictory,
            &write_envelope,
            false
        )
        .is_err(),
        "a contradictory native structural kind cannot support custody"
    );
    missing
        .patch
        .authoritative_record_patches
        .iter_mut()
        .find(|patch| patch.target == RecordRef::Relation(id))
        .unwrap()
        .relation_birth = None;
    let denied = run(
        execution,
        &edition,
        id,
        origin,
        &missing,
        &write_envelope,
        false,
    )
    .unwrap_err();
    assert!(
        matches!(denied, Stop::Payload(ref detail) if detail.contains("actual relation creation endpoints unavailable")),
        "{denied:?}"
    );
}

fn run(
    execution: &ExecutionRequest<'_, '_>,
    state: &dyn PartitionAccess,
    id: RelationId,
    selected: &CanonicalRequiredOutputOrigin,
    birth: &CanonicalCommitEnvelope,
    write: &Arc<CanonicalCommitEnvelope>,
    owned: bool,
) -> Result<(), Stop> {
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    let mut lookup = |version| {
                        if version == birth.commit.version_id {
                            Some(Arc::new(birth.clone()))
                        } else if version == write.commit.version_id {
                            Some(Arc::clone(write))
                        } else {
                            None
                        }
                    };
                    if owned {
                        validate_owned(state, id, selected, &mut lookup, context)
                    } else {
                        validate(state, id, selected, None, &mut lookup, context)
                    }
                },
            )
        })
        .unwrap()
        .unwrap()
        .0
}
