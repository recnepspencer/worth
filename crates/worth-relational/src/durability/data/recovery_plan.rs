use serde::{Deserialize, Serialize};

use super::{DurableCheckpoint, RecoveryAuthorityContinuityMismatch};
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::patch::data::PatchStreamPosition;
use crate::replay::data::ReplayVerificationLayer;
use crate::schema::data::DescriptorSemanticsVersion;

/// What a recovery rebuilt from: the checkpoint it started from (if any) and
/// the canonical tail it replayed after that checkpoint's coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryCoverage {
    /// The last position the selected checkpoint covers; `None` when recovery
    /// replayed history from the first position.
    pub checkpoint_covered_position: Option<PatchStreamPosition>,
    pub checkpoint_commits: usize,
    pub replayed_tail_commits: usize,
    pub recovered_through_commit: Option<RelationalCommitReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryAuthorityContinuityCheck {
    pub schema_parity: RecoveryAuthorityParity,
    pub profile_parity: RecoveryAuthorityParity,
    pub runtime_name_parity: RecoveryAuthorityParity,
    pub descriptor_version_parity: RecoveryAuthorityParity,
    pub schema_transition_parity: RecoveryAuthorityParity,
    pub continuation_descriptor_parity: RecoveryAuthorityParity,
    pub reconciliation_descriptor_parity: RecoveryAuthorityParity,
    pub schema_lineage_parity: RecoveryAuthorityParity,
    pub verification_outcome: RecoveryVerificationOutcome,
    pub first_mismatch: Option<RecoveryAuthorityContinuityMismatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryVerificationMode {
    NormalRecoveryVerification,
    AuditRecoveryVerification,
    CorruptionDiagnosisReplay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryVerificationPlan {
    Normal(NormalRecoveryVerificationPlan),
    Audit(AuditRecoveryVerificationPlan),
    CorruptionDiagnosis(CorruptionDiagnosisReplayPlan),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalRecoveryVerificationPlan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecoveryVerificationPlan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorruptionDiagnosisReplayPlan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryVerificationOutcome {
    VerifiedAtLayer(ReplayVerificationLayer),
    Rejected {
        layer: ReplayVerificationLayer,
        detail: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAuthorityParity {
    VerifiedAtLayer(ReplayVerificationLayer),
    Drift,
}

#[derive(Debug, Clone)]
pub struct RecoveryPlan {
    pub config: crate::runtime::RelationalRuntimeConfig,
    pub(crate) cold_source:
        Option<std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    pub checkpoint: Option<DurableCheckpoint>,
    pub(crate) tail_log: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
    pub(crate) branch_transitions: super::RecoveryBranchTransitions,
    pub authority_continuity: RecoveryAuthorityContinuityCheck,
    pub verification_plan: RecoveryVerificationPlan,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
    pub restore_authoritative_envelope_commit_ids: Vec<CommitId>,
    checkpoint_covered_position: Option<PatchStreamPosition>,
    pub(crate) commit_strategy_executors:
        crate::commit_strategies::FrozenCommitStrategyExecutorRegistry,
}

impl RecoveryVerificationPlan {
    pub fn from_mode(mode: RecoveryVerificationMode) -> Self {
        match mode {
            RecoveryVerificationMode::NormalRecoveryVerification => {
                Self::Normal(NormalRecoveryVerificationPlan)
            }
            RecoveryVerificationMode::AuditRecoveryVerification => {
                Self::Audit(AuditRecoveryVerificationPlan)
            }
            RecoveryVerificationMode::CorruptionDiagnosisReplay => {
                Self::CorruptionDiagnosis(CorruptionDiagnosisReplayPlan)
            }
        }
    }

    pub fn allows_deep_artifact_parity(&self) -> bool {
        !matches!(self, Self::Normal(_))
    }

    pub fn mode(&self) -> RecoveryVerificationMode {
        match self {
            Self::Normal(_) => RecoveryVerificationMode::NormalRecoveryVerification,
            Self::Audit(_) => RecoveryVerificationMode::AuditRecoveryVerification,
            Self::CorruptionDiagnosis(_) => RecoveryVerificationMode::CorruptionDiagnosisReplay,
        }
    }
}

impl RecoveryAuthorityParity {
    pub fn verified_at(layer: ReplayVerificationLayer) -> Self {
        Self::VerifiedAtLayer(layer)
    }

    pub fn drift() -> Self {
        Self::Drift
    }

    pub fn is_verified(&self) -> bool {
        matches!(self, Self::VerifiedAtLayer(_))
    }
}

impl RecoveryAuthorityContinuityCheck {
    pub fn verified_at(layer: ReplayVerificationLayer) -> Self {
        Self {
            schema_parity: RecoveryAuthorityParity::verified_at(layer),
            profile_parity: RecoveryAuthorityParity::verified_at(layer),
            runtime_name_parity: RecoveryAuthorityParity::verified_at(layer),
            descriptor_version_parity: RecoveryAuthorityParity::verified_at(layer),
            schema_transition_parity: RecoveryAuthorityParity::verified_at(layer),
            continuation_descriptor_parity: RecoveryAuthorityParity::verified_at(layer),
            reconciliation_descriptor_parity: RecoveryAuthorityParity::verified_at(layer),
            schema_lineage_parity: RecoveryAuthorityParity::verified_at(layer),
            verification_outcome: RecoveryVerificationOutcome::VerifiedAtLayer(layer),
            first_mismatch: None,
        }
    }
}

impl RecoveryPlan {
    pub(crate) fn new(
        config: crate::runtime::RelationalRuntimeConfig,
        checkpoint: Option<DurableCheckpoint>,
        tail_log: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
        branch_transitions: super::RecoveryBranchTransitions,
        authority_continuity: RecoveryAuthorityContinuityCheck,
        verification_mode: RecoveryVerificationMode,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        mut restore_authoritative_envelope_commit_ids: Vec<CommitId>,
        cold_source: Option<
            std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
        >,
    ) -> Self {
        restore_authoritative_envelope_commit_ids.sort_unstable();
        restore_authoritative_envelope_commit_ids.dedup();
        let checkpoint_covered_position = checkpoint
            .as_ref()
            .map(crate::durability::access::checkpoint_coverage);
        Self {
            config,
            cold_source,
            checkpoint,
            tail_log,
            branch_transitions,
            authority_continuity,
            verification_plan: RecoveryVerificationPlan::from_mode(verification_mode),
            descriptor_semantics_version,
            restore_authoritative_envelope_commit_ids,
            checkpoint_covered_position,
            commit_strategy_executors:
                crate::commit_strategies::FrozenCommitStrategyExecutorRegistry::default(),
        }
    }

    pub fn verification_mode(&self) -> RecoveryVerificationMode {
        self.verification_plan.mode()
    }

    pub(crate) fn with_commit_strategy_executors(
        mut self,
        executors: crate::commit_strategies::FrozenCommitStrategyExecutorRegistry,
    ) -> Self {
        self.commit_strategy_executors = executors;
        self
    }

    /// The last position the plan's checkpoint covers, fixed when the plan was
    /// built so it survives the rebuild taking the checkpoint image.
    pub fn checkpoint_covered_position(&self) -> Option<PatchStreamPosition> {
        self.checkpoint_covered_position
    }

    pub(crate) fn tail_commit_count(&self) -> usize {
        self.tail_log.len()
    }

    pub(crate) fn tail_envelopes_in_stream_order(
        &self,
    ) -> Vec<&crate::history::data::CanonicalCommitEnvelope> {
        let mut envelopes = self
            .tail_log
            .iter()
            .map(|entry| (entry.position(), entry.envelope()))
            .collect::<Vec<_>>();
        envelopes.sort_by_key(|(position, _)| *position);
        envelopes
            .into_iter()
            .map(|(_, envelope)| envelope)
            .collect()
    }

    pub(crate) fn should_restore_authoritative_envelope(&self, commit_id: CommitId) -> bool {
        self.restore_authoritative_envelope_commit_ids
            .binary_search(&commit_id)
            .is_ok()
    }
}
