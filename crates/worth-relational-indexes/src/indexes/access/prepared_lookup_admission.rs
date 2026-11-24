use worth_relational_branch::internal_port::indexes::data::bounded_entity_field_lookup::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind as Denial,
};
use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexId;

/// Request stops take precedence over the caller's admission at every
/// candidate and completion boundary, including certification traversal.
pub(super) fn admit<E>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    index: DerivedIndexId,
    admission: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), Stop<E>> {
    worth_relational_contracts::internal_port::execution::request_custody::check_request(execution)
        .map_err(|cause| {
            Stop::Lookup(super::prepared_entity_field_lookup::denied(
                Denial::Execution(cause),
                index,
            ))
        })?;
    admission().map_err(Stop::Admission)
}
