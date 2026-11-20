use crate::domain_computation::{
    WorthQueryProviderExecutionDestructorDisposition,
    WorthQueryProviderExecutionDisposalDisposition, WorthQueryProviderExecutionReleaseEvidence,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryManagedProviderExecutionReleaseSummary {
    release_count: usize,
    completed_disposal_count: usize,
    rejected_disposal_count: usize,
    panicked_disposal_count: usize,
    completed_destructor_count: usize,
    panicked_destructor_count: usize,
    recovery_evidence: Option<WorthQueryProviderExecutionReleaseEvidence>,
}

impl WorthQueryManagedProviderExecutionReleaseSummary {
    pub fn record(&mut self, evidence: &WorthQueryProviderExecutionReleaseEvidence) {
        self.release_count = self.release_count.saturating_add(1);
        match evidence.disposal() {
            WorthQueryProviderExecutionDisposalDisposition::Completed => {
                self.completed_disposal_count = self.completed_disposal_count.saturating_add(1);
            }
            WorthQueryProviderExecutionDisposalDisposition::Rejected => {
                self.rejected_disposal_count = self.rejected_disposal_count.saturating_add(1);
            }
            WorthQueryProviderExecutionDisposalDisposition::Panicked => {
                self.panicked_disposal_count = self.panicked_disposal_count.saturating_add(1);
            }
        }
        match evidence.destructor() {
            WorthQueryProviderExecutionDestructorDisposition::Completed => {
                self.completed_destructor_count = self.completed_destructor_count.saturating_add(1);
            }
            WorthQueryProviderExecutionDestructorDisposition::Panicked => {
                self.panicked_destructor_count = self.panicked_destructor_count.saturating_add(1);
            }
        }
        if evidence.recovery_required() {
            debug_assert!(
                self.recovery_evidence.is_none(),
                "managed execution terminalizes after its first physical-release failure"
            );
            self.recovery_evidence = Some(evidence.clone());
        }
    }

    pub const fn release_count(&self) -> usize {
        self.release_count
    }

    pub const fn completed_disposal_count(&self) -> usize {
        self.completed_disposal_count
    }

    pub const fn rejected_disposal_count(&self) -> usize {
        self.rejected_disposal_count
    }

    pub const fn panicked_disposal_count(&self) -> usize {
        self.panicked_disposal_count
    }

    pub const fn completed_destructor_count(&self) -> usize {
        self.completed_destructor_count
    }

    pub const fn panicked_destructor_count(&self) -> usize {
        self.panicked_destructor_count
    }

    pub fn recovery_evidence(&self) -> Option<&WorthQueryProviderExecutionReleaseEvidence> {
        self.recovery_evidence.as_ref()
    }

    pub const fn recovery_required(&self) -> bool {
        self.recovery_evidence.is_some()
    }
}
