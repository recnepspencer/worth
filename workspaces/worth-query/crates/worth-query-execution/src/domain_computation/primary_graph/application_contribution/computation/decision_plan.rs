//! Turn a prepared managed computation into a sealed request and owner completion.
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
    pub fn into_decision_plan<Value: 'static, Denial: 'static>(
        self,
        complete: impl FnOnce(
                Result<
                    WorthQueryCompletedManagedComputation<Schema, Feature, Computation, Owner>,
                    WorthQueryManagedComputationDenial<Owner::Stopped>,
                >,
            ) -> HandlerResult<Value, Denial>
            + 'static,
    ) -> WorthQueryDecisionPlan<Value, Denial> {
        WorthQueryDecisionPlan::prepared(
            PreparedCompute::new(move |member| self.compute(member.managed())),
            complete,
        )
    }
}
