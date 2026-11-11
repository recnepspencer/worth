use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryAuthorityLane, WorthQueryRuntimeError, WorthQueryWriteReceipt};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryMutationCounters {
    context_validation_count: usize,
    lower_runtime_execution_attempt_count: usize,
    lower_runtime_execution_completed_count: usize,
    inspection_materialization_count: usize,
}

impl WorthQueryMutationCounters {
    pub fn context_validation_count(&self) -> usize {
        self.context_validation_count
    }

    pub fn lower_runtime_execution_attempt_count(&self) -> usize {
        self.lower_runtime_execution_attempt_count
    }

    pub fn lower_runtime_execution_completed_count(&self) -> usize {
        self.lower_runtime_execution_completed_count
    }

    pub fn inspection_materialization_count(&self) -> usize {
        self.inspection_materialization_count
    }

    pub(crate) fn context_checked() -> Self {
        Self {
            context_validation_count: 1,
            ..Self::default()
        }
    }

    pub(crate) fn execution_attempted(mut self) -> Self {
        self.lower_runtime_execution_attempt_count += 1;
        self
    }

    pub(crate) fn execution_completed(mut self, inspection_materialized: bool) -> Self {
        self.lower_runtime_execution_completed_count += 1;
        self.inspection_materialization_count += usize::from(inspection_materialized);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLoweredMutationPlan {
    request_identity: WorthQueryEvidenceIdentity,
    handoff_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLoweredMutationPlan {
    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn handoff_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.handoff_identity
    }

    pub(crate) fn new(
        request_identity: WorthQueryEvidenceIdentity,
        handoff_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self {
            request_identity,
            handoff_identity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationAftermath {
    authority_lane: WorthQueryAuthorityLane,
    receipt_identity: WorthQueryEvidenceIdentity,
    inspection_identity: Option<WorthQueryEvidenceIdentity>,
    aftermath_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryMutationAftermath {
    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn inspection_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.inspection_identity.as_ref()
    }

    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.aftermath_identity
    }

    pub(crate) fn new(
        receipt: &WorthQueryWriteReceipt,
        receipt_identity: WorthQueryEvidenceIdentity,
        inspection_identity: Option<WorthQueryEvidenceIdentity>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let aftermath_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(WorthQueryEvidenceTag::new("role"), "mutation-aftermath")
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_lane"),
                    receipt.authority_lane().as_str(),
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), &receipt_identity)
                .seal()?;
        Ok(Self {
            authority_lane: receipt.authority_lane(),
            receipt_identity,
            inspection_identity,
            aftermath_identity,
        })
    }
}

pub struct WorthQueryMutationCompletion {
    plan: WorthQueryLoweredMutationPlan,
    receipt: WorthQueryWriteReceipt,
    aftermath: WorthQueryMutationAftermath,
    counters: WorthQueryMutationCounters,
}

impl WorthQueryMutationCompletion {
    pub fn lowered_plan(&self) -> &WorthQueryLoweredMutationPlan {
        &self.plan
    }

    pub fn receipt(&self) -> &WorthQueryWriteReceipt {
        &self.receipt
    }

    pub fn aftermath(&self) -> &WorthQueryMutationAftermath {
        &self.aftermath
    }

    pub fn counters(&self) -> &WorthQueryMutationCounters {
        &self.counters
    }

    pub(crate) fn new(
        plan: WorthQueryLoweredMutationPlan,
        receipt: WorthQueryWriteReceipt,
        aftermath: WorthQueryMutationAftermath,
        counters: WorthQueryMutationCounters,
    ) -> Self {
        Self {
            plan,
            receipt,
            aftermath,
            counters,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMutationStopSource {
    ForeignAuthority,
    StaleBasis,
    InspectionUnavailable,
    LowerRuntime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMutationNextAction {
    ReviseDeclaration,
    ProvideAuthority,
    RefreshContext,
    UseOperationalReceipt,
    InspectRuntimeDenial,
}

pub struct WorthQueryMutationStop {
    source: WorthQueryMutationStopSource,
    error: Option<WorthQueryRuntimeError>,
    counters: WorthQueryMutationCounters,
}

impl WorthQueryMutationStop {
    pub fn source(&self) -> WorthQueryMutationStopSource {
        self.source
    }

    pub fn error(&self) -> Option<&WorthQueryRuntimeError> {
        self.error.as_ref()
    }

    pub fn counters(&self) -> &WorthQueryMutationCounters {
        &self.counters
    }

    pub fn next_action(&self) -> WorthQueryMutationNextAction {
        match self.source {
            WorthQueryMutationStopSource::ForeignAuthority => {
                WorthQueryMutationNextAction::ProvideAuthority
            }
            WorthQueryMutationStopSource::StaleBasis => {
                WorthQueryMutationNextAction::RefreshContext
            }
            WorthQueryMutationStopSource::InspectionUnavailable => {
                WorthQueryMutationNextAction::UseOperationalReceipt
            }
            WorthQueryMutationStopSource::LowerRuntime => {
                WorthQueryMutationNextAction::InspectRuntimeDenial
            }
        }
    }

    pub(crate) fn authority(
        source: WorthQueryMutationStopSource,
        counters: WorthQueryMutationCounters,
    ) -> Self {
        Self {
            source,
            error: None,
            counters,
        }
    }

    pub(crate) fn runtime(
        error: WorthQueryRuntimeError,
        counters: WorthQueryMutationCounters,
    ) -> Self {
        Self {
            source: WorthQueryMutationStopSource::LowerRuntime,
            error: Some(error),
            counters,
        }
    }

    pub(crate) fn inspection_unavailable(
        error: WorthQueryRuntimeError,
        counters: WorthQueryMutationCounters,
    ) -> Self {
        Self {
            source: WorthQueryMutationStopSource::InspectionUnavailable,
            error: Some(error),
            counters,
        }
    }
}

pub enum WorthQueryMutationOutcome {
    Completed(WorthQueryMutationCompletion),
    Stopped(WorthQueryMutationStop),
}

impl WorthQueryMutationOutcome {
    pub fn completed(&self) -> Option<&WorthQueryMutationCompletion> {
        match self {
            Self::Completed(completion) => Some(completion),
            Self::Stopped(_) => None,
        }
    }

    pub fn stop(&self) -> Option<&WorthQueryMutationStop> {
        match self {
            Self::Completed(_) => None,
            Self::Stopped(stop) => Some(stop),
        }
    }
}
