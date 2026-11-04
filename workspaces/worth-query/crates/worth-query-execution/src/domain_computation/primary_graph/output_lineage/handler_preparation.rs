//! One owner retains a producer handler's completion through publication.
use crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary;
use crate::domain_computation::primary_graph::{ComputationPrior, SealedComputationRetention};

pub(in crate::domain_computation) struct ProducerHandlerPreparation {
    completed_handler_facts: Option<CompletedHandlerFactBoundary>,
    decision_context_use: Option<crate::domain_computation::primary_graph::DecisionContextUse>,
    prepared_decision_reuse: Option<
        crate::domain_computation::primary_graph::output_lineage::PreparedDecisionReuseContext,
    >,
    completed_decision_reuse: Option<
        crate::domain_computation::primary_graph::output_lineage::CompletedDecisionReuseProof,
    >,
    prepared_input_reuse_key:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey>,
    /// What the selected record retained for a partitioned computation the
    /// handler runs, and that record.
    computation_prior: Option<ComputationPrior>,
    /// The run sealed with the handler's facts, for the record publication
    /// writes.
    sealed_computation: HandlerComputationCustody,
}

enum HandlerComputationCustody {
    AwaitingHandler,
    Sealed(SealedComputationRetention),
    Transferred,
}

impl HandlerComputationCustody {
    fn seal(&mut self, computation: SealedComputationRetention) {
        assert!(
            matches!(self, Self::AwaitingHandler),
            "one completed handler retention result"
        );
        *self = Self::Sealed(computation);
    }
    fn take(&mut self) -> SealedComputationRetention {
        assert!(
            matches!(self, Self::Sealed(_)),
            "handler completion precedes the single publication transfer"
        );
        let Self::Sealed(computation) = std::mem::replace(self, Self::Transferred) else {
            unreachable!()
        };
        computation
    }
}

impl ProducerHandlerPreparation {
    pub(in crate::domain_computation) fn new() -> Self {
        Self {
            completed_handler_facts: None,
            decision_context_use: None,
            prepared_decision_reuse: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,
            computation_prior: None,
            sealed_computation: HandlerComputationCustody::AwaitingHandler,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retain_prepared_decision_reuse(
        &mut self,
        context: crate::domain_computation::primary_graph::output_lineage::PreparedDecisionReuseContext,
    ) {
        assert!(self.prepared_decision_reuse.replace(context).is_none());
    }

    pub(in crate::domain_computation::primary_graph) fn take_completed_decision_reuse(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::output_lineage::CompletedDecisionReuseProof>
    {
        self.completed_decision_reuse.take()
    }

    pub(in crate::domain_computation) fn record_decision_context_use(
        &mut self,
        use_mask: crate::domain_computation::primary_graph::DecisionContextUse,
    ) {
        assert!(self.decision_context_use.replace(use_mask).is_none());
    }

    pub(in crate::domain_computation::primary_graph) fn retain_computation_prior(
        &mut self,
        prior: ComputationPrior,
    ) {
        assert!(self.computation_prior.replace(prior).is_none());
    }

    pub(in crate::domain_computation) fn computation_prior(&self) -> Option<ComputationPrior> {
        self.computation_prior.clone()
    }

    /// The sealed run and the record its prior state came from.
    pub(in crate::domain_computation::primary_graph) fn take_sealed_computation(
        &mut self,
    ) -> (
        SealedComputationRetention,
        Option<crate::domain_computation::primary_graph::output_lineage::PriorComputationRecord>,
    ) {
        let sealed = self.sealed_computation.take();
        (
            sealed,
            self.computation_prior
                .take()
                .and_then(ComputationPrior::into_record),
        )
    }

    pub(in crate::domain_computation) fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
        computation: SealedComputationRetention,
    ) {
        assert!(
            self.completed_handler_facts.is_none(),
            "one completed handler read"
        );
        self.sealed_computation.seal(computation);
        self.completed_decision_reuse = self.prepared_decision_reuse.take().and_then(|prepared| {
            boundary.seal_decision_reuse(prepared, self.decision_context_use.take()?)
        });
        self.completed_handler_facts = Some(boundary);
    }

    pub(in crate::domain_computation::primary_graph) fn take_completed_handler_facts(
        &mut self,
    ) -> Option<CompletedHandlerFactBoundary> {
        self.completed_handler_facts.take()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_prepared_input_reuse_key(
        &mut self,
        key: crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey,
    ) {
        assert!(self.prepared_input_reuse_key.is_none());
        self.prepared_input_reuse_key = Some(key);
    }

    pub(in crate::domain_computation::primary_graph) fn take_prepared_input_reuse_key(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey>
    {
        self.prepared_input_reuse_key.take()
    }
}
