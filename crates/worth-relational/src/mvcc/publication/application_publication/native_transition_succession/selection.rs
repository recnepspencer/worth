//! Select one exact current route and its actual canonical publication.
use super::*;

pub(super) fn select(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<
    Option<(
        Arc<crate::history::data::CanonicalCommitEnvelope>,
        Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    )>,
    TransactionCommitError,
> {
    if installed.runtime_instance_id != inputs.projection.runtime_identity {
        return Err(origin_denial(
            "foreign installed owner for retirement successor",
        ));
    }
    if basis.descriptor().runtime_instance_id() != inputs.projection.runtime_identity
        || !basis.is_current()
    {
        return Err(origin_denial(
            "retirement successor requires the exact current native basis",
        ));
    }
    let selected = execution
        .run_native_with_checkpoint(|context| {
            basis
                .inner
                .root
                .output_publication_positions
                .get(meaning, context)
        })
        .map_err(custody::execution_stop)?
        .0?;
    consult_stage(execution, CommitStage::Admission)?;
    let selected = if selected.is_none()
        && basis
            .inner
            .root
            .output_publication_positions
            .unresolved_prefix()
            .is_some()
    {
        let current = if installed.source_scope_ownership.is_empty() {
            None
        } else {
            crate::publication::ownership::route_selection::read_current(execution, basis, meaning)?
        };
        match current {
            Some(position) => Some(position),
            None => {
                let historical = crate::history::data::readmit_required_output_route(
                    execution, inputs, basis, meaning,
                )
                .map_err(custody::cold_stop)?;
                if installed.source_scope_ownership.is_empty() {
                    historical
                } else {
                    crate::history::data::current_route_presence::require_current_owned_route(
                        execution,
                        inputs.canonical_history,
                        basis,
                        meaning,
                        historical,
                    )?
                }
            }
        }
    } else {
        selected
    };
    let Some(RequiredOutputRoutePosition::Published(commit)) = selected else {
        return Ok(None);
    };
    let envelope = inputs
        .canonical_history
        .demand_commit_envelope(execution, commit)
        .map_err(custody::execution_stop)?
        .ok_or_else(|| origin_denial("selected retirement successor history absent"))?;
    let origin = execution
        .run_native_with_checkpoint(|context| {
            envelope
                .validate_output_origins(context)
                .and_then(|origins| origins.output_origin_for_route(meaning, context))
                .map(Arc::clone)
        })
        .map_err(custody::execution_stop)?
        .0
        .map_err(custody::cold_stop)?;
    Ok(Some((envelope, origin)))
}
