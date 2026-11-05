use std::sync::Arc;

use {crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionDestructorDisposition, crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionDisposalDisposition, crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionReleaseEvidence};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadmissionRestoredExecutionCleanupInspection {
    disposal: WorthQueryProviderExecutionDisposalDisposition,
    disposal_failure_detail: Option<Arc<str>>,
    destructor: WorthQueryProviderExecutionDestructorDisposition,
}

impl WorthQueryReadmissionRestoredExecutionCleanupInspection {
    pub(in crate::domain_computation::managed_run::readmission) fn capture(
        release: &WorthQueryProviderExecutionReleaseEvidence,
    ) -> Self {
        Self {
            disposal: release.disposal(),
            disposal_failure_detail: release.disposal_failure_detail().map(Arc::from),
            destructor: release.destructor(),
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
