use std::sync::Arc;

pub use worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::{
    WorthQueryProviderExecutionDestructorDisposition,
    WorthQueryProviderExecutionDisposalDisposition,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryProviderExecutionReleaseEvidence {
    disposal: WorthQueryProviderExecutionDisposalDisposition,
    disposal_failure_detail: Option<Arc<str>>,
    destructor: WorthQueryProviderExecutionDestructorDisposition,
}

impl WorthQueryProviderExecutionReleaseEvidence {
    pub(super) fn new(
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
