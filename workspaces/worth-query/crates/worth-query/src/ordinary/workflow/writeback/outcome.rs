use crate::effect_lifecycle::{
    EffectDiagnosticsMaterialization, EffectExecutionReceipt, EffectReceiptTargetEvidence,
};
use crate::runtime::{
    WorthQueryOrdinaryWritebackExecutionError, WorthQueryOrdinaryWritebackFailureStage,
};

use super::{
    WorthQueryAdmittedWorkflowEffect, WorthQueryLoweredWorkflowPlan, WorthQueryWorkflowCounters,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWritebackStopSource {
    Handle(worth_query_execution::facade::primary_graph::WorthQueryHandleDenial),
    ForeignAuthority,
    StaleAuthority,
    InspectionUnavailable,
    Basis,
    Intent,
    Eligibility,
    Lowering,
    BridgeExecution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWritebackNextAction {
    ProvideAuthority,
    RebindCurrentAuthority,
    UseOperationalReceipt,
    InspectDenial,
}

pub struct WorthQueryWritebackStop {
    source: WorthQueryWritebackStopSource,
    message: String,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryWritebackStop {
    pub fn source(&self) -> WorthQueryWritebackStopSource {
        self.source
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn counters(&self) -> &WorthQueryWorkflowCounters {
        &self.counters
    }

    pub fn next_action(&self) -> WorthQueryWritebackNextAction {
        match self.source {
            WorthQueryWritebackStopSource::Handle(_) => {
                WorthQueryWritebackNextAction::InspectDenial
            }
            WorthQueryWritebackStopSource::ForeignAuthority => {
                WorthQueryWritebackNextAction::ProvideAuthority
            }
            WorthQueryWritebackStopSource::StaleAuthority => {
                WorthQueryWritebackNextAction::RebindCurrentAuthority
            }
            WorthQueryWritebackStopSource::InspectionUnavailable => {
                WorthQueryWritebackNextAction::UseOperationalReceipt
            }
            WorthQueryWritebackStopSource::Basis
            | WorthQueryWritebackStopSource::Intent
            | WorthQueryWritebackStopSource::Eligibility
            | WorthQueryWritebackStopSource::Lowering
            | WorthQueryWritebackStopSource::BridgeExecution => {
                WorthQueryWritebackNextAction::InspectDenial
            }
        }
    }

    pub(crate) fn denied(
        source: WorthQueryWritebackStopSource,
        message: impl Into<String>,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self {
            source,
            message: message.into(),
            counters,
        }
    }

    pub(crate) fn from_execution(
        error: WorthQueryOrdinaryWritebackExecutionError,
        counters: WorthQueryWorkflowCounters,
    ) -> Self {
        Self::denied(source_for_stage(error.stage()), error.message(), counters)
    }
}

fn source_for_stage(
    stage: WorthQueryOrdinaryWritebackFailureStage,
) -> WorthQueryWritebackStopSource {
    match stage {
        WorthQueryOrdinaryWritebackFailureStage::Handle(denial) => {
            WorthQueryWritebackStopSource::Handle(denial)
        }
        WorthQueryOrdinaryWritebackFailureStage::Authority => {
            WorthQueryWritebackStopSource::StaleAuthority
        }
        WorthQueryOrdinaryWritebackFailureStage::Basis => WorthQueryWritebackStopSource::Basis,
        WorthQueryOrdinaryWritebackFailureStage::Intent => WorthQueryWritebackStopSource::Intent,
        WorthQueryOrdinaryWritebackFailureStage::Eligibility => {
            WorthQueryWritebackStopSource::Eligibility
        }
        WorthQueryOrdinaryWritebackFailureStage::Lowering => {
            WorthQueryWritebackStopSource::Lowering
        }
        WorthQueryOrdinaryWritebackFailureStage::BridgeExecution => {
            WorthQueryWritebackStopSource::BridgeExecution
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWritebackAftermath {
    target_evidence: EffectReceiptTargetEvidence,
}

impl WorthQueryWritebackAftermath {
    pub fn target_evidence(&self) -> &EffectReceiptTargetEvidence {
        &self.target_evidence
    }

    pub fn outcome_identity_for_reporting(&self) -> &str {
        self.target_evidence
            .writeback_outcome_for_reporting()
            .expect("writeback completion must preserve writeback target evidence")
    }

    pub fn authority_receipt_identity_for_reporting(&self) -> &str {
        self.target_evidence
            .writeback_authority_receipt_for_reporting()
            .expect("writeback completion must preserve authority receipt evidence")
    }

    pub fn execution_receipt_identity_for_reporting(&self) -> &str {
        self.target_evidence
            .writeback_execution_receipt_for_reporting()
            .expect("writeback completion must preserve execution receipt evidence")
    }

    pub(crate) fn new(target_evidence: EffectReceiptTargetEvidence) -> Self {
        Self { target_evidence }
    }
}

pub struct WorthQueryWritebackCompletion {
    admitted_effect: WorthQueryAdmittedWorkflowEffect,
    lowered_plan: WorthQueryLoweredWorkflowPlan,
    receipt: EffectExecutionReceipt,
    aftermath: WorthQueryWritebackAftermath,
    diagnostics: Option<EffectDiagnosticsMaterialization>,
    counters: WorthQueryWorkflowCounters,
}

impl WorthQueryWritebackCompletion {
    pub fn admitted_effect(&self) -> &WorthQueryAdmittedWorkflowEffect {
        &self.admitted_effect
    }

    pub fn lowered_plan(&self) -> &WorthQueryLoweredWorkflowPlan {
        &self.lowered_plan
    }

    pub fn receipt(&self) -> &EffectExecutionReceipt {
        &self.receipt
    }

    pub fn aftermath(&self) -> &WorthQueryWritebackAftermath {
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
        aftermath: WorthQueryWritebackAftermath,
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

pub enum WorthQueryWritebackOutcome {
    Completed(WorthQueryWritebackCompletion),
    Stopped(WorthQueryWritebackStop),
}

impl WorthQueryWritebackOutcome {
    pub fn completed(&self) -> Option<&WorthQueryWritebackCompletion> {
        match self {
            Self::Completed(completion) => Some(completion),
            Self::Stopped(_) => None,
        }
    }

    pub fn stop(&self) -> Option<&WorthQueryWritebackStop> {
        match self {
            Self::Completed(_) => None,
            Self::Stopped(stop) => Some(stop),
        }
    }
}
