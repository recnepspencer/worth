use super::{
    WorthQueryPreviewJourneyOutcome, WorthQueryPromotionEligiblePreviewRequest,
    WorthQueryReadOnlyPreviewCompletion, WorthQueryReadOnlyPreviewRequest,
};
use crate::ordinary::workflow::{
    WorthQueryLoweredWorkflowPlan, WorthQueryWorkflowAftermath, WorthQueryWorkflowCounters,
    WorthQueryWorkflowOutcome, WorthQueryWorkflowRequest, WorthQueryWorkflowStop,
    WorthQueryWorkflowStopSource,
};
use crate::runtime::{WorthQueryOrdinaryAuthorityDrift, WorthQueryWorkspace};

impl WorthQueryReadOnlyPreviewRequest {
    pub fn open_and_close(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryPreviewJourneyOutcome {
        let counters = WorthQueryWorkflowCounters::context_checked();
        if self.context.authority.session_label() != Some(&self.declaration.label) {
            return WorthQueryPreviewJourneyOutcome::Stopped(WorthQueryWorkflowStop::denied(
                WorthQueryWorkflowStopSource::CrossSession,
                counters,
            ));
        }
        match workspace.ordinary_authority_drift(&self.context.authority) {
            WorthQueryOrdinaryAuthorityDrift::Handle(denial) => {
                return WorthQueryPreviewJourneyOutcome::Stopped(WorthQueryWorkflowStop::runtime(
                    denial.into(),
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::ForeignOwner => {
                return WorthQueryPreviewJourneyOutcome::Stopped(WorthQueryWorkflowStop::denied(
                    WorthQueryWorkflowStopSource::ForeignAuthority,
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::StaleSnapshot => {
                return WorthQueryPreviewJourneyOutcome::Stopped(WorthQueryWorkflowStop::denied(
                    WorthQueryWorkflowStopSource::StalePreview,
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::Current => {}
        }
        debug_assert_eq!(
            self.context.authority.family(),
            crate::runtime::WorthQueryOrdinaryAuthorityFamily::ReadOnlyPreview
        );
        let materialize_inspection = self
            .declaration
            .inspection_policy
            .materializes_rich_inspection();
        if materialize_inspection {
            if let Err(error) = workspace.admit_ordinary_rich_inspection() {
                return WorthQueryPreviewJourneyOutcome::Stopped(
                    WorthQueryWorkflowStop::inspection_unavailable(error, counters),
                );
            }
        }
        let basis_admission = self
            .context
            .authority
            .into_preview_basis()
            .expect("read-only context must carry its admitted preview basis");
        let counters = counters.execution_attempted();
        let execution = match workspace.execute_ordinary_read_only_preview(
            basis_admission,
            &self.declaration.identity,
            materialize_inspection,
        ) {
            Ok(execution) => execution,
            Err(error) => {
                return WorthQueryPreviewJourneyOutcome::Stopped(WorthQueryWorkflowStop::runtime(
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
        WorthQueryPreviewJourneyOutcome::ReadOnlyCompleted(
            WorthQueryReadOnlyPreviewCompletion::new(
                lowered_plan,
                aftermath,
                execution.into_outcome(),
                counters.execution_completed(materialize_inspection),
            ),
        )
    }
}

impl WorthQueryPromotionEligiblePreviewRequest {
    pub fn open_and_close(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryPreviewJourneyOutcome {
        let request = WorthQueryWorkflowRequest {
            declaration: self.declaration.workflow,
            context: crate::ordinary::workflow::WorthQueryWorkflowContext {
                authority: self.context.authority,
            },
        };
        match request.run(workspace) {
            WorthQueryWorkflowOutcome::Completed(completion) => {
                WorthQueryPreviewJourneyOutcome::PromotionCompleted(completion)
            }
            WorthQueryWorkflowOutcome::Stopped(stop) => {
                WorthQueryPreviewJourneyOutcome::Stopped(stop)
            }
        }
    }
}
