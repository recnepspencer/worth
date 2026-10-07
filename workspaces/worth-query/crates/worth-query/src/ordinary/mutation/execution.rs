use super::{
    WorthQueryLoweredMutationPlan, WorthQueryMutationAftermath, WorthQueryMutationCompletion,
    WorthQueryMutationCounters, WorthQueryMutationOutcome, WorthQueryMutationRequest,
    WorthQueryMutationStop, WorthQueryMutationStopSource,
};
use crate::runtime::{
    WorthQueryOrdinaryAuthorityDrift, WorthQueryOrdinaryAuthorityFamily, WorthQueryWorkspace,
};

impl WorthQueryMutationRequest {
    pub fn run(self, workspace: &mut WorthQueryWorkspace) -> WorthQueryMutationOutcome {
        let counters = WorthQueryMutationCounters::context_checked();
        debug_assert_eq!(
            self.context.authority.family(),
            WorthQueryOrdinaryAuthorityFamily::Mutation
        );
        match workspace.ordinary_authority_drift(&self.context.authority) {
            WorthQueryOrdinaryAuthorityDrift::Handle(denial) => {
                return WorthQueryMutationOutcome::Stopped(WorthQueryMutationStop::runtime(
                    denial.into(),
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::ForeignOwner => {
                return WorthQueryMutationOutcome::Stopped(WorthQueryMutationStop::authority(
                    WorthQueryMutationStopSource::ForeignAuthority,
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::StaleSnapshot => {
                return WorthQueryMutationOutcome::Stopped(WorthQueryMutationStop::authority(
                    WorthQueryMutationStopSource::StaleBasis,
                    counters,
                ));
            }
            WorthQueryOrdinaryAuthorityDrift::Current => {}
        }

        let (command, inspection_policy) = self.declaration.into_parts();
        let materialize_inspection = inspection_policy.materializes_rich_inspection();
        if materialize_inspection {
            if let Err(error) = workspace.admit_ordinary_rich_inspection() {
                return WorthQueryMutationOutcome::Stopped(
                    WorthQueryMutationStop::inspection_unavailable(error, counters),
                );
            }
        }
        let execution = match workspace
            .execute_ordinary_authoritative_mutation(command, materialize_inspection)
        {
            Ok(execution) => execution,
            Err(error) => {
                let counters = if matches!(
                    error,
                    crate::runtime::WorthQueryRuntimeError::MutationContractDenied(_)
                ) {
                    counters
                } else {
                    counters.execution_attempted()
                };
                return WorthQueryMutationOutcome::Stopped(WorthQueryMutationStop::runtime(
                    error, counters,
                ));
            }
        };
        let counters = counters.execution_attempted();
        let plan = WorthQueryLoweredMutationPlan::new(
            execution.request_identity().clone(),
            execution.handoff_identity().clone(),
        );
        let receipt_identity = execution.receipt_identity().clone();
        let inspection_identity = execution.inspection_identity().cloned();
        let receipt = execution.into_receipt();
        let aftermath =
            WorthQueryMutationAftermath::new(&receipt, receipt_identity, inspection_identity);
        WorthQueryMutationOutcome::Completed(WorthQueryMutationCompletion::new(
            plan,
            receipt,
            aftermath,
            counters.execution_completed(materialize_inspection),
        ))
    }
}
