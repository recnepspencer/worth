use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, PreviewDiscardRecordIdentityTag};

use super::counters::BridgeSpeculationCounters;
use super::execution::BridgePreviewExecutionRecord;
use super::session::{BridgePreviewSession, PreviewExecutionRecordIdentity};
use super::taxonomy::{BridgePreviewResidueClass, PreviewActive};

pub type BridgePreviewDiscardRecordIdentity = BridgeIdentity<PreviewDiscardRecordIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgePreviewDiscardCleanupOutcome {
    ZeroAuthoritativeResidue,
}

/// ```compile_fail,E0624
/// use worth_runtime_bridge::facade::BridgePreviewResidueReport;
/// let _ = BridgePreviewResidueReport::new(Vec::new());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewResidueReport {
    residue_classes: Arc<[BridgePreviewResidueClass]>,
    authoritative_residue_count: usize,
    destroyable_residue_count: usize,
    retained_non_authoritative_count: usize,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewResidueReport {
    pub(crate) fn new(residue_classes: Vec<BridgePreviewResidueClass>) -> Self {
        let authoritative_residue_count = residue_classes
            .iter()
            .filter(|class| {
                matches!(
                    class,
                    BridgePreviewResidueClass::AuthoritativeRoutingResidue
                        | BridgePreviewResidueClass::AuthoritativeCheckpointResidue
                        | BridgePreviewResidueClass::AuthoritativeReplayResidue
                        | BridgePreviewResidueClass::AuthoritativeDiagnosticsResidue
                        | BridgePreviewResidueClass::AuthoritativeWritebackResidue
                )
            })
            .count();
        let destroyable_residue_count = residue_classes
            .iter()
            .filter(|class| {
                matches!(
                    class,
                    BridgePreviewResidueClass::TemporaryRoutingResidue
                        | BridgePreviewResidueClass::TemporaryStructuralResidue
                        | BridgePreviewResidueClass::TemporaryDiagnosticsResidue
                )
            })
            .count();
        let retained_non_authoritative_count = residue_classes
            .iter()
            .filter(|class| {
                matches!(
                    class,
                    BridgePreviewResidueClass::PreviewExecutionRetained
                        | BridgePreviewResidueClass::PreviewDiagnosticsRetained
                        | BridgePreviewResidueClass::ReplayRetainedNonAuthoritative
                )
            })
            .count();
        let canonical_basis = Arc::<str>::from(format!(
            "preview-residue-report|authoritative={}|destroyable={}|retained-non-authoritative={}|classes={}",
            authoritative_residue_count,
            destroyable_residue_count,
            retained_non_authoritative_count,
            residue_classes
                .iter()
                .map(|class| format!("{class:?}"))
                .collect::<Vec<_>>()
                .join(","),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            residue_classes: Arc::from(residue_classes),
            authoritative_residue_count,
            destroyable_residue_count,
            retained_non_authoritative_count,
            canonical_basis,
            digest: Arc::from(format!("preview-residue-report:sha256:{digest:x}")),
        }
    }

    pub fn residue_classes(&self) -> &[BridgePreviewResidueClass] {
        &self.residue_classes
    }

    pub fn authoritative_residue_count(&self) -> usize {
        self.authoritative_residue_count
    }

    pub fn destroyable_residue_count(&self) -> usize {
        self.destroyable_residue_count
    }

    pub fn retained_non_authoritative_count(&self) -> usize {
        self.retained_non_authoritative_count
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

/// ```compile_fail,E0624
/// use worth_runtime_bridge::facade::{
///     BridgePreviewDiscardRecord, BridgePreviewExecutionRecord, BridgePreviewResidueReport,
///     BridgePreviewSession, BridgeSpeculationCounters, PreviewActive,
/// };
/// fn forge(session: &BridgePreviewSession<PreviewActive>, execution: &BridgePreviewExecutionRecord,
///          residue: BridgePreviewResidueReport, counters: BridgeSpeculationCounters) {
///     let _ = BridgePreviewDiscardRecord::from_active_session(session, execution, residue, counters);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgePreviewDiscardRecord {
    record_identity: BridgePreviewDiscardRecordIdentity,
    preview_session_identity: Arc<str>,
    preview_execution_record_identity: PreviewExecutionRecordIdentity,
    residue_report: BridgePreviewResidueReport,
    cleanup_outcome: BridgePreviewDiscardCleanupOutcome,
    counters: BridgeSpeculationCounters,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgePreviewDiscardRecord {
    pub(crate) fn from_active_session(
        session: &BridgePreviewSession<PreviewActive>,
        execution_record: &BridgePreviewExecutionRecord,
        residue_report: BridgePreviewResidueReport,
        counters: BridgeSpeculationCounters,
    ) -> Self {
        let execution_record_identity = session
            .execution_record_identity()
            .expect("active preview sessions must carry execution record identity")
            .clone();
        let canonical_basis = Arc::<str>::from(format!(
            "preview-discard-record|session={}|execution-record={}|execution-digest={}|residue-report={}|destroyed={}|retained-non-authoritative={}",
            session.session_identity().as_str(),
            execution_record_identity.as_str(),
            execution_record.digest(),
            residue_report.digest(),
            counters.destroyed_artifact_count(),
            counters.retained_non_authoritative_artifact_count(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            record_identity: BridgePreviewDiscardRecordIdentity::admit_bridge_owned(format!(
                "preview-discard-record:sha256:{digest:x}"
            )),
            preview_session_identity: Arc::from(session.session_identity().as_str()),
            preview_execution_record_identity: execution_record_identity,
            residue_report,
            cleanup_outcome: BridgePreviewDiscardCleanupOutcome::ZeroAuthoritativeResidue,
            counters,
            canonical_basis,
            digest: Arc::from(format!("preview-discard-record:sha256:{digest:x}")),
        }
    }

    pub fn record_identity(&self) -> &BridgePreviewDiscardRecordIdentity {
        &self.record_identity
    }

    pub fn residue_report(&self) -> &BridgePreviewResidueReport {
        &self.residue_report
    }

    pub fn preview_session_identity(&self) -> &str {
        self.preview_session_identity.as_ref()
    }

    pub fn cleanup_outcome(&self) -> BridgePreviewDiscardCleanupOutcome {
        self.cleanup_outcome
    }

    pub fn counters(&self) -> &BridgeSpeculationCounters {
        &self.counters
    }

    pub fn preview_execution_record_identity(&self) -> &PreviewExecutionRecordIdentity {
        &self.preview_execution_record_identity
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
