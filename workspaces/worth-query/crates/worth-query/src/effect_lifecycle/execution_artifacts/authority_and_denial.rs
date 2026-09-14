use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::RuntimeBridge;

use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::counters::EffectLifecycleCounters;
use super::super::lowering::LoweredEffectExecutionPlan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectExecutionDenialKind {
    AuthorityOverrideRejected,
    MissingRelationalAuthority,
    MissingBridgeAuthority,
    BridgePolicyAdmissionFailed,
    BridgeWritebackExecutionFailed,
    RelationalAuthorityBindingMalformed,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
    CandidateIdentityExhausted,
    TransactionRetentionOwnerUnavailable,
    TransactionRetentionIdentityExhausted,
    TransactionRetentionInvariantViolation,
    TransactionSavepointIdentityExhausted,
    TransactionMaterializationAuthorityRequired,
    TransactionMaterializationModeMismatch,
    TransactionOverlayBudgetExceeded {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    TransactionFootprintBudgetExceeded {
        maximum_loci: usize,
        required_loci: usize,
    },
    TransactionSavepointBudgetExceeded {
        maximum_savepoints: usize,
    },
    TransactionSavepointFootprintBudgetExceeded {
        maximum_loci: usize,
        required_loci: usize,
    },
    PreparedRootBudgetExceeded {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    RelationalExactBasisStale,
    RelationalStrategyCanonicalizationFailed,
    RelationalStrategyExecutionFailed,
    RelationalStrategyAuthorityLoweringFailed,
    RelationalStrategyAuthorityValidationFailed,
    RelationalCommitFailed,
    MergePreparationFailed,
    MergeExecutionFailed,
}

impl EffectExecutionDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorityOverrideRejected => "authority_override_rejected",
            Self::MissingRelationalAuthority => "missing_relational_authority",
            Self::MissingBridgeAuthority => "missing_bridge_authority",
            Self::BridgePolicyAdmissionFailed => "bridge_policy_admission_failed",
            Self::BridgeWritebackExecutionFailed => "bridge_writeback_execution_failed",
            Self::RelationalAuthorityBindingMalformed => "relational_authority_binding_malformed",
            Self::ActiveSnapshotCapacityExhausted { .. } => "active_snapshot_capacity_exhausted",
            Self::SnapshotIdentityExhausted => "snapshot_identity_exhausted",
            Self::CandidateIdentityExhausted => "candidate_identity_exhausted",
            Self::TransactionRetentionOwnerUnavailable => "transaction_retention_owner_unavailable",
            Self::TransactionRetentionIdentityExhausted => {
                "transaction_retention_identity_exhausted"
            }
            Self::TransactionRetentionInvariantViolation => {
                "transaction_retention_invariant_violation"
            }
            Self::TransactionSavepointIdentityExhausted => {
                "transaction_savepoint_identity_exhausted"
            }
            Self::TransactionMaterializationAuthorityRequired => {
                "transaction_materialization_authority_required"
            }
            Self::TransactionMaterializationModeMismatch => {
                "transaction_materialization_mode_mismatch"
            }
            Self::TransactionOverlayBudgetExceeded { .. } => "transaction_overlay_budget_exceeded",
            Self::TransactionFootprintBudgetExceeded { .. } => {
                "transaction_footprint_budget_exceeded"
            }
            Self::TransactionSavepointBudgetExceeded { .. } => {
                "transaction_savepoint_budget_exceeded"
            }
            Self::TransactionSavepointFootprintBudgetExceeded { .. } => {
                "transaction_savepoint_footprint_budget_exceeded"
            }
            Self::PreparedRootBudgetExceeded { .. } => "prepared_root_budget_exceeded",
            Self::RelationalExactBasisStale => "relational_exact_basis_stale",
            Self::RelationalStrategyCanonicalizationFailed => {
                "relational_strategy_canonicalization_failed"
            }
            Self::RelationalStrategyExecutionFailed => "relational_strategy_execution_failed",
            Self::RelationalStrategyAuthorityLoweringFailed => {
                "relational_strategy_authority_lowering_failed"
            }
            Self::RelationalStrategyAuthorityValidationFailed => {
                "relational_strategy_authority_validation_failed"
            }
            Self::RelationalCommitFailed => "relational_commit_failed",
            Self::MergePreparationFailed => "merge_preparation_failed",
            Self::MergeExecutionFailed => "merge_execution_failed",
        }
    }
}

#[derive(Debug)]
pub struct EffectExecutionAuthority<'a> {
    relational: Option<&'a mut RelationalRuntime>,
    bridge: Option<&'a RuntimeBridge>,
}

impl<'a> EffectExecutionAuthority<'a> {
    pub fn relational(runtime: &'a mut RelationalRuntime) -> Self {
        Self {
            relational: Some(runtime),
            bridge: None,
        }
    }

    pub fn bridge(runtime: &'a RuntimeBridge) -> Self {
        Self {
            relational: None,
            bridge: Some(runtime),
        }
    }

    pub fn combined(relational: &'a mut RelationalRuntime, bridge: &'a RuntimeBridge) -> Self {
        Self {
            relational: Some(relational),
            bridge: Some(bridge),
        }
    }

    pub fn repair_pending_settlement(
        self,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        super::super::EffectSettlementRepairError,
    > {
        super::super::settlement_repair::repair_pending_effect_settlement(self, commit_id)
    }

    pub(crate) fn relational_runtime(&mut self) -> Option<&mut RelationalRuntime> {
        self.relational.as_deref_mut()
    }

    pub(crate) fn has_relational_authority(&self) -> bool {
        self.relational.is_some()
    }

    pub(crate) fn bridge_runtime(&self) -> Option<&RuntimeBridge> {
        self.bridge
    }

    pub(crate) fn has_bridge_authority(&self) -> bool {
        self.bridge.is_some()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecutionDenial {
    denial_kind: EffectExecutionDenialKind,
    message: String,
    lowered_effect_execution_plan_identity: WorthQueryEvidenceIdentity,
    denial_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl EffectExecutionDenial {
    pub(crate) fn new(
        lowered: &LoweredEffectExecutionPlan,
        denial_kind: EffectExecutionDenialKind,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let plan_identity = lowered.lowered_effect_execution_plan_identity().clone();
        let denial_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_denial_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), &plan_identity)
                .field_shape(WorthQueryEvidenceTag::new("kind"), denial_kind.as_str())
                .field_shape(WorthQueryEvidenceTag::new("message"), message.as_str())
                .seal();
        Self {
            denial_kind,
            message,
            lowered_effect_execution_plan_identity: plan_identity,
            denial_identity,
            counters: EffectLifecycleCounters::execution_denied(
                lowered.counters().effect_support_row_count(),
                lowered.counters().effect_lowering_width(),
                lowered.counters().effect_executor_rediscovery_count(),
            ),
        }
    }

    pub fn denial_kind(&self) -> EffectExecutionDenialKind {
        self.denial_kind
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn lowered_effect_execution_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_effect_execution_plan_identity
    }
    pub fn lowered_effect_execution_plan_for_reporting(&self) -> &str {
        self.lowered_effect_execution_plan_identity.as_str()
    }
    pub fn denial_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.denial_identity
    }
    pub fn denial_for_reporting(&self) -> &str {
        self.denial_identity.as_str()
    }
    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}
