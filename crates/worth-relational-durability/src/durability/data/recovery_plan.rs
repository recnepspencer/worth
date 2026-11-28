use serde::{Deserialize, Serialize};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port as storage;

use super::DurableCheckpoint;
use crate::replay::data::ReplayVerificationLayer;
use contracts::durability::data::recovery_errors::RecoveryAuthorityContinuityMismatch;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion;

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

pub use contracts::durability::data::recovery_verification::RecoveryVerificationMode;

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
    pub(crate) home_world_selection: Option<
        std::sync::Arc<
            worth_proof::VerifiedHomeWorldSelection<
                std::sync::Arc<crate::durability::home_world_catalog::RelationalWorldSourceCatalog>,
            >,
        >,
    >,
    /// Actual retained Memory transition DATA, independent of external-source admission.
    pub(crate) retained_transition_prefix: Option<
        std::sync::Arc<
            crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputs,
        >,
    >,
    pub(crate) original_source: Option<
        std::sync::Arc<
            crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs,
        >,
    >,
    pub config: contracts::config::data::runtime_config::RelationalRuntimeConfig,
    pub(crate) cold_source: Option<
        std::sync::Arc<dyn storage::durability::medium::cold_source::RelationalDurableColdSource>,
    >,
    pub checkpoint: Option<DurableCheckpoint>,
    pub(crate) selected_checkpoint_acknowledged: bool,
    pub(crate) tail_log: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
    pub(crate) branch_transitions: super::RecoveryBranchTransitions,
    pub authority_continuity: RecoveryAuthorityContinuityCheck,
    pub verification_plan: RecoveryVerificationPlan,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
    pub restore_authoritative_envelope_commit_ids: Vec<CommitId>,
    checkpoint_covered_position: Option<PatchStreamPosition>,
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
        config: contracts::config::data::runtime_config::RelationalRuntimeConfig,
        checkpoint: Option<DurableCheckpoint>,
        tail_log: Vec<crate::durability::migration::ReadmittedCanonicalCommit>,
        branch_transitions: super::RecoveryBranchTransitions,
        authority_continuity: RecoveryAuthorityContinuityCheck,
        verification_mode: RecoveryVerificationMode,
        descriptor_semantics_version: DescriptorSemanticsVersion,
        mut restore_authoritative_envelope_commit_ids: Vec<CommitId>,
        cold_source: Option<
            std::sync::Arc<
                dyn storage::durability::medium::cold_source::RelationalDurableColdSource,
            >,
        >,
    ) -> Self {
        restore_authoritative_envelope_commit_ids.sort_unstable();
        restore_authoritative_envelope_commit_ids.dedup();
        let checkpoint_covered_position = checkpoint
            .as_ref()
            .map(crate::durability::checkpoint_coverage::checkpoint_coverage);
        Self {
            home_world_selection: None,
            original_source: None,
            retained_transition_prefix: None,
            config,
            cold_source,
            checkpoint,
            selected_checkpoint_acknowledged: false,
            tail_log,
            branch_transitions,
            authority_continuity,
            verification_plan: RecoveryVerificationPlan::from_mode(verification_mode),
            descriptor_semantics_version,
            restore_authoritative_envelope_commit_ids,
            checkpoint_covered_position,
        }
    }

    pub fn verification_mode(&self) -> RecoveryVerificationMode {
        self.verification_plan.mode()
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
    ) -> Vec<&branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope> {
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

mod owner_port;
pub use owner_port::{RecoveryPlanFixturePort, RecoveryPlanOwnerPort};

mod fixture_port;
pub use fixture_port::RecoveryPlanFieldsFixturePort;
