use std::sync::Arc;

use worth_relational::facade::transactions::{MutationIntent, WorkerIntentBatch};

use super::effect_lowering::{
    lower_provider_effect, CreatedEffectReferences, ObservedFactIndex,
    WorthQueryLoweredProviderEffect,
};
use super::{
    retained_bytes_denial, WorthQueryAdmittedApplicationEmissionBatch,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEmission;
use crate::domain_computation::WorthQueryProvisionalEffectStep;

mod expected_steps;
pub(in crate::domain_computation::primary_graph) use expected_steps::WorthQueryExpectedEffectStepPreparationWork;
use expected_steps::WorthQueryExpectedEffectSteps;

pub(super) struct WorthQueryProviderEffectAccumulator<'facts> {
    facts: ObservedFactIndex<'facts>,
    mutation_partition: worth_relational::facade::identity::PartitionId,
    created: CreatedEffectReferences,
    lowered: Vec<WorthQueryLoweredProviderEffect>,
}

pub(super) struct WorthQueryRegisteredProviderEffects {
    expected_steps: WorthQueryExpectedEffectSteps,
    batch: WorkerIntentBatch,
    emissions: WorthQueryAdmittedApplicationEmissionBatch,
    output_correspondence: super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate,
    mutation_partition: worth_relational::facade::identity::PartitionId,
}

impl<'facts> WorthQueryProviderEffectAccumulator<'facts> {
    pub(super) fn new(
        facts: &'facts [WorthQueryApplicationObservedFact],
        effects: &[WorthQueryApplicationRealizedEffect],
        mutation_partition: worth_relational::facade::identity::PartitionId,
        application_effect_count: usize,
    ) -> Self {
        let mutation_partition = effects[..application_effect_count]
            .iter()
            .find_map(|effect| match effect {
                WorthQueryApplicationRealizedEffect::CreateEntity { partition, .. } => {
                    Some(partition.resolve(mutation_partition))
                }
                _ => None,
            })
            .unwrap_or(mutation_partition);
        Self {
            facts: ObservedFactIndex::new(facts),
            mutation_partition,
            created: CreatedEffectReferences::new(effects, mutation_partition),
            lowered: Vec::with_capacity(effects.len()),
        }
    }

    pub(super) fn add_effect(
        &mut self,
        effect: WorthQueryApplicationRealizedEffect,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let lowered =
            lower_provider_effect(&self.facts, &self.created, self.mutation_partition, effect)?;
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
        let expected_steps = WorthQueryExpectedEffectSteps::from_lowered(self.lowered)?;
        Ok(WorthQueryRegisteredProviderEffects {
            expected_steps,
            batch,
            emissions,
            output_correspondence: output_correspondence
                .remap_created_partition(self.mutation_partition),
            mutation_partition: self.mutation_partition,
        })
    }
}

impl WorthQueryRegisteredProviderEffects {
    pub(super) fn expected_steps(&self) -> &[WorthQueryProvisionalEffectStep] {
        self.expected_steps.steps()
    }

    pub(super) fn shared_expected_steps(&self) -> Arc<[WorthQueryProvisionalEffectStep]> {
        self.expected_steps.shared_steps()
    }

    pub(super) const fn expected_step_preparation_work(
        &self,
    ) -> WorthQueryExpectedEffectStepPreparationWork {
        self.expected_steps.preparation_work()
    }

    pub(super) const fn batch(&self) -> &WorkerIntentBatch {
        &self.batch
    }

    pub(super) fn into_migration_batch(self) -> Result<WorkerIntentBatch, &'static str> {
        if self.emissions.len() != 0 {
            return Err("program migration cannot carry application emissions");
        }
        Ok(self.batch)
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
        let mutation_partition = self.mutation_partition;
        let mut batch = provider.bind_application_idempotency_intent(
            self.batch,
            idempotency,
            outcome_identity,
            emitted_effect_count,
            mutation_partition,
        );
        if let Some(causality) = aftermath_causality {
            batch = provider.bind_application_aftermath_causality_intent(
                batch,
                causality,
                outcome_identity,
                mutation_partition,
            );
        }
        let (batch, dispatch_outbox) =
            provider.bind_application_dispatch_outbox(batch, dispatch_basis, mutation_partition)?;
        Ok((
            Self {
                expected_steps: self.expected_steps,
                batch,
                emissions: self.emissions,
                output_correspondence: self.output_correspondence,
                mutation_partition,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCreationPartition;
    use worth_relational::facade::identity::{KindId, PartitionId};

    #[test]
    fn platform_only_create_does_not_select_the_application_mutation_partition() {
        let issued = PartitionId::new(3);
        let workflow = PartitionId::new(7);
        let effects = [WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: KindId::new(11),
            key: "transition".to_owned(),
            fields: BTreeMap::new(),
            partition: WorthQueryApplicationCreationPartition::Context(workflow),
        }];
        let accumulator = WorthQueryProviderEffectAccumulator::new(&[], &effects, issued, 0);
        assert_eq!(accumulator.mutation_partition, issued);
    }
}
