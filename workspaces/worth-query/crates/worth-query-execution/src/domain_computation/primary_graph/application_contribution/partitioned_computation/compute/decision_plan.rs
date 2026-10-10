//! Prepare on the owner, compute through its sealed request, complete on the owner.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::computation::prepared_request::PreparedCompute,
    application_entry::mutation::{HandlerResult, WorthQueryDecisionPlan},
};

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// The callback runs on the owner and may catch a computation denial.
    pub fn into_decision_plan<Value: 'static, Domain: 'static>(
        self,
        complete: impl FnOnce(
                Result<
                    WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>,
                    Denial<Schema, Feature, Computation, Owner>,
                >,
            ) -> HandlerResult<Value, Domain>
            + 'static,
    ) -> WorthQueryDecisionPlan<Value, Domain> {
        WorthQueryDecisionPlan::prepared(
            PreparedCompute::new(move |member| self.compute(member.managed())),
            complete,
        )
    }
}
