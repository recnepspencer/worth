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
            PreparedCompute::new(move |member| {
                member.compute(|execution, _poll, transport| {
                    self.compute_with_transport(execution, transport)
                        .map_err(MemberComputeFailure::into_failure)
                })
            }),
            complete,
        )
    }
}

impl<Schema, Feature, Computation, Owner>
    WorthQueryInstalledPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// Gather through the original decision reader and complete on its owner.
    pub fn decision_plan<Binding, Value: 'static, Domain: 'static>(
        &self,
        reader: &mut crate::domain_computation::primary_graph::DecisionReader<'_, '_, '_, Schema, Binding>,
        input: &super::super::InputValue<Schema, Feature, Computation>,
        complete: impl FnOnce(Owner::Output) -> HandlerResult<Value, Domain> + 'static,
    ) -> WorthQueryDecisionPlan<Value, Domain>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema, Operation = Owner::Operation>,
        Owner::Stopped: Into<crate::domain_computation::primary_graph::application_entry::mutation::HandlerFailure<Domain>> + std::fmt::Debug + Sync,
    {
        match self.prepare(reader, input) {
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
