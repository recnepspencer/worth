//! Preserve the effect-lowering refusal when binding an application proposal.
use super::WorthQueryProviderAttemptPreparation;
use crate::domain_computation::primary_graph::application_attempt::{
    provider_binding::{prepare_provider_attempt, WorthQueryPreparedApplicationProviderAttempt},
    WorthQueryApplicationAttemptDenial,
};

pub(super) fn prepare_application_provider_attempt(
    lease: &super::WorthQueryApplicationSnapshotLease,
    preparation: WorthQueryProviderAttemptPreparation,
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> Result<WorthQueryPreparedApplicationProviderAttempt, WorthQueryApplicationAttemptDenial> {
    prepare_provider_attempt(
        lease.layout.unique_fields(),
        &|entity, aspect| lease.layout.aspect_contract(entity, aspect),
        mutation_partition,
        preparation.application_effect_count,
        preparation.installed_read_scopes,
        preparation.effect_routes,
        preparation.application_projection,
        preparation.original_read,
        preparation.consumed_outputs,
        preparation.effects,
        preparation.preimage_demand,
        preparation.conditional_definition,
        preparation.output_correspondence,
        preparation.retain_output_demand_observation,
        preparation.retain_client_observation,
        preparation.producer_required_invariants,
        preparation.output_currentness_facts,
    )
    .map(|prepared| {
        prepared
            .with_workflow_origin(preparation.workflow_origin)
            .with_local_action_settlement(preparation.local_action_settlement)
            .with_body_external_emission(preparation.body_external_emission)
            .with_required_output_demand(preparation.required_output_demand)
    })
}
