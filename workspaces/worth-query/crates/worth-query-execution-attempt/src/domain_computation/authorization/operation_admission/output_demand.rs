use super::WorthQueryAdmittedOperation;
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary, ComputationPrior,
    RequiredOutputDemandContext, SealedComputationRetention,
};
use crate::domain_computation::provider_session::ManagedGraphWorkSessionPrimaryExt;

impl WorthQueryAdmittedOperation {
    pub fn completed_input_prefix(&self) -> Option<usize> {
        if let Some(context) = &self.data.required_output_demand {
            context.completed_input_prefix()
        } else {
            self.data
                .graph_work
                .coupled_handler_typed()
                .and_then(|handler| handler.completed_input_prefix())
        }
    }

    pub fn insert_input_facts(&mut self, expected: usize, count: usize) -> bool {
        if let Some(context) = &mut self.data.required_output_demand {
            context.insert_input_facts(expected, count)
        } else {
            self.data
                .graph_work
                .coupled_handler_typed_mut()
                .is_some_and(|handler| handler.insert_input_facts(expected, count))
        }
    }

    pub fn record_decision_context_use(
        &mut self,
        use_mask: crate::domain_computation::primary_graph::DecisionContextUse,
    ) {
        if let Some(context) = &mut self.data.required_output_demand {
            context.record_decision_context_use(use_mask);
        } else if let Some(handler) = self.data.graph_work.coupled_handler_typed_mut() {
            handler.record_decision_context_use(use_mask);
        }
    }

    /// Framework execution custody follows the admitted operation through its
    /// handler and sealed read set. It grants no operation authority.
    pub fn bind_required_output_demand(&mut self, context: RequiredOutputDemandContext) {
        assert!(
            self.data.required_output_demand.is_none(),
            "one operation carries one output demand"
        );
        self.data.required_output_demand = Some(context);
    }

    pub fn take_required_output_demand(&mut self) -> Option<RequiredOutputDemandContext> {
        self.data.required_output_demand.take()
    }

    /// Seal's facts, and the partitioned computation run they sealed. A run
    /// is retained only for a producer's output demand.
    pub fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
        computation: SealedComputationRetention,
    ) {
        if let Some(context) = &mut self.data.required_output_demand {
            context.record_completed_handler_facts(boundary, computation);
        } else if let Some(handler) = self.data.graph_work.coupled_handler_typed_mut() {
            handler.record_completed_handler_facts(boundary, computation);
        } else {
            crate::domain_computation::primary_graph::observe_discarded(&computation);
            crate::domain_computation::primary_graph::SealedComputationRun::keep_in_test(
                match computation {
                    SealedComputationRetention::Produced(run) => Some(run),
                    SealedComputationRetention::Absent(_) => None,
                },
            );
        }
    }

    /// What the producer that runs this operation retained for the
    /// partitioned computation its handler runs.
    pub fn computation_prior(&self) -> Option<ComputationPrior> {
        let prior = self
            .data
            .required_output_demand
            .as_ref()
            .and_then(RequiredOutputDemandContext::computation_prior)
            .or_else(|| {
                self.data
                    .graph_work
                    .coupled_handler_typed()
                    .and_then(|handler| handler.computation_prior())
            });
        prior.or_else(ComputationPrior::handed_in_test)
    }
}
