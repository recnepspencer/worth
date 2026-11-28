use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::contracts::BridgePreviewReuseEquivalence;
use super::counters::BridgeSpeculationCounters;
use super::session::{BridgePreviewSession, PreviewExecutionRecordIdentity};
use super::taxonomy::{PreviewActive, PreviewAdmitted};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewExecutionRecord {
    record_identity: PreviewExecutionRecordIdentity,
    preview_session_identity: Arc<str>,
    preview_declaration_digest: Arc<str>,
    branch_binding_digest: Arc<str>,
    counters: BridgeSpeculationCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewExecutionRecord {
    pub(crate) fn from_admitted_session(
        session: &BridgePreviewSession<PreviewAdmitted>,
        counters: BridgeSpeculationCounters,
    ) -> Self {
        let declaration = session.declaration().declaration();
        let canonical_basis = Arc::<str>::from(format!(
            "preview-execution-record|session={}|declaration={}|binding={}|truth-view={}|structural-basis={}|source-capability={}|request-shape={}|artifact-schema={}|preview-artifacts={}|discard-artifacts={}|retained-non-authoritative={}",
            session.session_identity().as_str(),
            session.declaration().digest(),
            declaration.branch_binding().digest(),
            declaration.truth_view_basis_digest(),
            declaration.structural_basis_digest().unwrap_or("none"),
            declaration.source_capability_digest(),
            declaration.request_shape_digest(),
            declaration.retained_artifact_schema_digest(),
            counters.preview_artifact_count(),
            counters.discard_artifact_count(),
            counters.retained_non_authoritative_artifact_count(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            record_identity: PreviewExecutionRecordIdentity::admit_bridge_owned(format!(
                "preview-execution-record:sha256:{digest:x}"
            )),
            preview_session_identity: Arc::from(session.session_identity().as_str()),
            preview_declaration_digest: Arc::from(session.declaration().digest()),
            branch_binding_digest: Arc::from(declaration.branch_binding().digest()),
            counters,
            canonical_basis,
            digest: Arc::from(format!("preview-execution-record:sha256:{digest:x}")),
        }
    }

    pub(crate) fn from_reused_active_session(
        source_session: &BridgePreviewSession<PreviewActive>,
        target_session: &BridgePreviewSession<PreviewAdmitted>,
        source_execution_record: &BridgePreviewExecutionRecord,
        reuse_equivalence: &BridgePreviewReuseEquivalence,
        counters: BridgeSpeculationCounters,
    ) -> Self {
        let declaration = target_session.declaration().declaration();
        let canonical_basis = Arc::<str>::from(format!(
            "preview-execution-record|session={}|declaration={}|binding={}|truth-view={}|structural-basis={}|source-capability={}|request-shape={}|artifact-schema={}|reused-from-session={}|reused-from-execution-record={}|reuse-equivalence={}|preview-artifacts={}|discard-artifacts={}|retained-non-authoritative={}",
            target_session.session_identity().as_str(),
            target_session.declaration().digest(),
            declaration.branch_binding().digest(),
            declaration.truth_view_basis_digest(),
            declaration.structural_basis_digest().unwrap_or("none"),
            declaration.source_capability_digest(),
            declaration.request_shape_digest(),
            declaration.retained_artifact_schema_digest(),
            source_session.session_identity().as_str(),
            source_execution_record.record_identity().as_str(),
            reuse_equivalence.digest(),
            counters.preview_artifact_count(),
            counters.discard_artifact_count(),
            counters.retained_non_authoritative_artifact_count(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            record_identity: PreviewExecutionRecordIdentity::admit_bridge_owned(format!(
                "preview-execution-record:sha256:{digest:x}"
            )),
            preview_session_identity: Arc::from(target_session.session_identity().as_str()),
            preview_declaration_digest: Arc::from(target_session.declaration().digest()),
            branch_binding_digest: Arc::from(declaration.branch_binding().digest()),
            counters,
            canonical_basis,
            digest: Arc::from(format!("preview-execution-record:sha256:{digest:x}")),
        }
    }

    pub fn record_identity(&self) -> &PreviewExecutionRecordIdentity {
        &self.record_identity
    }

    pub fn preview_session_identity(&self) -> &str {
        self.preview_session_identity.as_ref()
    }

    pub fn preview_declaration_digest(&self) -> &str {
        self.preview_declaration_digest.as_ref()
    }

    pub fn branch_binding_digest(&self) -> &str {
        self.branch_binding_digest.as_ref()
    }

    pub fn counters(&self) -> &BridgeSpeculationCounters {
        &self.counters
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
