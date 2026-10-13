use crate::indexes::data::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind as Denial,
    DerivedIndexId,
};

/// Request stops take precedence over the caller's admission at every
/// candidate and completion boundary, including certification traversal.
pub(super) fn admit<E>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    index: DerivedIndexId,
    admission: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), Stop<E>> {
    crate::execution::check_request(execution).map_err(|cause| {
        Stop::Lookup(super::prepared_entity_field_lookup::denied(
            Denial::Execution(cause),
            index,
        ))
    })?;
    admission().map_err(Stop::Admission)
}
