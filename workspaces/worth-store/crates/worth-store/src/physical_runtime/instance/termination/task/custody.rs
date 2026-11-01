use crate::physical_runtime::RuntimeIdentity;
use worth_proof::{LinearResource, TerminalState};

worth_proof::authority_marker!(PhysicalCloseTaskAuthority);

enum PhysicalCloseTaskDisposition {
    OutcomeDelivered,
    FailureEvidenceDelivered,
    ObservationAbandoned,
}

impl TerminalState for PhysicalCloseTaskDisposition {
    fn label(&self) -> &'static str {
        match self {
            Self::OutcomeDelivered => "outcome-delivered",
            Self::FailureEvidenceDelivered => "failure-evidence-delivered",
            Self::ObservationAbandoned => "observation-abandoned",
        }
    }
}

type TaskResource =
    LinearResource<RuntimeIdentity, PhysicalCloseTaskDisposition, PhysicalCloseTaskAuthority>;

// This resource describes caller custody, not the worker's Store effect
// authority. Observer abandonment never certifies or cancels a physical close.
pub(super) struct PhysicalCloseTaskCustody {
    resource: Option<TaskResource>,
}

impl PhysicalCloseTaskCustody {
    pub(super) fn accepted(identity: RuntimeIdentity) -> Self {
        Self {
            resource: Some(LinearResource::mint(
                identity,
                &PhysicalCloseTaskAuthority::witness(),
            )),
        }
    }

    pub(super) fn identity(&self) -> RuntimeIdentity {
        *self.resource.as_ref().expect("live caller custody").id()
    }

    pub(super) fn outcome_delivered(mut self) {
        let _receipt = self
            .resource
            .take()
            .expect("consumed caller custody")
            .terminate(PhysicalCloseTaskDisposition::OutcomeDelivered);
    }

    pub(super) fn failure_evidence_delivered(mut self) {
        let _receipt = self
            .resource
            .take()
            .expect("consumed failure custody")
            .terminate(PhysicalCloseTaskDisposition::FailureEvidenceDelivered);
    }
}

impl Drop for PhysicalCloseTaskCustody {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let _receipt = resource.terminate(PhysicalCloseTaskDisposition::ObservationAbandoned);
        }
    }
}
