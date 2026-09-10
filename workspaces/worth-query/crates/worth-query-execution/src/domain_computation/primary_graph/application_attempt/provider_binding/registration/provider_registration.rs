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

pub(in crate::domain_computation::primary_graph) struct WorthQueryPrimaryGraphApplicationAttempt {
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
    live_delivery_reservation: Option<
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePublicationReservation,
    >,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryPublishedApplicationCausality {
    outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    emitted_effect_count: usize,
}

impl WorthQueryPublishedApplicationCausality {
    pub(in crate::domain_computation::primary_graph) const fn outcome_identity(
        &self,
    ) -> WorthQueryApplicationCommitOutcomeIdentity {
        self.outcome_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }
}

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) const fn affinity(
        &self,
    ) -> &WorthQueryApplicationAttemptAffinity {
        &self.affinity
    }

    pub(in crate::domain_computation::primary_graph) const fn facts(
        &self,
    ) -> &std::collections::BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact> {
        self.decision_facts.facts()
    }

    pub(in crate::domain_computation::primary_graph) fn expected_steps(
        &self,
    ) -> Vec<crate::domain_computation::WorthQueryProvisionalEffectStep> {
        self.effects.expected_steps()
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

    pub(in crate::domain_computation::primary_graph) fn decision_fact_count(&self) -> usize {
        self.decision_facts.decision_fact_count()
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

    pub(in crate::domain_computation::primary_graph) fn reserve_live_delivery(
        &mut self,
        provider: &WorthQueryPrimaryGraphProvider,
    ) -> Result<bool, &'static str> {
        assert!(self.live_delivery_reservation.is_none());
        let reservation = provider.reserve_application_commit_causality(
            self.affinity.product_publication().observation(),
            self.effects.emissions().retained_bytes(),
        )?;
        let requires_observation = reservation.requires_successor_observation();
        self.live_delivery_reservation = Some(reservation);
        Ok(requires_observation)
    }

    pub(in crate::domain_computation::primary_graph) fn publish_causality(
        self,
        provider: &WorthQueryPrimaryGraphProvider,
        publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> WorthQueryPublishedApplicationCausality {
        let emitted_effect_count = provider.publish_application_commit_causality(
            self.live_delivery_reservation
                .expect("World publication reserved live causality before owner effects"),
            publication,
            self.effects.into_emissions(),
        );
        WorthQueryPublishedApplicationCausality {
            outcome_identity: self.outcome_identity,
            emitted_effect_count,
        }
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
        steps: Vec<crate::domain_computation::WorthQueryProvisionalEffectStep>,
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
            effect_owner: _effect_owner,
            affinity,
            mut decision_facts,
            effects,
            idempotency,
            retained_authorization_fact_count,
            external_effect,
            preimage_demand,
            aftermath_causality,
            conditional_definition,
        } = registration;
        let emitted_effect_count = u64::try_from(effects.emissions().len())
            .map_err(|_| "application emission count exceeds provider representation")?;
        let outcome_identity = WorthQueryApplicationCommitOutcomeIdentity::mint()
            .ok_or("application outcome identity space is exhausted")?;
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
        Ok(WorthQueryPreparedApplicationAttempt {
            attempt: WorthQueryPrimaryGraphApplicationAttempt {
                affinity,
                outcome_identity,
                decision_facts,
                effects,
                idempotency,
                preimage_demand: preimage_demand.cloned(),
                aftermath_causality,
                dispatch_outbox,
                conditional_definition,
                live_delivery_reservation: None,
            },
            requests,
            dispatch_outbox: dispatch_outbox_record,
        })
    }
}
