use worth_execution::ChargedBytes;
// Turn a prepared managed computation into a sealed request and owner completion.
use super::prepared_request::PreparedCompute;
use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerResult, WorthQueryDecisionPlan,
};

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    /// Move only prepared compute into the request; the callback receives its
    /// result on the owner and may complete it or handle a computation denial.
    /// `maximum_result_heap_bytes` bounds every allocation of the returned
    /// prepared value, computed value, or denial beyond their inline storage.
    /// This declaration is separate from the prepare-time retained-byte limit.
    pub fn into_decision_plan<Value: 'static, Denial: 'static>(
        self,
        maximum_result_heap_bytes: u64,
        complete: impl FnOnce(
                Result<
                    WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>,
                    WorthQueryManagedComputationDenial<Owner::Stopped>,
                >,
            ) -> HandlerResult<Value, Denial>
            + 'static,
    ) -> WorthQueryDecisionPlan<Value, Denial>
    where
        Owner::Computed: worth_execution::ChargedBytes,
        Owner::Stopped: worth_execution::ChargedBytes,
    {
        WorthQueryDecisionPlan::prepared(
            PreparedCompute::new(
                self.prepared.retained_bytes() as u64,
                maximum_result_heap_bytes,
                move |member| self.compute_for_member(member),
                |result| match result {
                    Ok(completed) => (completed.prepared.retained_bytes() as u64)
                        .saturating_add(completed.computed.additional_charged_bytes()),
                    Err(WorthQueryManagedComputationDenial::Owner(stopped)) => {
                        stopped.additional_charged_bytes()
                    }
                    Err(
                        WorthQueryManagedComputationDenial::Resource(_)
                        | WorthQueryManagedComputationDenial::Interrupted(_)
                        | WorthQueryManagedComputationDenial::NestedPatternStopped,
                    ) => 0,
                },
                Result::is_err,
            ),
            complete,
        )
    }
}
