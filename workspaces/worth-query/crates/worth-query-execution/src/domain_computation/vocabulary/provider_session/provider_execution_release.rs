//! Moved whole from `domain_computation::provider_session::graph_provider::bounded_step::provider_execution_release`: the items below and every impl they own.
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderExecutionDisposalDisposition {
    Completed,
    Rejected,
    Panicked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderExecutionDestructorDisposition {
    Completed,
    Panicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderExecutionReleaseEvidence {
    disposal: WorthQueryProviderExecutionDisposalDisposition,
    disposal_failure_detail: Option<Arc<str>>,
    destructor: WorthQueryProviderExecutionDestructorDisposition,
}

impl WorthQueryProviderExecutionReleaseEvidence {
    pub(in crate::domain_computation) fn new(
        disposal: WorthQueryProviderExecutionDisposalDisposition,
        disposal_failure_detail: Option<Arc<str>>,
        destructor: WorthQueryProviderExecutionDestructorDisposition,
    ) -> Self {
        Self {
            disposal,
            disposal_failure_detail,
            destructor,
        }
    }

    pub const fn disposal(&self) -> WorthQueryProviderExecutionDisposalDisposition {
        self.disposal
    }

    pub fn disposal_failure_detail(&self) -> Option<&str> {
        self.disposal_failure_detail.as_deref()
    }

    pub const fn destructor(&self) -> WorthQueryProviderExecutionDestructorDisposition {
        self.destructor
    }

    pub const fn recovery_required(&self) -> bool {
        !matches!(
            self.disposal,
            WorthQueryProviderExecutionDisposalDisposition::Completed
        ) || matches!(
            self.destructor,
            WorthQueryProviderExecutionDestructorDisposition::Panicked
        )
    }
}
