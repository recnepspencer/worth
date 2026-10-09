//! Atomic registration of Query-owned application-attempt records.

use super::WorthQueryApplicationAttemptRegistration;
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptAffinity, WorthQueryApplicationCommitOutcomeIdentity,
    WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::provider::{
    dispatch_outbox::WorthQueryDispatchOutboxBasis, WorthQueryPrimaryGraphApplicationDecisionFact,
    WorthQueryPrimaryGraphProvider,
};

#[path = "provider_registration/consumed_capacity.rs"]
mod consumed_capacity;
#[path = "provider_registration/output_contract.rs"]
mod output_contract;
#[path = "provider_registration/published_causality.rs"]
mod published_causality;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPrimaryGraphApplicationAttempt {
    required_output_demand: Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
    affinity: WorthQueryApplicationAttemptAffinity,
    outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    decision_facts: crate::domain_computation::authorization::WorthQueryProviderDecisionFactBinding,
    effects: super::super::effect_accumulator::WorthQueryRegisteredProviderEffects,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    preimage_demand: Option<worth_query_installation::facade::InstalledPreImageDemand>,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
    dispatch_outbox:
        Option<crate::domain_computation::application_aftermath::WorthQueryPendingDispatchOutbox>,
    conditional_definition:
        Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>,
    indexed_rebase_work_budget: usize,
    live_delivery_reservation: Option<
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePublicationReservation,
    >,
    publication_recovery_reservation: Option<
        crate::domain_computation::primary_graph::provider::WorthQueryApplicationPublicationRecoveryReservation,
    >,
    outstanding_dispatch_reservation: Option<
        crate::domain_computation::primary_graph::provider::OutstandingDispatchReservation,
    >,
    retain_output_demand_observation: bool,
    retain_client_observation: bool,
    producer_required_invariants:
        &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement],
    source_fact_rebase: Option<crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase>,
    consumed_outputs: std::sync::Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>,
}

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn take_required_output_demand(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext> {
        self.required_output_demand.take()
    }

    pub(in crate::domain_computation::primary_graph) const fn affinity(
        &self,
    ) -> &WorthQueryApplicationAttemptAffinity {
        &self.affinity
    }

    pub(in crate::domain_computation::primary_graph) fn facts(
        &self,
    ) -> &std::collections::BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact> {
        self.decision_facts.facts()
    }

    pub(in crate::domain_computation::primary_graph) fn shared_facts(
        &self,
    ) -> std::sync::Arc<
        std::collections::BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact>,
    > {
        self.decision_facts.shared_facts()
    }

    pub(in crate::domain_computation::primary_graph) fn take_source_fact_rebase(
        &mut self,
    ) -> crate::domain_computation::primary_graph::provider::PreparedSourceFactRebase {
        self.source_fact_rebase
            .take()
            .expect("registered pre-effect source rebase is consumed once")
    }
    pub(in crate::domain_computation::primary_graph) fn consumed_outputs(
        &self,
    ) -> &[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]
    {
        &self.consumed_outputs
    }

    pub(in crate::domain_computation::primary_graph) fn retain_consumed_outputs(
        &self,
    ) -> std::sync::Arc<
        [crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
    > {
        std::sync::Arc::clone(&self.consumed_outputs)
    }

    pub(in crate::domain_computation::primary_graph) fn expected_steps(
        &self,
    ) -> &[crate::domain_computation::WorthQueryProvisionalEffectStep] {
        self.effects.expected_steps()
    }

    pub(in crate::domain_computation::primary_graph) const fn expected_step_preparation_work(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryExpectedEffectStepPreparationWork{
        self.effects.expected_step_preparation_work()
    }

    pub(in crate::domain_computation::primary_graph) const fn batch(
        &self,
    ) -> &worth_relational::facade::transactions::WorkerIntentBatch {
        self.effects.batch()
    }

    pub(in crate::domain_computation::primary_graph) const fn idempotency(
        &self,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    pub(in crate::domain_computation::primary_graph) const fn outcome_identity(
        &self,
    ) -> WorthQueryApplicationCommitOutcomeIdentity {
        self.outcome_identity
    }

    pub(in crate::domain_computation::primary_graph) fn emitted_effect_count(&self) -> usize {
        self.effects.emissions().len()
    }

    pub(in crate::domain_computation::primary_graph) fn seal_output_correspondence(
        &self,
        commit: &worth_relational::facade::transactions::CommitResult,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        self.effects.seal_output_correspondence(commit)
    }

    pub(in crate::domain_computation::primary_graph) fn decision_fact_count(&self) -> usize {
        self.decision_facts.decision_fact_count()
    }

    pub(in crate::domain_computation::primary_graph) const fn indexed_rebase_work_budget(
        &self,
    ) -> usize {
        self.indexed_rebase_work_budget
    }

    pub(in crate::domain_computation::primary_graph) const fn producer_required_invariants(
        &self,
    ) -> &'static [crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement]
    {
        self.producer_required_invariants
    }

    pub(in crate::domain_computation::primary_graph) const fn preimage_demand(
        &self,
    ) -> Option<&worth_query_installation::facade::InstalledPreImageDemand> {
        self.preimage_demand.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) const fn dispatch_outbox(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryPendingDispatchOutbox>
    {
        self.dispatch_outbox.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) const fn aftermath_causality(
        &self,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    > {
        self.aftermath_causality.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn take_conditional_definition(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationConditionalDefinition>
    {
        self.conditional_definition.take()
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_successor_observations(
        &mut self,
        provider: &WorthQueryPrimaryGraphProvider,
    ) -> Result<(bool, bool, bool), &'static str> {
        assert!(self.live_delivery_reservation.is_none());
        assert!(self.publication_recovery_reservation.is_none());
        assert!(self.outstanding_dispatch_reservation.is_none());
        let outstanding = provider.reserve_outstanding_dispatch(
            self.dispatch_outbox
                .as_ref()
                .map(|pending| pending.record()),
            self.affinity.product_publication().observation(),
        )?;
        let publication_recovery = provider.reserve_application_publication_recovery(
            self.affinity.product_publication().observation(),
        )?;
        let reservation = provider.reserve_application_commit_causality(
            self.affinity.product_publication().observation(),
            self.effects.emissions().retained_bytes(),
        )?;
        let live = reservation.requires_successor_observation();
        let demand = self.retain_output_demand_observation;
        let client = self.retain_client_observation;
        self.live_delivery_reservation = Some(reservation);
        self.publication_recovery_reservation = Some(publication_recovery);
        self.outstanding_dispatch_reservation = outstanding;
        Ok((live, demand, client))
    }

    pub(in crate::domain_computation::primary_graph) fn take_publication_recovery_reservation(
        &mut self,
    ) -> crate::domain_computation::primary_graph::provider::WorthQueryApplicationPublicationRecoveryReservation
    {
        self.publication_recovery_reservation
            .take()
            .expect("World publication reserved bounded recovery custody before owner effects")
    }

    pub(in crate::domain_computation::primary_graph) fn take_outstanding_dispatch_reservation(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::provider::OutstandingDispatchReservation>
    {
        self.outstanding_dispatch_reservation.take()
    }
}

struct WorthQueryPreparedApplicationAttempt {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    requests: Vec<crate::domain_computation::WorthQueryDecisionFactRequest>,
    dispatch_outbox: Option<WorthQueryDispatchOutboxRecord>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptRegistrationCompletion
{
    requests: Vec<crate::domain_computation::WorthQueryDecisionFactRequest>,
    dispatch_outbox: Option<WorthQueryDispatchOutboxRecord>,
}

impl WorthQueryApplicationAttemptRegistrationCompletion {
    pub(super) fn finish<'run>(
        self,
        seal: super::WorthQueryRegisteredProviderAttemptSeal,
        staged: crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
        steps: std::sync::Arc<[crate::domain_computation::WorthQueryProvisionalEffectStep]>,
    ) -> crate::domain_computation::primary_graph::WorthQueryRegisteredProviderAttempt<'run> {
        crate::domain_computation::primary_graph::WorthQueryRegisteredProviderAttempt::from_registration(
            seal,
            staged,
            self.requests,
            steps,
            self.dispatch_outbox,
        )
    }
}

impl WorthQueryPrimaryGraphProvider {
    /// Binds every Query-owned record that must land in the operation transaction.
    pub(in crate::domain_computation::primary_graph) fn register_application_attempt(
        &self,
        registration: WorthQueryApplicationAttemptRegistration<'_>,
    ) -> Result<WorthQueryApplicationAttemptRegistrationCompletion, &'static str> {
        let reservation = self.reserve_application_attempt(&registration.affinity)?;
        let prepared = self.prepare_application_attempt(registration)?;
        let WorthQueryPreparedApplicationAttempt {
            attempt,
            requests,
            dispatch_outbox,
        } = prepared;
        reservation.complete(attempt)?;
        Ok(WorthQueryApplicationAttemptRegistrationCompletion {
            requests,
            dispatch_outbox,
        })
    }

    fn prepare_application_attempt<'a>(
        &self,
        registration: WorthQueryApplicationAttemptRegistration<'a>,
    ) -> Result<WorthQueryPreparedApplicationAttempt, &'static str> {
        let super::WorthQueryApplicationAttemptRegistration {
            required_output_demand,
            effect_owner: _effect_owner,
            affinity,
            mut decision_facts,
            effects,
            idempotency,
            outcome_identity,
            retained_authorization_fact_count,
            external_effect,
            preimage_demand,
            aftermath_causality,
            conditional_definition,
            indexed_rebase_work_budget,
            retain_output_demand_observation,
            retain_client_observation,
            producer_required_invariants,
            mut consumed_outputs,
            source_fact_rebase,
        } = registration;
        let emitted_effect_count = u64::try_from(effects.emissions().len())
            .map_err(|_| "application emission count exceeds provider representation")?;
        let external_payload = effects.emissions().external_payload(external_effect)?;
        let (effects, dispatch_outbox) = effects.bind_registration_intents(
            self,
            emitted_effect_count,
            aftermath_causality.as_ref(),
            WorthQueryDispatchOutboxBasis {
                external_effect,
                external_payload: external_payload.as_deref(),
                operation_slot: affinity.operation(),
                operation_version: affinity.installed_binding().generation(),
                idempotency,
                outcome_identity,
                branch: affinity.branch(),
            },
        )?;
        decision_facts.validate_session(
            &affinity.graph_work_session(),
            retained_authorization_fact_count,
        )?;
        let requests = decision_facts.take_read_requests();
        let dispatch_outbox_record = dispatch_outbox
            .as_ref()
            .map(|pending| pending.record().clone());
        consumed_capacity::admit_backing(self, &mut consumed_outputs)?;
        Ok(WorthQueryPreparedApplicationAttempt {
            attempt: WorthQueryPrimaryGraphApplicationAttempt {
                required_output_demand,
                affinity,
                outcome_identity,
                decision_facts,
                effects,
                idempotency,
                preimage_demand: preimage_demand.cloned(),
                aftermath_causality,
                dispatch_outbox,
                conditional_definition,
                indexed_rebase_work_budget,
                live_delivery_reservation: None,
                publication_recovery_reservation: None,
                outstanding_dispatch_reservation: None,
                retain_output_demand_observation,
                retain_client_observation,
                producer_required_invariants,
                source_fact_rebase: Some(source_fact_rebase),
                consumed_outputs: std::sync::Arc::from(consumed_outputs),
            },
            requests,
            dispatch_outbox: dispatch_outbox_record,
        })
    }
}
