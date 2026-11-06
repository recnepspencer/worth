//! Retained decision-fact storage refusals, moved whole from `primary_graph::application_attempt::retained_decision_facts`: the store's denial here, its control and admitted endpoints as children.
use worth_execution::ExecutionAllocationDenial;

pub(crate) mod control;
pub(crate) mod endpoints;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoreDenial {
    Allocation(ExecutionAllocationDenial),
    ExecutionRequest(worth_execution::WorkCeilingDenial),
    RequestInterruption(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption,
    ),
    Representability,
    PhysicalAllocation,
    ConflictingBody,
    ContentInconsistency,
    Content(worth_relational::facade::content::RelationalContentDenial),
    InvalidOrdinal,
}

impl From<ExecutionAllocationDenial> for StoreDenial {
    fn from(denial: ExecutionAllocationDenial) -> Self {
        Self::Allocation(denial)
    }
}

impl std::fmt::Display for StoreDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "retained fact storage refused: {self:?}")
    }
}

impl std::error::Error for StoreDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(denial) => Some(denial),
            _ => None,
        }
    }
}

impl StoreDenial {
    pub(in crate::domain_computation::primary_graph) fn into_attempt_denial(
        self,
        subject: &str,
    ) -> crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial {
        use {
            crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial as Denial,
            crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind as Kind,
        };
        if let Self::Content(cause) = self {
            return Denial::relational_read_stopped(cause.into_execution_cause(), subject);
        }
        let kind = match &self {
            Self::Allocation(_) | Self::PhysicalAllocation => Kind::AllocationDenied,
            Self::RequestInterruption(_) => Kind::CurrentAuthorityDenied,
            Self::ExecutionRequest(_) => Kind::RetainedSourceStorageDenied,
            Self::ConflictingBody => Kind::DecisionDependencyMismatch,
            Self::ContentInconsistency => Kind::ContentInconsistency,
            _ => Kind::RetainedSourceStorageDenied,
        };
        Denial::retained_source_denied(kind, subject, self)
    }
}

impl worth_execution::ChargedBytes for StoreDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // Every current variant contains only inline denial/control data.
        match self {
            Self::Allocation(denial) => {
                worth_execution::ChargedBytes::additional_charged_bytes(denial)
            }
            Self::RequestInterruption(interruption) => {
                fn inline<T: Copy>(_: &T) {}
                inline(interruption);
                0
            }
            Self::ExecutionRequest(cause) => {
                fn inline<T: Copy>(_: &T) {}
                inline(cause);
                0
            }
            Self::Representability
            | Self::ConflictingBody
            | Self::PhysicalAllocation
            | Self::ContentInconsistency
            | Self::Content(_) => 0,
            Self::InvalidOrdinal => 0,
        }
    }
}

impl StoreDenial {
    #[doc(hidden)]
    pub fn into_attempt_denial_for_test(
        self,
        subject: &str,
    ) -> crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial {
        Self::into_attempt_denial(self, subject)
    }
}
