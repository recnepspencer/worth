use std::sync::Arc;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

use crate::diagnostics::data::{
    DiagnosticCode, RelationalDiagnosticFields, RelationalDiagnosticValue,
    RelationalDiagnosticsEntry,
};
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexArtifacts, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexGenerationId, DerivedIndexId, DerivedIndexPublicationStatus,
};

use super::{RelationalCommitArtifact, RelationalCommitAuthoritativeAllocationKind};

fn canonical_accounting_fixture() -> crate::history::data::CanonicalCommitEnvelope {
    use crate::branch::reference::observation::relational_branch_observation;
    use crate::branch::reference_checkpoint_data::{
        RelationalBranchCellCheckpoint, CANONICAL_RUNTIME_INSTANCE_ID,
    };
    use crate::branch::target::RelationalBranchTarget;
    use crate::branch::version::RelationalBranchVersion;
    use crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update;
    use crate::identity::data::{EntityId, PartitionId};
    use crate::transactions::data::primitives::RecordRef;
    use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let mut envelope = envelope_with_patch_and_update(RecordRef::Entity(entity), entity);
    let receipt = &envelope.commit;
    let branch_id = receipt.branch_id.clone();
    let target = RelationalBranchTarget::new(
        CANONICAL_RUNTIME_INSTANCE_ID,
        receipt.commit_id.0,
        receipt.version_id.as_u64(),
        Vec::new(),
        RelationalBranchTarget::roots_for_commit(receipt),
    );
    let observation = relational_branch_observation(
        CANONICAL_RUNTIME_INSTANCE_ID,
        &branch_id.0,
        FoundationalBranchTarget::basis(target),
        FoundationalBranchReferenceGeneration::new(1),
    )
    .unwrap();
    envelope.body_mut_for_test().branch_cell_checkpoint = Some(RelationalBranchCellCheckpoint {
        runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
        branch_id,
        observation,
        truth_version: RelationalBranchVersion::new(1),
        fork_provenance: None,
        fork_source_branch_id: None,
    });
    envelope
}

#[test]
fn canonical_artifact_accounting_excludes_nested_diagnostic_perturbation() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut envelope = canonical_accounting_fixture();
    let baseline_artifact = RelationalCommitArtifact::from_preencoded(
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::new(envelope.independent_fixture_image()),
        )
        .unwrap(),
    )
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
    let mut omitted_checkpoint = envelope.independent_fixture_image();
    omitted_checkpoint
        .body_mut_for_test()
        .branch_cell_checkpoint = None;
    assert!(
        envelope.allocation_inventory().authoritative_nested_bytes
            > omitted_checkpoint
                .allocation_inventory()
                .authoritative_nested_bytes,
        "the populated branch-cell checkpoint owns canonical nested bytes",
    );
    let nested_text = "nested-diagnostic-payload".repeat(256);
    envelope
        .body_mut_for_test()
        .diagnostics_summary
        .entries
        .push(
            RelationalDiagnosticsEntry::new(
                DiagnosticCode::CommitPublished,
                nested_text.clone(),
                RelationalDiagnosticFields::from_diagnostic_value(
                    RelationalDiagnosticValue::object([(
                        "nested",
                        RelationalDiagnosticValue::array([
                            RelationalDiagnosticValue::string(nested_text),
                            RelationalDiagnosticValue::object([(
                                "leaf",
                                RelationalDiagnosticValue::CanonicalBytes(vec![7; 2_048]),
                            )]),
                        ]),
                    )]),
                ),
            )
            .canonicalized(),
        );
    let artifact = RelationalCommitArtifact::from_preencoded(
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::new(envelope),
        )
        .unwrap(),
    )
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
        "the root owns exactly the encoded canonical payload; diagnostics add nothing",
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
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut envelope = canonical_accounting_fixture();
    let receipt = envelope.commit.clone();
    envelope.body_mut_for_test().derived_index_artifacts = DerivedIndexArtifacts::default();
    let baseline = RelationalCommitArtifact::from_preencoded(
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::new(envelope.independent_fixture_image()),
        )
        .unwrap(),
    )
    .expect("baseline canonical authority payload encodes");
    let baseline_allocations = baseline.authoritative_allocation_observations();
    let baseline_payload_bytes = baseline.canonical_payload_bytes();
    let baseline_optional_cache_bytes = baseline
        .excluded_allocation_inventory()
        .optional_cache_bytes;

    envelope.body_mut_for_test().derived_index_artifacts =
        DerivedIndexArtifacts::new(vec![DerivedIndexGeneration {
            generation_id: DerivedIndexGenerationId(9_001),
            index_id: DerivedIndexId(7_001),
            source_commit_id: receipt.commit_id,
            source_branch_id: receipt.branch_id.clone(),
            applicability: DerivedIndexApplicability {
                branch_id: receipt.branch_id.clone(),
                version_id: receipt.version_id,
                schema_version: envelope.schema_version,
            },
            status: DerivedIndexPublicationStatus::Published,
            entries: DerivedIndexEntries::EntityField(Default::default()),
        }]);
    let perturbed = RelationalCommitArtifact::from_preencoded(
        crate::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::new(envelope),
        )
        .unwrap(),
    )
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
