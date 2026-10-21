use std::sync::Arc;

use super::ready_backing::PreparedReadyBacking;
use super::WorthQueryOutputDemandInterest;
use super::WorthQueryOutputDemandKey;
use super::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::ComputationPrior;
use crate::domain_computation::primary_graph::SealedComputationRetention;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

/// The live demand record selected for a producer operation. This is a
/// move-only carrier; the source epoch and installed producer are the registry
/// key, while the registry remains the sole owner of prerequisite claims.
pub(in crate::domain_computation) struct RequiredOutputDemandContext {
    family: &'static str,
    owner: WorthQueryOutputDemandRegistry,
    key: Arc<WorthQueryOutputDemandKey>,
    work_membership: Option<Arc<super::required_work::RequiredWorkMembership>>,

    ready_backing: Option<PreparedReadyBacking>,
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
    reuses_live_output_only: bool,
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

/// Both parts are issued together before an installed producer can execute.
pub(in crate::domain_computation) struct RequiredOutputExecution {
    context: RequiredOutputDemandContext,
    ready_backing: PreparedReadyBacking,
}

impl RequiredOutputExecution {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (RequiredOutputDemandContext, PreparedReadyBacking) {
        (self.context, self.ready_backing)
    }
}

impl RequiredOutputDemandContext {
    pub(in crate::domain_computation::primary_graph) fn publication_family(&self) -> &'static str {
        self.family
    }

    /// The selection this context executes reuses the live output it was
    /// selected for, or stops before any effect.
    pub(in crate::domain_computation::primary_graph) fn reuse_live_output_only(&mut self) {
        self.reuses_live_output_only = true;
    }

    pub(in crate::domain_computation::primary_graph) fn reuses_live_output_only(&self) -> bool {
        self.reuses_live_output_only
    }

    /// Fills the slot whose bytes the context reserved when it was issued.
    pub(in crate::domain_computation::primary_graph) fn work_membership(
        &self,
    ) -> Option<Arc<super::required_work::RequiredWorkMembership>> {
        self.work_membership.clone()
    }
    pub(in crate::domain_computation::primary_graph) fn prepare_execution(
        mut self,
        mode: crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority,
    ) -> RequiredOutputExecution {
        RequiredOutputExecution {
            ready_backing: self
                .ready_backing
                .take()
                .expect("required context owns pre-effect Ready storage")
                .bind_execution_mode(mode),
            context: self,
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

    pub(super) fn key_arc(&self) -> &Arc<WorthQueryOutputDemandKey> {
        &self.key
    }

    pub(in crate::domain_computation::primary_graph) fn registry(
        &self,
    ) -> &WorthQueryOutputDemandRegistry {
        &self.owner
    }

    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &WorthQueryOutputDemandKey {
        self.key.as_ref()
    }
}

impl WorthQueryOutputDemandInterest {
    pub(in crate::domain_computation::primary_graph) fn required_context(
        &self,
        family: &'static str,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<RequiredOutputDemandContext, WorthQueryOutputDemandDenial> {
        self.owner.drain_terminal_cleanup_admitted(source_owner)?;
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.drain_cancelled_settlement_vacancies();
        // Selecting this demand's own record is registry navigation, charged
        // apart from the work the producer declares for its operation.

        let Some(record) = state.records.get(&self.key) else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "required output demand no longer has a live registry record",
            ));
        };
        let work_membership = record.work_membership.clone();
        let ready_backing = PreparedReadyBacking::new();
        let key = Arc::new(self.key.clone());

        Ok(RequiredOutputDemandContext {
            family,
            owner: self.owner.clone(),
            key,
            work_membership,

            ready_backing: Some(ready_backing),
            completed_handler_facts: None,
            decision_context_use: None,
            prepared_decision_reuse: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,

            reuses_live_output_only: false,
            computation_prior: None,
            sealed_computation: HandlerComputationCustody::AwaitingHandler,
        })
    }
}
