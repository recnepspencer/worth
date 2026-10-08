//! Preserve the effect-lowering refusal when binding an application proposal.
use super::WorthQueryProviderAttemptPreparation;
use crate::domain_computation::primary_graph::application_attempt::{
    provider_binding::{prepare_provider_attempt, WorthQueryPreparedApplicationProviderAttempt},
    WorthQueryApplicationAttemptDenial,
};

pub(super) fn prepare_application_provider_attempt(
    preparation: WorthQueryProviderAttemptPreparation,
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> Result<WorthQueryPreparedApplicationProviderAttempt, WorthQueryApplicationAttemptDenial> {
    prepare_provider_attempt(
        mutation_partition,
        preparation.application_effect_count,
        preparation.installed_read_scopes,
        preparation.facts,
        preparation.consumed_outputs,
        preparation.effects,
        preparation.emission_retained_bytes,
        preparation.emission_retained_bytes_ceiling,
        preparation.preimage_demand,
        preparation.conditional_definition,
        preparation.output_correspondence,
        preparation.retain_output_demand_observation,
        preparation.retain_client_observation,
        preparation.producer_required_invariants,
        preparation.output_currentness_facts,
    )
    .map(|prepared| prepared.with_required_output_demand(preparation.required_output_demand))
}
