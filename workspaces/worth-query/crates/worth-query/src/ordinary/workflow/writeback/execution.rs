use crate::ordinary::workflow::{
    WorthQueryAdmittedWorkflowEffect, WorthQueryLoweredWorkflowPlan, WorthQueryWorkflowCounters,
};
use crate::runtime::{WorthQueryOrdinaryAuthorityDrift, WorthQueryWorkspace};

use super::{
    WorthQueryWritebackAftermath, WorthQueryWritebackCompletion, WorthQueryWritebackOutcome,
    WorthQueryWritebackRequest, WorthQueryWritebackStop, WorthQueryWritebackStopSource,
};

impl WorthQueryWritebackRequest {
    pub fn run(self, workspace: &mut WorthQueryWorkspace) -> WorthQueryWritebackOutcome {
        let counters = WorthQueryWorkflowCounters::context_checked();
        match workspace.ordinary_authority_drift(&self.context.authority) {
            WorthQueryOrdinaryAuthorityDrift::Handle(denial) => {
                return WorthQueryWritebackOutcome::Stopped(WorthQueryWritebackStop::denied(
                    WorthQueryWritebackStopSource::Handle(denial),
                    denial.to_string(),
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::ForeignOwner => {
                return WorthQueryWritebackOutcome::Stopped(WorthQueryWritebackStop::denied(
                    WorthQueryWritebackStopSource::ForeignAuthority,
                    "writeback context belongs to a different runtime authority",
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::StaleSnapshot => {
                return WorthQueryWritebackOutcome::Stopped(WorthQueryWritebackStop::denied(
                    WorthQueryWritebackStopSource::StaleAuthority,
                    "writeback context is bound to a stale authoritative snapshot",
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::Current => {}
        }
        let (identity, _trigger, inspection_policy) = self.declaration.into_parts();
        let materialize_inspection = inspection_policy.materializes_rich_inspection();
        if materialize_inspection {
            if let Err(error) = workspace.admit_ordinary_rich_inspection() {
                return WorthQueryWritebackOutcome::Stopped(WorthQueryWritebackStop::denied(
                    WorthQueryWritebackStopSource::InspectionUnavailable,
                    error.to_string(),
                    counters,
                ));
            }
        }
        let counters = counters.lower_runtime_attempted();
        let execution = match workspace.execute_ordinary_writeback(
            self.context.authority,
            identity.evidence_identity(),
            materialize_inspection,
        ) {
            Ok(execution) => execution,
            Err(error) => {
                return WorthQueryWritebackOutcome::Stopped(
                    WorthQueryWritebackStop::from_execution(error, counters),
                );
            }
        };
        let admitted_effect = WorthQueryAdmittedWorkflowEffect::from_effect_lifecycle(
            execution.admitted_effect_identity().clone(),
        );
        let lowered_plan =
            WorthQueryLoweredWorkflowPlan::new(execution.lowered_plan_identity().clone());
        let aftermath = WorthQueryWritebackAftermath::new(execution.receipt().target_evidence());
        let (receipt, diagnostics) = execution.into_parts();
        WorthQueryWritebackOutcome::Completed(WorthQueryWritebackCompletion::new(
            admitted_effect,
            lowered_plan,
            receipt,
            aftermath,
            diagnostics,
            counters.execution_completed(materialize_inspection),
        ))
    }
}
