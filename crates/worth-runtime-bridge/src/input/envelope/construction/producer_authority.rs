use sha2::{Digest, Sha256};

use crate::error::{BridgeRouteError, BridgeRouteErrorKind};
use crate::snapshot::TruthSnapshotIdentity;

use super::super::{
    BridgeAuthoritativeSourceProvenance, BridgeCommittedPatchItem, BridgeCommittedRecordChange,
    BridgeProducerAuthorityKind, BridgeProducerMetadata, TruthBranchIdentity, TruthCommitIdentity,
    TruthPatchIdentity,
};

pub(super) fn validate_producer_metadata(
    metadata: &BridgeProducerMetadata,
) -> Result<(), BridgeRouteError> {
    super::validate_identity(
        "producer export schema version",
        metadata.export_schema_version(),
    )?;
    if metadata.export_schema_version() != super::super::BRIDGE_PRODUCER_EXPORT_SCHEMA_V1 {
        return Err(BridgeRouteError::new(
            BridgeRouteErrorKind::UnsupportedProducerEnvelope,
            format!(
                "Committed patch producer schema `{}` is not supported; expected `{}`.",
                metadata.export_schema_version(),
                super::super::BRIDGE_PRODUCER_EXPORT_SCHEMA_V1
            ),
        ));
    }

    match metadata.authority_kind() {
        BridgeProducerAuthorityKind::RegisteredAuthoritativeSource => {
            if metadata.authoritative_source().is_none() {
                return Err(BridgeRouteError::new(
                    BridgeRouteErrorKind::UnsupportedProducerEnvelope,
                    "registered authoritative source envelope omitted its source provenance",
                ));
            }
        }
        BridgeProducerAuthorityKind::BridgeHarnessFixture => {}
        BridgeProducerAuthorityKind::Unknown => {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::UnsupportedProducerEnvelope,
                "Committed patch producer authority `unknown` is not supported.",
            ));
        }
    }

    if let Some(semantics_version) = metadata.producer_semantics_version() {
        super::validate_identity("producer semantics version", semantics_version)?;
    }
    if let Some(source) = metadata.authoritative_source() {
        if source.runtime_instance_id() == 0 {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::UnsupportedProducerEnvelope,
                "authoritative source provenance requires a runtime instance",
            ));
        }
        super::validate_identity("authoritative graph role", source.graph_role())?;
        super::validate_identity(
            "authoritative adapter semantic identity",
            source.adapter_semantic_identity(),
        )?;
        super::validate_identity("authoritative source basis", source.source_basis())?;
    }
    if let Some(feedback_context) = metadata.writeback_feedback_context() {
        super::validate_identity(
            "writeback feedback context digest",
            feedback_context.digest(),
        )?;
        super::validate_identity(
            "writeback feedback provenance digest",
            feedback_context.provenance_digest(),
        )?;
        super::validate_identity(
            "writeback feedback causality digest",
            feedback_context.causality_digest(),
        )?;
        super::validate_identity(
            "writeback feedback effect intent digest",
            feedback_context.effect_intent_digest(),
        )?;
    }
    Ok(())
}

pub(super) fn digest_basis(
    producer_metadata: &BridgeProducerMetadata,
    commit_identity: &TruthCommitIdentity,
    patch_identity: &TruthPatchIdentity,
    snapshot_identity: &TruthSnapshotIdentity,
    branch_identity: &TruthBranchIdentity,
    canonical_items: &[BridgeCommittedPatchItem],
    canonical_record_changes: &[BridgeCommittedRecordChange],
    normalized_patch_item_count: usize,
) -> String {
    let mut basis = format!(
        "patch|producer={}|schema={}|source={}|commit={}|patch={}|snapshot={}|branch={}|normalized-item-count={}",
        producer_metadata.authority_kind().canonical_label(),
        producer_metadata.export_schema_version(),
        producer_metadata
            .authoritative_source()
            .map(BridgeAuthoritativeSourceProvenance::canonical_basis)
            .unwrap_or_else(|| "none".to_string()),
        commit_identity.as_str(),
        patch_identity.as_str(),
        snapshot_identity.as_str(),
        branch_identity.as_str(),
        normalized_patch_item_count,
    );
    for item in canonical_items {
        basis.push_str("|item=");
        basis.push_str(item.entity_identity());
        basis.push(':');
        basis.push_str(&item.canonical_basis());
    }
    for record_change in canonical_record_changes {
        basis.push_str("|record-change=");
        basis.push_str(&record_change.canonical_basis());
    }
    basis
}

pub(super) fn committed_patch_digest_from_basis(canonical_basis: &str) -> String {
    let digest = Sha256::digest(canonical_basis.as_bytes());
    format!("patch:sha256:{digest:x}")
}
