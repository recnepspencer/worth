use super::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary, RequiredOutputDemandContext,
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

    pub(in crate::domain_computation) fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
    ) {
        if let Some(context) = &mut self.required_output_demand {
            context.record_completed_handler_facts(boundary);
        }
    }
}
