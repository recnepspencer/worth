//! Native alone admits absence of publication ownership for an ordinary part.
use super::*;
use crate::transactions::data::EntityReference;

pub(super) fn require(
    candidate: &RelationalAuthoredCoupledCandidate<'_>,
    execution: &ExecutionRequest<'_, '_>,
    admission: &worth_proof::AdmittedRequiredOutputPublication,
    absence: &RelationalRequiredOutputRouteAbsence,
    meaning: &RequiredOutputPublicationMeaning,
    roles: &[RequiredOutputRoleTarget],
) -> Result<(), TransactionCommitError> {
    let binding = candidate.binding;
    let foreign = || part::reject(Cause::ForeignCoupledOwner, meaning.source_scope);
    if !matches!(
        binding.owners.source,
        super::super::super::CoupledSource::Edit(_)
    ) {
        return Err(foreign());
    }
    if binding
        .installed
        .source_scope_ownership
        .scope_kind(admission.producer())
        .is_some()
    {
        binding
            .installed
            .require_current_ownership_storage(&binding.owners.before, meaning.source_scope)?;
        return Err(foreign());
    }
    execution
        .run_native_with_checkpoint(|context| {
            let mut existing = false;
            for role in roles {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                existing |= matches!(role.target, EntityReference::Existing(_));
            }
            if !existing {
                return Err(foreign());
            }
            for owner in binding.owners.required_owners() {
                checkpoint(
                    context,
                    (meaning.family.len() + owner.origin.meaning.family.len() + 65) as u64,
                )
                .map_err(custody::execution_stop)?;
                if owner.origin.meaning.same_route(meaning) {
                    return Err(part::reject(
                        Cause::MissingCoupledOwner,
                        meaning.source_scope,
                    ));
                }
            }
            // Recheck current Native route presence, not the caller's ordinary label.
            RequiredOutputPublicationAdmission::birth_retirement_in_context(
                &binding.installed,
                &binding.owners.before,
                meaning,
                Some(absence),
                context,
            )?;
            Ok(())
        })
        .map_err(custody::execution_stop)?
        .0
}
