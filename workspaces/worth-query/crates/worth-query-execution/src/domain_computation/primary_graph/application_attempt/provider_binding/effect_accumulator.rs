use std::collections::BTreeMap;
use std::sync::Arc;

use worth_relational::facade::transactions::{EntityReference, MutationIntent, WorkerIntentBatch};

use super::effect_lowering::{
    created_entity_symbols, lower_provider_effect, WorthQueryLoweredProviderEffect,
};
use super::{
    retained_bytes_denial, WorthQueryAdmittedApplicationEmissionBatch,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEmission;
use crate::domain_computation::WorthQueryProvisionalEffectStep;

pub(super) struct WorthQueryProviderEffectAccumulator<'facts> {
    facts: &'facts [WorthQueryApplicationObservedFact],
    symbols: BTreeMap<EntityReference, Arc<str>>,
    lowered: Vec<WorthQueryLoweredProviderEffect>,
}

pub(super) struct WorthQueryRegisteredProviderEffects {
    lowered: Vec<WorthQueryLoweredProviderEffect>,
    batch: WorkerIntentBatch,
    emissions: WorthQueryAdmittedApplicationEmissionBatch,
    output_correspondence: super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
}

impl<'facts> WorthQueryProviderEffectAccumulator<'facts> {
    pub(super) fn new(
        facts: &'facts [WorthQueryApplicationObservedFact],
        effects: &[WorthQueryApplicationRealizedEffect],
    ) -> Self {
        Self {
            facts,
            symbols: created_entity_symbols(effects),
            lowered: Vec::with_capacity(effects.len()),
        }
    }

    pub(super) fn add_effect(
        &mut self,
        effect: WorthQueryApplicationRealizedEffect,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let lowered = lower_provider_effect(self.facts, &self.symbols, effect)?;
        self.lowered.push(lowered);
        Ok(())
    }

    pub(super) fn finish(
        self,
        expected_emission_retained_bytes: u64,
        emission_retained_bytes_ceiling: u64,
        output_correspondence: super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    ) -> Result<WorthQueryRegisteredProviderEffects, WorthQueryApplicationAttemptDenial> {
        let (intents, emissions) = materialize_commit_projections(&self.lowered);
        let batch = intents.into_iter().fold(
            WorkerIntentBatch::new("application-provider-attempt"),
            WorkerIntentBatch::push,
        );
        let emissions = WorthQueryAdmittedApplicationEmissionBatch::admit(
            emissions,
            emission_retained_bytes_ceiling,
        )
        .map_err(|_| retained_bytes_denial())?;
        if emissions.retained_bytes() != expected_emission_retained_bytes {
            return Err(retained_bytes_denial());
        }
        Ok(WorthQueryRegisteredProviderEffects {
            lowered: self.lowered,
            batch,
            emissions,
            output_correspondence,
        })
    }
}

impl WorthQueryRegisteredProviderEffects {
    pub(super) fn expected_steps(&self) -> Vec<WorthQueryProvisionalEffectStep> {
        let mut unique = Vec::new();
        for step in self
            .lowered
            .iter()
            .filter_map(|effect| match effect {
                WorthQueryLoweredProviderEffect::Mutation { steps, .. } => Some(steps.as_slice()),
                WorthQueryLoweredProviderEffect::Emission(_) => None,
            })
            .flatten()
        {
            if !unique.contains(step) {
                unique.push(step.clone());
            }
        }
        unique
    }

    pub(super) const fn batch(&self) -> &WorkerIntentBatch {
        &self.batch
    }

    pub(super) const fn emissions(&self) -> &WorthQueryAdmittedApplicationEmissionBatch {
        &self.emissions
    }

    /// Appends the Primary Graph registration records without releasing or
    /// replacing the sealed provider-effect batch.
    pub(super) fn bind_registration_intents(
        self,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        emitted_effect_count: u64,
        aftermath_causality: Option<
            &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
        dispatch_basis: crate::domain_computation::primary_graph::provider::dispatch_outbox::WorthQueryDispatchOutboxBasis<'_>,
    ) -> Result<
        (
            Self,
            Option<
                crate::domain_computation::application_aftermath::WorthQueryPendingDispatchOutbox,
            >,
        ),
        &'static str,
    > {
        let idempotency = dispatch_basis.idempotency;
        let outcome_identity = dispatch_basis.outcome_identity;
        let mut batch = provider.bind_application_idempotency_intent(
            self.batch,
            idempotency,
            outcome_identity,
            emitted_effect_count,
        );
        if let Some(causality) = aftermath_causality {
            batch = provider.bind_application_aftermath_causality_intent(
                batch,
                causality,
                outcome_identity,
            );
        }
        let (batch, dispatch_outbox) =
            provider.bind_application_dispatch_outbox(batch, dispatch_basis)?;
        Ok((
            Self {
                lowered: self.lowered,
                batch,
                emissions: self.emissions,
                output_correspondence: self.output_correspondence,
            },
            dispatch_outbox,
        ))
    }

    pub(super) fn into_emissions(self) -> WorthQueryAdmittedApplicationEmissionBatch {
        self.emissions
    }

    pub(super) fn seal_output_correspondence(
        &self,
        commit: &worth_relational::facade::transactions::CommitResult,
    ) -> super::super::effect_program::WorthQueryApplicationOutputCorrespondence {
        self.output_correspondence.clone().seal(commit)
    }
}

fn materialize_commit_projections(
    lowered: &[WorthQueryLoweredProviderEffect],
) -> (Vec<MutationIntent>, Vec<WorthQueryApplicationEmission>) {
    let mut intents = Vec::new();
    let mut emissions = Vec::new();
    for effect in lowered {
        match effect {
            WorthQueryLoweredProviderEffect::Mutation { intent, .. } => {
                intents.push(intent.clone());
            }
            WorthQueryLoweredProviderEffect::Emission(emission) => emissions.push(emission.clone()),
        }
    }
    (intents, emissions)
}
