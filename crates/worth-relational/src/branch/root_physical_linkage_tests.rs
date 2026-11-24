//! Immutable physical-linkage predicate inputs, never an admitted runtime world.
use super::*;
use crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update;
use crate::identity::data::{EntityId, PartitionId};
use crate::transactions::data::primitives::RecordRef;

#[test]
fn equal_envelope_bytes_cannot_replace_exact_physical_catalog_custody() {
    let host = crate::execution::test_policy::serial_request();
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
            Arc::new(envelope.copy_original_for_test(execution)),
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let partition_id = crate::identity::data::PartitionId::main();
    let (partition, entities, _) =
        crate::storage::overlay::small_relation_partition::relation_partition(partition_id, 1, &[]);
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
        crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update(
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

    let equal_but_unsealed = Arc::new(envelope.copy_original_for_test(execution));
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
