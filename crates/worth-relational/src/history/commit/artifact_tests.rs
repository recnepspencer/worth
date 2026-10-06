use std::sync::Arc;

use crate::diagnostics::data::{
    DiagnosticCode, RelationalDiagnosticFields, RelationalDiagnosticValue,
    RelationalDiagnosticsEntry,
};
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexArtifacts, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexGenerationId, DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::tests::support::{create_entity_outcome, persisted_runtime_with_test_schema};

#[test]
fn fresh_scalar_commit_seals_exact_native_lifecycle_and_field_touches() {
    use crate::history::data::RelationalDescriptiveTouch;
    use crate::identity::data::KindId;
    use crate::tests::support::{create_entity_outcome_with_lease, test_execution_lease};

    for checked in [false, true] {
        let runtime = persisted_runtime_with_test_schema();
        let committed = if checked {
            create_entity_outcome_with_lease(&runtime, "touch-value", &test_execution_lease())
        } else {
            create_entity_outcome(&runtime, "touch-value")
        };
        let envelope = runtime
            .replay()
            .canonical_commit_envelope(committed.commit.commit_id)
            .expect("performed commit has sealed envelope");
        let touches = envelope
            .descriptive_touches()
            .exact_touches()
            .expect("fresh scalar commit has exact native delta");
        assert!(touches.iter().any(|touch| matches!(
            touch,
            RelationalDescriptiveTouch::EntityLifecycle {
                kind: KindId(1),
                ..
            }
        )));
        assert!(touches.iter().any(|touch| matches!(touch,
            RelationalDescriptiveTouch::AspectRevision { aspect, .. } if aspect.as_str() == "name"
        )));
        assert!(touches.iter().any(|touch| matches!(touch,
            RelationalDescriptiveTouch::FieldRevision { kind: KindId(1), aspect, path, presence, .. }
                if aspect.as_str() == "name"
                    && path.fields()[0].as_str() == "name"
                    && *presence == crate::storage::data::RelationalFieldPresence::Present
        )));
    }
}

#[test]
fn installed_field_index_commit_touch_carries_native_old_and_new_keys() {
    use crate::history::data::RelationalDescriptiveTouch;
    use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
    use crate::tests::support::{aspect_field_locator, aspect_key, field_key};
    use worth_foundational::facade::AspectValue;

    let runtime = persisted_runtime_with_test_schema();
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "touch-index".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let committed = create_entity_outcome(&runtime, "indexed-value");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(committed.commit.commit_id)
        .expect("performed indexed commit has envelope");
    let touches = envelope
        .descriptive_touches()
        .exact_touches()
        .expect("exact native graph");
    let expected = crate::storage::data::AuthoritativeFieldComparisonKey::from_aspect_value(
        &AspectValue::String("indexed-value".into()),
    );
    assert!(touches.iter().any(|touch| matches!(touch,
        RelationalDescriptiveTouch::IndexMembership { index: id, old_key: None, new_key: Some(key), .. }
            if *id == index.index_id && key == &expected
    )));
}

use super::{
    RelationalCommitArtifact, RelationalCommitArtifactDenial,
    RelationalCommitAuthoritativeAllocationKind,
};

#[test]
fn recovery_relinks_an_already_sealed_payload_without_reencoding_or_weakening_root_checks() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(&runtime, "preencoded-recovery-root");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("committed main has one root");
    let envelope = root
        .canonical_envelope()
        .cloned()
        .expect("committed root carries its canonical envelope");
    let preencoded = RelationalCommitArtifact::from_envelope(Arc::clone(&envelope)).unwrap();
    let legacy =
        RelationalCommitArtifact::from_envelope_with_root(Arc::clone(&envelope), Arc::clone(&root))
            .unwrap();
    let relinked = preencoded
        .relink_preencoded_recovery(&envelope, Some(&root), None)
        .unwrap();
    assert!(Arc::ptr_eq(
        &preencoded.canonical_payload,
        &relinked.canonical_payload
    ));
    assert_eq!(relinked.identity(), legacy.identity());
    assert_eq!(relinked.parentage(), legacy.parentage());
    assert_eq!(relinked.roots(), legacy.roots());
    assert_eq!(
        relinked.canonical_payload_digest(),
        legacy.canonical_payload_digest()
    );

    let equal_but_unsealed = Arc::new((*envelope).clone());
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&equal_but_unsealed, Some(&root), None),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
    let mismatched_schema = crate::branch::RelationalBranchRootDescriptor::new(
        *legacy.roots().truth_root(),
        [0xAB; 32],
    );
    assert!(matches!(
        preencoded.relink_preencoded_recovery(&envelope, None, Some(&mismatched_schema)),
        Err(RelationalCommitArtifactDenial::RootLinkage)
    ));
}

#[test]
fn canonical_artifact_accounting_excludes_nested_diagnostic_perturbation() {
    let runtime = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "artifact-allocation-basis");
    let mut envelope = runtime
        .replay()
        .canonical_commit_envelope(committed.commit.commit_id)
        .expect("the performed commit owns one canonical envelope")
        .clone();
    let baseline_artifact = RelationalCommitArtifact::from_envelope(Arc::new(envelope.clone()))
        .expect("baseline canonical authority payload encodes");
    let baseline_payload_bytes = baseline_artifact.canonical_payload_bytes();
    let baseline_authoritative_nested = baseline_artifact
        .authoritative_allocation_observations()
        .into_iter()
        .find(|allocation| {
            allocation.kind
                == RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage
        })
        .expect("authoritative nested lane exists")
        .authoritative_bytes;
    let baseline_diagnostic_bytes = baseline_artifact
        .excluded_allocation_inventory()
        .diagnostic_bytes;
    let mut omitted_checkpoint = envelope.clone();
    omitted_checkpoint.branch_cell_checkpoint = None;
    assert!(
        envelope.allocation_inventory().authoritative_nested_bytes
            > omitted_checkpoint
                .allocation_inventory()
                .authoritative_nested_bytes,
        "the populated branch-cell checkpoint owns canonical nested bytes",
    );
    let nested_text = "nested-diagnostic-payload".repeat(256);
    envelope.diagnostics_summary.entries.push(
        RelationalDiagnosticsEntry::new(
            DiagnosticCode::CommitPublished,
            nested_text.clone(),
            RelationalDiagnosticFields::from_diagnostic_value(RelationalDiagnosticValue::object([
                (
                    "nested",
                    RelationalDiagnosticValue::array([
                        RelationalDiagnosticValue::string(nested_text),
                        RelationalDiagnosticValue::object([(
                            "leaf",
                            RelationalDiagnosticValue::CanonicalBytes(vec![7; 2_048]),
                        )]),
                    ]),
                ),
            ])),
        )
        .canonicalized(),
    );
    let artifact = RelationalCommitArtifact::from_envelope(Arc::new(envelope))
        .expect("the owner seals the canonical payload allocation");

    assert_eq!(
        artifact.canonical_payload_bytes(),
        baseline_payload_bytes,
        "diagnostic richness cannot perturb canonical authority payload bytes"
    );
    let allocations = artifact.authoritative_allocation_observations();
    let allocation = |kind| {
        allocations
            .iter()
            .find(|allocation| allocation.kind == kind)
            .expect("every canonical owner allocation is inventoried")
            .authoritative_bytes
    };
    assert_eq!(
        allocation(RelationalCommitAuthoritativeAllocationKind::ArtifactObject),
        std::mem::size_of::<RelationalCommitArtifact>() as u64,
    );
    assert_eq!(
        allocation(RelationalCommitAuthoritativeAllocationKind::CanonicalPayload),
        baseline_payload_bytes,
    );
    assert_eq!(
        allocation(RelationalCommitAuthoritativeAllocationKind::EnvelopeObject),
        std::mem::size_of::<crate::history::data::CanonicalCommitEnvelope>() as u64,
    );
    assert_eq!(
        allocation(RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage),
        baseline_authoritative_nested,
        "diagnostic allocation cannot enter authoritative nested-envelope accounting"
    );
    assert!(
        artifact.excluded_allocation_inventory().diagnostic_bytes
            > baseline_diagnostic_bytes.saturating_add(2_048),
        "nested diagnostic strings and canonical bytes remain visible in their own lane"
    );
}

#[test]
fn populated_derived_index_artifacts_only_change_the_optional_cache_lane() {
    let runtime = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "derived-index-allocation-basis");
    let mut envelope = runtime
        .replay()
        .canonical_commit_envelope(committed.commit.commit_id)
        .expect("the performed commit owns one canonical envelope")
        .clone();
    envelope.derived_index_artifacts = DerivedIndexArtifacts::default();
    let baseline = RelationalCommitArtifact::from_envelope(Arc::new(envelope.clone()))
        .expect("baseline canonical authority payload encodes");
    let baseline_allocations = baseline.authoritative_allocation_observations();
    let baseline_payload_bytes = baseline.canonical_payload_bytes();
    let baseline_optional_cache_bytes = baseline
        .excluded_allocation_inventory()
        .optional_cache_bytes;

    envelope.derived_index_artifacts = DerivedIndexArtifacts::new(vec![DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(9_001),
        index_id: DerivedIndexId(7_001),
        source_commit_id: committed.commit.commit_id,
        source_branch_id: committed.commit.branch_id.clone(),
        applicability: DerivedIndexApplicability {
            branch_id: committed.commit.branch_id.clone(),
            version_id: committed.commit.version_id,
            schema_version: envelope.schema_version,
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(Default::default()),
    }]);
    let perturbed = RelationalCommitArtifact::from_envelope(Arc::new(envelope))
        .expect("derived artifacts remain outside canonical authority encoding");

    assert_eq!(perturbed.canonical_payload_bytes(), baseline_payload_bytes);
    assert_eq!(
        perturbed.authoritative_allocation_observations(),
        baseline_allocations,
        "optional derived indexes cannot perturb any authoritative allocation lane",
    );
    assert!(
        perturbed
            .excluded_allocation_inventory()
            .optional_cache_bytes
            > baseline_optional_cache_bytes,
        "the populated artifact remains observable in the optional-cache lane",
    );
}
