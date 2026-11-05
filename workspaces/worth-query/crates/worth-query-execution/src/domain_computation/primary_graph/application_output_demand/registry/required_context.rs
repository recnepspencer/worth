use std::sync::Arc;

use super::ready_backing::PreparedReadyBacking;
use super::WorthQueryOutputDemandInterest;
use super::WorthQueryOutputDemandKey;
use super::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation;
use crate::domain_computation::primary_graph::ComputationPrior;
use crate::domain_computation::primary_graph::SealedComputationRetention;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

/// The live demand record selected for a producer operation. This is a
/// move-only carrier; the source epoch and installed producer are the registry
/// key, while the registry remains the sole owner of prerequisite claims.
pub struct RequiredOutputDemandContext {
    family: &'static str,
    owner: WorthQueryOutputDemandRegistry,
    key: Arc<WorthQueryOutputDemandKey>,
    work_membership: Option<Arc<super::required_work::RequiredWorkMembership>>,

    ready_backing: Option<PreparedReadyBacking>,
    handler: ProducerHandlerPreparation,
    reuses_live_output_only: bool,
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
        self.handler.retain_prepared_decision_reuse(context)
    }

    pub(in crate::domain_computation::primary_graph) fn take_completed_decision_reuse(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::output_lineage::CompletedDecisionReuseProof>
    {
        self.handler.take_completed_decision_reuse()
    }

    pub(in crate::domain_computation) fn record_decision_context_use(
        &mut self,
        use_mask: crate::domain_computation::primary_graph::DecisionContextUse,
    ) {
        self.handler.record_decision_context_use(use_mask)
    }

    pub(in crate::domain_computation::primary_graph) fn retain_computation_prior(
        &mut self,
        prior: ComputationPrior,
    ) {
        self.handler.retain_computation_prior(prior)
    }

    pub(in crate::domain_computation) fn computation_prior(&self) -> Option<ComputationPrior> {
        self.handler.computation_prior()
    }

    pub(in crate::domain_computation::primary_graph) fn take_sealed_computation(
        &mut self,
    ) -> (
        SealedComputationRetention,
        Option<crate::domain_computation::primary_graph::output_lineage::PriorComputationRecord>,
    ) {
        self.handler.take_sealed_computation()
    }

    pub(in crate::domain_computation) fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
        computation: SealedComputationRetention,
    ) {
        self.handler
            .record_completed_handler_facts(boundary, computation)
    }

    pub(in crate::domain_computation::primary_graph) fn take_completed_handler_facts(
        &mut self,
    ) -> Option<CompletedHandlerFactBoundary> {
        self.handler.take_completed_handler_facts()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_prepared_input_reuse_key(
        &mut self,
        key: crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey,
    ) {
        self.handler.retain_prepared_input_reuse_key(key)
    }

    pub(in crate::domain_computation::primary_graph) fn take_prepared_input_reuse_key(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey>
    {
        self.handler.take_prepared_input_reuse_key()
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
            handler: ProducerHandlerPreparation::new(),
            reuses_live_output_only: false,
        })
    }
}

mod court_doors_for_test;
