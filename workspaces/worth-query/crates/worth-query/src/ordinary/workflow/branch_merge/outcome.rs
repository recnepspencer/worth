use crate::effect_lifecycle::{
    EffectDiagnosticsMaterialization, EffectExecutionReceipt, EffectReceiptTargetEvidence,
};
use crate::runtime::WorthQueryOrdinaryMergeFailureStage;
use crate::WorthQueryEvidenceIdentity;

use crate::ordinary::workflow::{
    WorthQueryAdmittedWorkflowEffect, WorthQueryLoweredWorkflowPlan, WorthQueryWorkflowCounters,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchMergeStopSource {
    ForeignAuthority,
    StaleAuthority,
    MismatchedContext,
    InspectionUnavailable,
    Basis,
    Intent,
    Eligibility,
    Lowering,
    RelationalExecution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchMergeNextAction {
    ProvideCurrentBranchAuthority,
    RebindCurrentBranches,
    UseMatchingDeclaration,
    UseOperationalReceipt,
    InspectDenial,
    RetryDeferredPublication,
    /// Reopen the Relational owner; reopening resolves the commit's durable fate.
    ReopenRelationalOwner,
}

pub struct WorthQueryBranchMergeDeferred {
    kind: crate::effect_lifecycle::EffectExecutionDeferredKind,
    message: String,
    counters: WorthQueryWorkflowCounters,
}

pub struct WorthQueryBranchMergeControlStopped {
    kind: crate::effect_lifecycle::EffectExecutionControlStopKind,
    message: String,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryBranchMergeControlStopped {
    pub(crate) fn new(
        kind: crate::effect_lifecycle::EffectExecutionControlStopKind,
        message: String,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            kind,
            message,
            counters,
        }
    }

    pub const fn kind(&self) -> crate::effect_lifecycle::EffectExecutionControlStopKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }
}

impl WorthQueryBranchMergeDeferred {
    pub(crate) fn new(
        kind: crate::effect_lifecycle::EffectExecutionDeferredKind,
        message: String,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            kind,
            message,
            counters,
        }
    }

    pub const fn kind(&self) -> crate::effect_lifecycle::EffectExecutionDeferredKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    pub fn next_action(&self) -> WorthQueryBranchMergeNextAction {
        WorthQueryBranchMergeNextAction::RetryDeferredPublication
    }
}

pub struct WorthQueryBranchMergeStop {
    source: WorthQueryBranchMergeStopSource,
    effect_kind: Option<crate::effect_lifecycle::EffectExecutionDenialKind>,
    message: String,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryBranchMergeStop {
    pub fn source(&self) -> WorthQueryBranchMergeStopSource {
        self.source
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn effect_kind(&self) -> Option<crate::effect_lifecycle::EffectExecutionDenialKind> {
        self.effect_kind
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    pub fn next_action(&self) -> WorthQueryBranchMergeNextAction {
        match self.source {
            WorthQueryBranchMergeStopSource::ForeignAuthority => {
                WorthQueryBranchMergeNextAction::ProvideCurrentBranchAuthority
            }
            WorthQueryBranchMergeStopSource::StaleAuthority => {
                WorthQueryBranchMergeNextAction::RebindCurrentBranches
            }
            WorthQueryBranchMergeStopSource::MismatchedContext => {
                WorthQueryBranchMergeNextAction::UseMatchingDeclaration
            }
            WorthQueryBranchMergeStopSource::InspectionUnavailable => {
                WorthQueryBranchMergeNextAction::UseOperationalReceipt
            }
            WorthQueryBranchMergeStopSource::Basis
            | WorthQueryBranchMergeStopSource::Intent
            | WorthQueryBranchMergeStopSource::Eligibility
            | WorthQueryBranchMergeStopSource::Lowering
            | WorthQueryBranchMergeStopSource::RelationalExecution => {
                WorthQueryBranchMergeNextAction::InspectDenial
            }
        }
    }

    pub(crate) fn denied(
        source: WorthQueryBranchMergeStopSource,
        message: impl Into<String>,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            source,
            effect_kind: None,
            message: message.into(),
            counters,
        }
    }

    pub(crate) fn from_execution_denial(
        stage: WorthQueryOrdinaryMergeFailureStage,
        effect_kind: Option<crate::effect_lifecycle::EffectExecutionDenialKind>,
        message: String,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            source: source_for_stage(stage),
            effect_kind,
            message,
            counters,
        }
    }
}

/// The merge commit's durable fate is unknown. Nothing was installed live, but
/// the commit may be durable; the Relational owner refuses further publication
/// until it is reopened.
pub struct WorthQueryBranchMergeDurabilityUnresolved {
    message: String,
    counters: WorthQueryWorkflowCounters,
    unresolved: worth_relational::facade::durability::RelationalDurabilityUnresolved,
}

impl WorthQueryBranchMergeDurabilityUnresolved {
    pub(crate) fn new(
        message: String,
        unresolved: worth_relational::facade::durability::RelationalDurabilityUnresolved,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            message,
            counters: counters.durability_unresolved(),
            unresolved,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    /// The unresolved durable publication this outcome retains.
    pub fn unresolved(
        &self,
    ) -> &worth_relational::facade::durability::RelationalDurabilityUnresolved {
        &self.unresolved
    }

    pub fn next_action(&self) -> WorthQueryBranchMergeNextAction {
        WorthQueryBranchMergeNextAction::ReopenRelationalOwner
    }

    /// The unresolved commit, or `None` when the publication blocking the
    /// merge is an unresolved branch transition.
    pub fn commit_id(&self) -> Option<worth_relational::facade::history::CommitId> {
        crate::effect_lifecycle::unresolved_commit_id(&self.unresolved)
    }
}

fn source_for_stage(stage: WorthQueryOrdinaryMergeFailureStage) -> WorthQueryBranchMergeStopSource {
    match stage {
        WorthQueryOrdinaryMergeFailureStage::Basis => WorthQueryBranchMergeStopSource::Basis,
        WorthQueryOrdinaryMergeFailureStage::Intent => WorthQueryBranchMergeStopSource::Intent,
        WorthQueryOrdinaryMergeFailureStage::Eligibility => {
            WorthQueryBranchMergeStopSource::Eligibility
        }
        WorthQueryOrdinaryMergeFailureStage::Lowering => WorthQueryBranchMergeStopSource::Lowering,
        WorthQueryOrdinaryMergeFailureStage::RelationalExecution => {
            WorthQueryBranchMergeStopSource::RelationalExecution
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBranchMergeAftermath {
    commit_id: u64,
    version_id: u64,
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBranchMergeAftermath {
    pub fn commit_id(&self) -> u64 {
        self.commit_id
    }

    pub fn version_id(&self) -> u64 {
        self.version_id
    }

    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn identity_for_reporting(&self) -> &str {
        self.identity.as_str()
    }

    pub(crate) fn from_receipt(receipt: &EffectExecutionReceipt) -> Option<Self> {
        match receipt.target_evidence() {
            EffectReceiptTargetEvidence::MergeCommit {
                commit_id,
                version_id,
            } => Some(Self {
                commit_id,
                version_id,
                identity: receipt
                    .integrity_markers()
                    .authority_artifact_identity()
                    .clone(),
            }),
            _ => None,
        }
    }
}

pub struct WorthQueryBranchMergeCompletion {
    admitted_effect: WorthQueryAdmittedWorkflowEffect,
    lowered_plan: WorthQueryLoweredWorkflowPlan,
    receipt: EffectExecutionReceipt,
    aftermath: WorthQueryBranchMergeAftermath,
    diagnostics: Option<EffectDiagnosticsMaterialization>,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryBranchMergeCompletion {
    pub fn admitted_effect(&self) -> &WorthQueryAdmittedWorkflowEffect {
        &self.admitted_effect
    }

    pub fn lowered_plan(&self) -> &WorthQueryLoweredWorkflowPlan {
        &self.lowered_plan
    }

    pub fn receipt(&self) -> &EffectExecutionReceipt {
        &self.receipt
    }

    pub fn aftermath(&self) -> &WorthQueryBranchMergeAftermath {
        &self.aftermath
    }

    pub fn diagnostics(&self) -> Option<&EffectDiagnosticsMaterialization> {
        self.diagnostics.as_ref()
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    pub(crate) fn new(
        admitted_effect: WorthQueryAdmittedWorkflowEffect,
        lowered_plan: WorthQueryLoweredWorkflowPlan,
        receipt: EffectExecutionReceipt,
        aftermath: WorthQueryBranchMergeAftermath,
        diagnostics: Option<EffectDiagnosticsMaterialization>,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            admitted_effect,
            lowered_plan,
            receipt,
            aftermath,
            diagnostics,
            counters,
        }
    }
}

pub enum WorthQueryBranchMergeOutcome {
    Completed(WorthQueryBranchMergeCompletion),
    Stopped(WorthQueryBranchMergeStop),
    Deferred(WorthQueryBranchMergeDeferred),
    ControlStopped(WorthQueryBranchMergeControlStopped),
    DurabilityUnresolved(WorthQueryBranchMergeDurabilityUnresolved),
}

impl WorthQueryBranchMergeOutcome {
    pub fn completed(&self) -> Option<&WorthQueryBranchMergeCompletion> {
        match self {
            Self::Completed(completion) => Some(completion),
            Self::Stopped(_)
            | Self::Deferred(_)
            | Self::ControlStopped(_)
            | Self::DurabilityUnresolved(_) => None,
        }
    }

    pub fn stop(&self) -> Option<&WorthQueryBranchMergeStop> {
        match self {
            Self::Completed(_) => None,
            Self::Stopped(stop) => Some(stop),
            Self::Deferred(_) | Self::ControlStopped(_) | Self::DurabilityUnresolved(_) => None,
        }
    }

    pub fn deferred(&self) -> Option<&WorthQueryBranchMergeDeferred> {
        match self {
            Self::Deferred(deferred) => Some(deferred),
            Self::Completed(_)
            | Self::Stopped(_)
            | Self::ControlStopped(_)
            | Self::DurabilityUnresolved(_) => None,
        }
    }

    pub fn control_stopped(&self) -> Option<&WorthQueryBranchMergeControlStopped> {
        match self {
            Self::ControlStopped(stopped) => Some(stopped),
            Self::Completed(_)
            | Self::Stopped(_)
            | Self::Deferred(_)
            | Self::DurabilityUnresolved(_) => None,
        }
    }

    pub fn durability_unresolved(&self) -> Option<&WorthQueryBranchMergeDurabilityUnresolved> {
        match self {
            Self::Completed(_) | Self::Stopped(_) | Self::Deferred(_) | Self::ControlStopped(_) => {
                None
            }
            Self::DurabilityUnresolved(unresolved) => Some(unresolved),
        }
    }
}
