use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::ordinary::WorthQueryOrdinaryInspectionPolicy;
use crate::runtime::{
    CausalInspection, CausalInspectionPlanErrorKind, CausalInspectionProofFlow,
    QueryObservationReceipt, WorthQueryBackendInspectionErrorKind, WorthQueryWorkspace,
};

use super::{
    WorthQueryInspectionCompletion, WorthQueryInspectionCounters,
    WorthQueryInspectionMaterialization, WorthQueryInspectionOutcome, WorthQueryInspectionReceipt,
    WorthQueryInspectionRequest, WorthQueryInspectionStop, WorthQueryInspectionStopSource,
    WorthQueryInspectionUnavailable, WorthQueryInspectionUnavailableSource,
};

impl WorthQueryInspectionRequest {
    pub fn run(
        self,
        workspace: &WorthQueryWorkspace,
    ) -> Result<WorthQueryInspectionOutcome, IdentityDerivationDenial> {
        let counters = WorthQueryInspectionCounters::context_handed_off().planning_attempted();
        let basis = self.context.basis;
        let observation = QueryObservationReceipt::from_read_receipt(
            &self.declaration.source_receipt,
            basis.clone(),
        )?;
        let plan = match CausalInspection::for_observation(observation, basis)
            .reference_only()
            .plan()
        {
            Ok(plan) => plan,
            Err(error) => {
                let source = match error.kind() {
                    CausalInspectionPlanErrorKind::Construction => {
                        WorthQueryInspectionStopSource::Construction
                    }
                    CausalInspectionPlanErrorKind::BasisMismatch => {
                        WorthQueryInspectionStopSource::BasisMismatch
                    }
                    CausalInspectionPlanErrorKind::Anchor => {
                        WorthQueryInspectionStopSource::InvalidOutcomeEvidence
                    }
                    CausalInspectionPlanErrorKind::MissingEvidence => {
                        WorthQueryInspectionStopSource::MissingEvidence
                    }
                    CausalInspectionPlanErrorKind::Request => {
                        WorthQueryInspectionStopSource::InvalidRequest
                    }
                };
                return Ok(WorthQueryInspectionOutcome::Stopped(
                    WorthQueryInspectionStop::new(source, error, counters),
                ));
            }
        };
        let receipt = WorthQueryInspectionReceipt::from_plan(&plan);
        let rich = self.declaration.inspection_policy == WorthQueryOrdinaryInspectionPolicy::Rich;
        let cost = super::WorthQueryInspectionCost::from_plan(&plan, rich);
        let (materialization, counters) = if rich {
            let attempted = counters.materialization_attempted();
            match workspace.materialize_ordinary_inspection(&plan) {
                Ok(artifact) => (
                    Some(WorthQueryInspectionMaterialization::new(artifact)),
                    attempted.materialization_completed(),
                ),
                Err(error) => {
                    let source = match error.kind() {
                        WorthQueryBackendInspectionErrorKind::EvidenceIdentityDerivationDenied => {
                            WorthQueryInspectionUnavailableSource::EvidenceIdentityDerivationDenied
                        }
                        WorthQueryBackendInspectionErrorKind::Construction => {
                            WorthQueryInspectionUnavailableSource::Construction
                        }
                        WorthQueryBackendInspectionErrorKind::Unavailable => {
                            WorthQueryInspectionUnavailableSource::Runtime
                        }
                        WorthQueryBackendInspectionErrorKind::Materialization => {
                            WorthQueryInspectionUnavailableSource::Materialization
                        }
                    };
                    return Ok(WorthQueryInspectionOutcome::Unavailable(
                        WorthQueryInspectionUnavailable::from_backend(
                            source, error, receipt, cost, attempted,
                        ),
                    ));
                }
            }
        } else {
            (None, counters)
        };
        let completion =
            WorthQueryInspectionCompletion::new(receipt, cost, materialization, counters);
        Ok(match plan.admission() {
            CausalInspectionProofFlow::Admitted(_) => {
                WorthQueryInspectionOutcome::Completed(completion)
            }
            CausalInspectionProofFlow::Advisory(_) => {
                WorthQueryInspectionOutcome::Advisory(completion)
            }
            CausalInspectionProofFlow::Denied(_) => {
                WorthQueryInspectionOutcome::Violation(completion)
            }
        })
    }
}
