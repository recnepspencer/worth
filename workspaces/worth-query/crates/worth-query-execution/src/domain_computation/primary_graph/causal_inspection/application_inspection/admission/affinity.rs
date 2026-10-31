//! Allocation-free custody checks precede selection of protected state.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationQueryBinding,
};

/// Structural custody is checked before selecting or resolving protected state.
/// This shares only the actual installed Arc; it sizes or traverses no metadata.
pub(in crate::domain_computation::primary_graph::causal_inspection::application_inspection) fn validate_original_affinity<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    installed: &WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
    original: &super::super::super::application_observation::WorthQueryApplicationQueryObservation,
) -> Result<
    worth_query_installation::facade::WorthQueryInstalledApplicationInspectionContract,
    InspectionAdmissionDenial,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    let contract = installed
        .query()
        .binding_observation()
        .and_then(|binding| binding.inspection_contract())
        .ok_or(InspectionAdmissionDenial::MissingInspection)?;
    if original.source.runtime_authority != runtime.runtime.authority_identity().as_u64()
        || &original.source.schema_binding != contract.graph_obligations().binding_identity()
        || original.source.original_binding() != Some(contract.binding_observation())
    {
        return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
    }
    Ok(contract)
}
