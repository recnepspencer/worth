use super::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary, ComputationPrior,
    RequiredOutputDemandContext, SealedComputationRun,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation) fn record_decision_context_use(
        &mut self,
        use_mask: crate::domain_computation::primary_graph::DecisionContextUse,
    ) {
        if let Some(context) = &mut self.required_output_demand {
            context.record_decision_context_use(use_mask);
        }
    }

    /// Framework execution custody follows the admitted operation through its
    /// handler and sealed read set. It grants no operation authority.
    pub(in crate::domain_computation) fn bind_required_output_demand(
        &mut self,
        context: RequiredOutputDemandContext,
    ) {
        assert!(
            self.required_output_demand.is_none(),
            "one operation carries one output demand"
        );
        self.required_output_demand = Some(context);
    }

    pub(in crate::domain_computation) fn take_required_output_demand(
        &mut self,
    ) -> Option<RequiredOutputDemandContext> {
        self.required_output_demand.take()
    }

    /// Seal's facts, and the partitioned computation run they sealed. A run
    /// is retained only for a producer's output demand.
    pub(in crate::domain_computation) fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
        computation: Option<SealedComputationRun>,
    ) {
        if let Some(context) = &mut self.required_output_demand {
            context.record_completed_handler_facts(boundary, computation);
        } else {
            #[cfg(test)]
            SealedComputationRun::keep_in_test(computation);
        }
    }

    /// What the producer that runs this operation retained for the
    /// partitioned computation its handler runs.
    pub(in crate::domain_computation) fn computation_prior(&self) -> Option<ComputationPrior> {
        let prior = self
            .required_output_demand
            .as_ref()
            .and_then(RequiredOutputDemandContext::computation_prior);
        #[cfg(test)]
        let prior = prior.or_else(ComputationPrior::handed_in_test);
        prior
    }
}
