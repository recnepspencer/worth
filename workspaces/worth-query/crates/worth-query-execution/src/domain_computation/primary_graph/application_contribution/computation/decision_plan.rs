//! Installed preparation, request-owned compute and validated owner completion.
use super::prepared_request::PreparedCompute;
use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    HandlerFailure, HandlerResult, WorthQueryDecisionPlan,
};

impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    /// The success callback receives only the owner's validated completion.
    /// Computation failures retain their domain, execution or interruption kind.
    pub fn decision_plan<Value: 'static, Domain: 'static>(
        &self,
        input: &<Computation::Input as ApplicationComputationInput>::Value,
        complete: impl FnOnce(Owner::Output) -> HandlerResult<Value, Domain> + 'static,
    ) -> WorthQueryDecisionPlan<Value, Domain>
    where
        Owner::Stopped: Into<HandlerFailure<Domain>>,
    {
        match self.prepare(input) {
            Ok(prepared) => prepared.into_decision_plan(move |computed| {
                match computed.and_then(|computed| computed.complete()) {
                    Ok(value) => complete(value),
                    Err(cause) => HandlerResult::from_result(Err(cause.into())),
                }
            }),
            Err(cause) => {
                WorthQueryDecisionPlan::immediate(HandlerResult::from_result(Err(cause.into())))
            }
        }
    }
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryPreparedManagedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
{
    /// The lower-level owner callback may deliberately reinterpret a domain
    /// refusal. Native prepared values and results require no byte forecast.
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
            PreparedCompute::new(move |member| self.compute_for_member(member)),
            complete,
        )
    }
}
