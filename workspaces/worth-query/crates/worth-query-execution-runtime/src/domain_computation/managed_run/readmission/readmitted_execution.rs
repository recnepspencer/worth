use super::super::{WorthQueryActiveDirectGraphExecution, WorthQueryActiveWorkflowGraphExecution};
use super::WorthQueryReadmissionEvidence;

#[must_use = "a readmitted direct execution must enter the managed-run lifecycle"]
pub struct WorthQueryReadmittedDirectGraphExecution {
    active: WorthQueryActiveDirectGraphExecution,
    evidence: WorthQueryReadmissionEvidence,
}

#[must_use = "a readmitted workflow execution must enter the managed-run lifecycle"]
pub struct WorthQueryReadmittedWorkflowGraphExecution {
    active: WorthQueryActiveWorkflowGraphExecution,
    evidence: WorthQueryReadmissionEvidence,
}

impl WorthQueryReadmittedDirectGraphExecution {
    pub(super) fn new(
        active: WorthQueryActiveDirectGraphExecution,
        evidence: WorthQueryReadmissionEvidence,
    ) -> Self {
        Self { active, evidence }
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub const fn active(&self) -> &WorthQueryActiveDirectGraphExecution {
        &self.active
    }

    pub fn into_active(self) -> WorthQueryActiveDirectGraphExecution {
        self.active
    }
}

impl WorthQueryReadmittedWorkflowGraphExecution {
    pub(super) fn new(
        active: WorthQueryActiveWorkflowGraphExecution,
        evidence: WorthQueryReadmissionEvidence,
    ) -> Self {
        Self { active, evidence }
    }

    pub const fn readmission_evidence(&self) -> WorthQueryReadmissionEvidence {
        self.evidence
    }

    pub const fn active(&self) -> &WorthQueryActiveWorkflowGraphExecution {
        &self.active
    }

    pub fn into_active(self) -> WorthQueryActiveWorkflowGraphExecution {
        self.active
    }
}
