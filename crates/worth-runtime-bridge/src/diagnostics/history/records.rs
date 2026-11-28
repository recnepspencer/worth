use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::diagnostics::history::BridgeHistoricalEvaluationCounters;
use crate::error::{BridgeErrorContext, BridgeReplayError, BridgeReplayErrorKind};
use crate::identity::{
    BridgeIdentity, HistoricalEvaluationDecisionLogIdentityTag,
    HistoricalEvaluationRecordIdentityTag,
};
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity};
use crate::snapshot::{
    BridgeTruthViewSelectorIdentity, HistoricalEvaluationDeclaration,
    HistoricalEvaluationDeclarationIdentity, SnapshotReadPacket, TruthSnapshotIdentity,
};

pub const BRIDGE_CANONICAL_HISTORICAL_EVALUATION_RECORD_SCHEMA_V1: &str =
    "worth-runtime-bridge.historical-evaluation-record.v1";

pub type BridgeHistoricalEvaluationRecordIdentity =
    BridgeIdentity<HistoricalEvaluationRecordIdentityTag>;
pub type BridgeHistoricalEvaluationDecisionLogIdentity =
    BridgeIdentity<HistoricalEvaluationDecisionLogIdentityTag>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeHistoricalMaterializationPath {
    DirectSnapshotRead,
    CommitEnvelopeSnapshot,
    PinnedCommitObservation,
    BranchHeadEnvelopeSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalEvaluationDecisionLog {
    decision_log_identity: BridgeHistoricalEvaluationDecisionLogIdentity,
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    selector_identity: BridgeTruthViewSelectorIdentity,
    resolved_policy_digest: Arc<str>,
    planned_packet_digest: Arc<str>,
    authority_digest: Arc<str>,
    materialization_path: BridgeHistoricalMaterializationPath,
    branch_identity: TruthBranchIdentity,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: TruthSnapshotIdentity,
}

impl BridgeHistoricalEvaluationDecisionLog {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        declaration_identity: HistoricalEvaluationDeclarationIdentity,
        selector_identity: BridgeTruthViewSelectorIdentity,
        resolved_policy_digest: Arc<str>,
        planned_packet_digest: Arc<str>,
        authority_digest: Arc<str>,
        materialization_path: BridgeHistoricalMaterializationPath,
        branch_identity: TruthBranchIdentity,
        commit_identity: Option<TruthCommitIdentity>,
        snapshot_identity: TruthSnapshotIdentity,
    ) -> Self {
        let commit_identity_basis = commit_identity
            .as_ref()
            .map(TruthCommitIdentity::as_str)
            .unwrap_or("none");
        let canonical_basis = format!(
            "historical-evaluation-decision-log|declaration={}|selector={}|policy={}|planned={}|authority={}|path={materialization_path:?}|branch={}|commit={}|snapshot={}",
            declaration_identity.as_str(),
            selector_identity.as_str(),
            resolved_policy_digest.as_ref(),
            planned_packet_digest.as_ref(),
            authority_digest.as_ref(),
            branch_identity.as_str(),
            commit_identity_basis,
            snapshot_identity.as_str(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            decision_log_identity:
                BridgeHistoricalEvaluationDecisionLogIdentity::admit_bridge_owned(format!(
                    "historical-evaluation-decision-log:sha256:{digest:x}"
                )),
            declaration_identity,
            selector_identity,
            resolved_policy_digest,
            planned_packet_digest,
            authority_digest,
            materialization_path,
            branch_identity,
            commit_identity,
            snapshot_identity,
        }
    }

    pub fn decision_log_identity(&self) -> &BridgeHistoricalEvaluationDecisionLogIdentity {
        &self.decision_log_identity
    }

    pub fn declaration_identity(&self) -> &HistoricalEvaluationDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn selector_identity(&self) -> &BridgeTruthViewSelectorIdentity {
        &self.selector_identity
    }

    pub fn resolved_policy_digest(&self) -> &str {
        self.resolved_policy_digest.as_ref()
    }

    pub fn planned_packet_digest(&self) -> &str {
        self.planned_packet_digest.as_ref()
    }

    pub fn authority_digest(&self) -> &str {
        self.authority_digest.as_ref()
    }

    pub fn materialization_path(&self) -> BridgeHistoricalMaterializationPath {
        self.materialization_path
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn commit_identity(&self) -> Option<&TruthCommitIdentity> {
        self.commit_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalEvaluationRecord {
    record_identity: BridgeHistoricalEvaluationRecordIdentity,
    declaration: HistoricalEvaluationDeclaration,
    read_packet: SnapshotReadPacket,
    decision_log: BridgeHistoricalEvaluationDecisionLog,
    counters: BridgeHistoricalEvaluationCounters,
}

impl BridgeHistoricalEvaluationRecord {
    pub(crate) fn new(
        declaration: HistoricalEvaluationDeclaration,
        read_packet: SnapshotReadPacket,
        decision_log: BridgeHistoricalEvaluationDecisionLog,
        counters: BridgeHistoricalEvaluationCounters,
    ) -> Self {
        let canonical_basis = format!(
            "historical-evaluation-record|declaration={}|packet={}|decision-log={}",
            declaration.declaration_identity().as_str(),
            decision_log.planned_packet_digest(),
            decision_log.decision_log_identity().as_str(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            record_identity: BridgeHistoricalEvaluationRecordIdentity::admit_bridge_owned(format!(
                "historical-evaluation-record:sha256:{digest:x}"
            )),
            declaration,
            read_packet,
            decision_log,
            counters,
        }
    }

    pub fn record_identity(&self) -> &BridgeHistoricalEvaluationRecordIdentity {
        &self.record_identity
    }

    pub fn declaration(&self) -> &HistoricalEvaluationDeclaration {
        &self.declaration
    }

    pub fn read_packet(&self) -> &SnapshotReadPacket {
        &self.read_packet
    }

    pub fn decision_log(&self) -> &BridgeHistoricalEvaluationDecisionLog {
        &self.decision_log
    }

    pub fn counters(&self) -> &BridgeHistoricalEvaluationCounters {
        &self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCanonicalHistoricalEvaluationRecord {
    schema_version: Arc<str>,
    record: BridgeHistoricalEvaluationRecord,
}

impl BridgeCanonicalHistoricalEvaluationRecord {
    pub(crate) fn new(record: BridgeHistoricalEvaluationRecord) -> Self {
        Self {
            schema_version: Arc::from(BRIDGE_CANONICAL_HISTORICAL_EVALUATION_RECORD_SCHEMA_V1),
            record,
        }
    }

    pub fn schema_version(&self) -> &str {
        self.schema_version.as_ref()
    }

    pub fn record_identity(&self) -> &BridgeHistoricalEvaluationRecordIdentity {
        self.record.record_identity()
    }

    pub fn declaration(&self) -> &HistoricalEvaluationDeclaration {
        self.record.declaration()
    }

    pub fn decision_log(&self) -> &BridgeHistoricalEvaluationDecisionLog {
        self.record.decision_log()
    }

    pub fn read_packet(&self) -> &SnapshotReadPacket {
        self.record.read_packet()
    }

    pub fn counters(&self) -> &BridgeHistoricalEvaluationCounters {
        self.record.counters()
    }

    pub(crate) fn decode(&self) -> Result<BridgeHistoricalEvaluationRecord, BridgeReplayError> {
        if self.schema_version() != BRIDGE_CANONICAL_HISTORICAL_EVALUATION_RECORD_SCHEMA_V1 {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::CanonicalArtifactCoherenceFailure,
                format!(
                    "Bridge canonical historical evaluation record schema `{}` is not supported; expected `{}`.",
                    self.schema_version(),
                    BRIDGE_CANONICAL_HISTORICAL_EVALUATION_RECORD_SCHEMA_V1
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        Ok(self.record.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeHistoricalEvaluationExplanation {
    record_identity: BridgeHistoricalEvaluationRecordIdentity,
    declaration_identity: HistoricalEvaluationDeclarationIdentity,
    branch_identity: TruthBranchIdentity,
    commit_identity: Option<TruthCommitIdentity>,
    snapshot_identity: TruthSnapshotIdentity,
    materialization_path: BridgeHistoricalMaterializationPath,
}

impl BridgeHistoricalEvaluationExplanation {
    pub fn from_canonical_record(record: &BridgeCanonicalHistoricalEvaluationRecord) -> Self {
        let record = &record.record;
        Self {
            record_identity: record.record_identity().clone(),
            declaration_identity: record.declaration().declaration_identity().clone(),
            branch_identity: record.decision_log().branch_identity().clone(),
            commit_identity: record.decision_log().commit_identity().cloned(),
            snapshot_identity: record.decision_log().snapshot_identity().clone(),
            materialization_path: record.decision_log().materialization_path(),
        }
    }

    pub fn record_identity(&self) -> &BridgeHistoricalEvaluationRecordIdentity {
        &self.record_identity
    }

    pub fn declaration_identity(&self) -> &HistoricalEvaluationDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn commit_identity(&self) -> Option<&TruthCommitIdentity> {
        self.commit_identity.as_ref()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn materialization_path(&self) -> BridgeHistoricalMaterializationPath {
        self.materialization_path
    }
}
