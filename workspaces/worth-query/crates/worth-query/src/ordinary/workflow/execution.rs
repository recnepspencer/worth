use super::{
    WorthQueryAdmittedWorkflowEffect, WorthQueryLoweredWorkflowPlan,
    WorthQueryPromotionEligibility, WorthQueryWorkflowAftermath, WorthQueryWorkflowCompletion,
    WorthQueryWorkflowCounters, WorthQueryWorkflowFamily, WorthQueryWorkflowOutcome,
    WorthQueryWorkflowRequest, WorthQueryWorkflowStop, WorthQueryWorkflowStopSource,
};
use crate::runtime::{WorthQueryOrdinaryAuthorityDrift, WorthQueryWorkspace};

impl WorthQueryWorkflowRequest {
    pub fn run(self, workspace: &mut WorthQueryWorkspace) -> WorthQueryWorkflowOutcome {
        let counters = WorthQueryWorkflowCounters::context_checked();
        let (identity, family, label, mutation, inspection_policy) = self.declaration.into_parts();
        if self.context.authority.session_label() != Some(&label) {
            return WorthQueryWorkflowOutcome::Stopped(WorthQueryWorkflowStop::denied(
                WorthQueryWorkflowStopSource::CrossSession,
                counters,
            ));
        }
        debug_assert_eq!(family, WorthQueryWorkflowFamily::PreviewPromotion);
        match workspace.ordinary_authority_drift(&self.context.authority) {
            WorthQueryOrdinaryAuthorityDrift::Handle(denial) => {
                return WorthQueryWorkflowOutcome::Stopped(WorthQueryWorkflowStop::runtime(
                    denial.into(),
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::ForeignOwner => {
                return WorthQueryWorkflowOutcome::Stopped(WorthQueryWorkflowStop::denied(
                    WorthQueryWorkflowStopSource::ForeignAuthority,
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::StaleSnapshot => {
                return WorthQueryWorkflowOutcome::Stopped(WorthQueryWorkflowStop::denied(
                    WorthQueryWorkflowStopSource::StalePreview,
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::Current => {}
        }
        debug_assert_eq!(
            self.context.authority.family(),
            crate::runtime::WorthQueryOrdinaryAuthorityFamily::PromotionPreview
        );
        debug_assert!(self.context.authority.preview_basis().is_some());
        let eligibility = WorthQueryPromotionEligibility::from_authority(&self.context.authority);
        let admitted_effect = WorthQueryAdmittedWorkflowEffect::new(
            identity.evidence_identity(),
            &self.context.authority,
        );
        let basis_admission = self
            .context
            .authority
            .into_preview_basis()
            .expect("promotion context must carry its admitted preview basis");
        let (command, _) = mutation.into_parts();
        let materialize_inspection = inspection_policy.materializes_rich_inspection();
        if materialize_inspection {
            if let Err(error) = workspace.admit_ordinary_rich_inspection() {
                return WorthQueryWorkflowOutcome::Stopped(
                    WorthQueryWorkflowStop::inspection_unavailable(error, counters),
                );
            }
        }
        let counters = counters.execution_attempted();
        let execution = match workspace.execute_ordinary_preview_promotion(
            basis_admission,
            identity.evidence_identity(),
            command,
            materialize_inspection,
        ) {
            Ok(execution) => execution,
            Err(error) => {
                return WorthQueryWorkflowOutcome::Stopped(WorthQueryWorkflowStop::runtime(
                    error, counters,
                ));
            }
        };
        let lowered_plan = WorthQueryLoweredWorkflowPlan::new(execution.request_identity().clone());
        let aftermath = WorthQueryWorkflowAftermath::new(
            execution.outcome(),
            execution.receipt_identity().clone(),
            execution.aftermath_identity().clone(),
            execution.inspection_identity().cloned(),
        );
        WorthQueryWorkflowOutcome::Completed(WorthQueryWorkflowCompletion::new(
            eligibility,
            admitted_effect,
            lowered_plan,
            aftermath,
            execution.into_outcome(),
            counters.execution_completed(materialize_inspection),
        ))
    }
}
