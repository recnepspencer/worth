//! Immutable physical-linkage predicate inputs, never an admitted runtime world.
use super::*;
use crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test as envelope_with_patch_and_update;
use crate::identity::data::{EntityId, PartitionId};
use crate::transactions::data::primitives::RecordRef;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;
use worth_relational_contracts::internal_port::history::data::canonical_record_allocation::CanonicalRecordAllocationFixturePort as _;
use worth_relational_contracts::internal_port::lineage::data::artifacts::published_artifact::PublishedLineageArtifactFixturePort as _;
use worth_relational_contracts::internal_port::lineage::data::events::assemble_lineage_event_record;
use worth_relational_contracts::internal_port::lineage::data::events::LineageEventRecordParts;

#[test]
fn equal_envelope_bytes_cannot_replace_exact_physical_catalog_custody() {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let envelope = Arc::new(envelope_with_patch_and_update(
        RecordRef::Entity(entity),
        entity,
    ));
    let descriptor = RelationalBranchRootDescriptor::new([7; 32], [8; 32]);
    let axes = RelationalBranchRootAxes {
        storage_version: 1,
        storage_root: *descriptor.truth_root(),
        schema_root: *descriptor.schema_root(),
        correctness_index: RelationalRootCorrectnessIndex::AuthoritativeFallback,
        visibility: RelationalBranchVisibilityCommitment::for_root(
            &envelope,
            *descriptor.truth_root(),
            *descriptor.schema_root(),
            RelationalRootCorrectnessIndex::AuthoritativeFallback,
        ),
    };
    let mut root = (*RelationalBranchRoot::empty()).clone();
    root.committed = Some(RelationalCommittedBranchRoot {
        commit_change: None,
        descriptor,
        evidence: RelationalRootCommitEvidence::original_for_request(
            execution,
            Arc::clone(&envelope),
        )
        .unwrap(),
        axes,
        publication_cost: Default::default(),
    });
    let root = Arc::new(root);
    let artifact = crate::history::commit::RelationalCommitArtifact::from_envelope_with_root(
        Arc::clone(&envelope),
        Arc::clone(&root),
    )
    .unwrap();
    let mut distinct_custody = (*root).clone();
    distinct_custody.committed.as_mut().unwrap().evidence =
        RelationalRootCommitEvidence::original_for_request(
            execution,
            Arc::new(envelope.independent_fixture_image()),
        )
        .unwrap();
    let distinct_custody = Arc::new(distinct_custody);
    assert!(artifact.links_root(&root));
    assert!(!artifact.links_root(&distinct_custody));
    assert!(!distinct_custody.links_envelope(&envelope));
    // The differing pointer is the only changed predicate input. No runtime,
    // publication, checkpoint or operational admission is claimed here.
    assert_eq!(root.descriptor(), distinct_custody.descriptor());
}

#[test]
fn recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks() {
    use crate::history::commit::{RelationalCommitArtifact, RelationalCommitArtifactDenial};
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let partition_id = crate::identity::data::PartitionId::main();
    let (partition, entities, _) =
        crate::storage::overlay::small_relation_partition::relation_partition_for_test(
            partition_id,
            1,
            &[],
        );
    let entity = entities[0];
    let registry = crate::schema::data::RelationalSchemaRegistry::new()
        .register_entity_kind(crate::schema::data::EntityKindRegistration {
            kind_id: crate::identity::data::KindId(1),
            kind_name: "artifact-entity".to_owned(),
            schema_id: crate::schema::data::SchemaId("artifact".to_owned()),
            schema_version_id: crate::schema::data::SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let mut envelope =
        crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test(
            crate::transactions::data::primitives::RecordRef::Entity(entity),
            entity,
        );
    envelope.body_mut_for_test().schema_authority = registry.authority_snapshot();
    let envelope = Arc::new(envelope);
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(entity.slot_index());
    let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
        &std::collections::BTreeMap::from([(partition_id, (partition.clone(), journal))]),
    );
    let partitions = std::collections::BTreeMap::from([(partition_id, partition)]);
    let root = crate::branch::root::RelationalBranchRoot::prepare_capture(
        execution,
        &crate::branch::root::RelationalBranchRootIdentityIssuer::default(),
        &partitions,
        &delta,
        None,
        Arc::clone(&envelope),
        &Default::default(),
        &registry,
        &Default::default(),
        None,
    )
    .unwrap()
    .into_root();
    let preencoded = RelationalCommitArtifact::from_preencoded(
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::clone(&envelope),
        )
        .unwrap(),
    )
    .unwrap();
    let legacy =
        RelationalCommitArtifact::from_envelope_with_root(Arc::clone(&envelope), Arc::clone(&root))
            .unwrap();
    let relinked = preencoded
        .relink_preencoded_recovery(&envelope, Some(&root), None)
        .unwrap();
    assert!(Arc::ptr_eq(
        &preencoded.envelope_residency(),
        &relinked.envelope_residency()
    ));
    assert_eq!(relinked.identity(), legacy.identity());
    assert_eq!(relinked.parentage(), legacy.parentage());
    assert_eq!(relinked.roots(), legacy.roots());
    assert_eq!(
        relinked
            .canonical_payload_digest()
            .expect("resident fixture has a canonical digest"),
        legacy
            .canonical_payload_digest()
            .expect("resident fixture has a canonical digest")
    );

    let equal_but_unsealed = Arc::new(envelope.independent_fixture_image());
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&equal_but_unsealed, Some(&root), None),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
    let mismatched_schema = crate::branch::target::RelationalBranchRootDescriptor::new(
        *legacy.roots().truth_root(),
        [0xAB; 32],
    );
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&envelope, None, Some(&mismatched_schema)),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
}

pub(super) fn committed_root_and_envelope(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::symbols::data::StringInterner,
    Arc<RelationalBranchRoot>,
    crate::history::data::CanonicalCommitEnvelope,
) {
    let root = capture_committed_fixture(request, 1, None);
    let envelope = root
        .canonical_envelope()
        .unwrap()
        .independent_fixture_image();
    (Default::default(), root, envelope)
}

pub(super) fn capture_committed_fixture(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    version: u64,
    previous: Option<&Arc<RelationalBranchRoot>>,
) -> Arc<RelationalBranchRoot> {
    use crate::branch::reference::observation::relational_branch_observation;
    use crate::branch::reference_checkpoint_data::{
        RelationalBranchCellCheckpoint, CANONICAL_RUNTIME_INSTANCE_ID,
    };
    use crate::branch::target::RelationalBranchTarget;
    use crate::branch::version::RelationalBranchVersion;
    use crate::history::data::{
        BranchId, CanonicalRecordAllocation, CommitId, RelationalCommitReceipt,
    };
    use crate::identity::data::{KindId, LineageId, VersionId};
    use crate::lineage::data::{LineageEventKind, LineageSubject};
    use crate::schema::data::{
        EntityKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    };
    use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

    let partition_id = PartitionId::main();
    let (partition, entities, _) =
        crate::storage::overlay::small_relation_partition::relation_partition_for_test(
            partition_id,
            1,
            &[],
        );
    let record = RecordRef::Entity(entities[0]);
    let branch_id = BranchId("main".into());
    let receipt = RelationalCommitReceipt {
        commit_id: CommitId(version),
        version_id: VersionId(version),
        branch_id: branch_id.clone(),
        parents: previous
            .iter()
            .filter_map(|root| root.commit_id())
            .collect(),
    };
    let mut envelope =
        crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_at(
            record.clone(),
            entities[0],
            receipt.clone(),
        );
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "fixture-entity".into(),
            schema_id: SchemaId("fixture".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let target = RelationalBranchTarget::new(
        CANONICAL_RUNTIME_INSTANCE_ID,
        receipt.commit_id.0,
        receipt.version_id.0,
        Vec::new(),
        RelationalBranchTarget::roots_for_commit(&receipt),
    );
    let observation = relational_branch_observation(
        CANONICAL_RUNTIME_INSTANCE_ID,
        &branch_id.0,
        FoundationalBranchTarget::basis(target),
        FoundationalBranchReferenceGeneration::new(version),
    )
    .unwrap();
    let body = envelope.body_mut_for_test();
    body.schema_authority = registry.authority_snapshot();
    body.branch_cell_checkpoint = Some(RelationalBranchCellCheckpoint {
        runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
        branch_id: branch_id.clone(),
        observation,
        truth_version: RelationalBranchVersion::new(version),
        fork_provenance: None,
        fork_source_branch_id: None,
    });
    body.install_record_allocations(vec![
        CanonicalRecordAllocation::canonical_allocation_for_test(0, record),
    ]);
    body.published_lineage_mut_for_test()
        .lineage_events_mut_for_test()
        .push(assemble_lineage_event_record(LineageEventRecordParts {
            event_id: version,
            commit: receipt,
            branch_id,
            kind: LineageEventKind::Create,
            sources: Vec::new(),
            targets: vec![LineageId(version)],
            subject: LineageSubject::Entity,
            name_birth: None,
            kept_name: None,
            relation_id: None,
            entity_id: None,
        }));
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(entities[0].slot_index());
    let delta = crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
        &std::collections::BTreeMap::from([(partition_id, (partition.clone(), journal))]),
    );
    RelationalBranchRoot::prepare_capture(
        request,
        &RelationalBranchRootIdentityIssuer::default(),
        &std::collections::BTreeMap::from([(partition_id, partition)]),
        &delta,
        previous,
        Arc::new(envelope),
        &Default::default(),
        &registry,
        &Default::default(),
        None,
    )
    .unwrap()
    .into_root()
}
