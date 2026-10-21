//! Admission of native whole-struct effects under actual installed field grants.
use super::{TouchAdmissionFailure as Failure, WorthQueryPrimaryGraphLayout};
use worth_foundational::facade::{AbsenceLaw, AspectMask, AspectShape, MutationMask};
use worth_query_installation::facade::{
    WorthQueryOperationTouchContract, WorthQueryOperationTouchScope,
};
use worth_relational::facade::mvcc::ValidatedEntityStructTouch;

pub(super) fn admit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    actual: &ValidatedEntityStructTouch<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    touches: &WorthQueryOperationTouchContract,
) -> Result<usize, Failure> {
    let entity = layout.entity_name(actual.kind()).ok_or(())?;
    let contract = actual.contract();
    let field = actual.field();
    if layout.aspect_contract(entity, contract.key()) != Some(contract)
        || !matches!(contract.shape(), AspectShape::Struct(_))
    {
        return Err(().into());
    }
    match contract.absence() {
        AbsenceLaw::Optional => {}
        AbsenceLaw::Required if actual.before_present() && actual.after_present() => {}
        AbsenceLaw::Required | AbsenceLaw::Defaulted => return Err(().into()),
    }
    let (index, grant) = touches
        .scopes()
        .iter()
        .enumerate()
        .find_map(|(index, scope)| {
            let WorthQueryOperationTouchScope::WriteField(grant) = scope else {
                return None;
            };
            (grant.entity() == entity
                && grant.contract() == contract
                && grant.field_path().fields().len() == 1
                && &grant.field_path().fields()[0] == field)
                .then_some((index, grant))
        })
        .ok_or(())?;
    // This is a concrete declared field mask, never an implicit whole-aspect grant.
    let bytes = grant
        .field_path()
        .owned_allocation_capacity_bytes()
        .checked_add(std::mem::size_of::<
            worth_foundational::facade::CanonicalFieldPath,
        >())
        .ok_or(())?;
    execution
        .consult()
        .map_err(|stop| projection_stop(stop.into()))?;
    let _memory = execution
        .in_scope(|lease| {
            let reservation = match lease {
                Some(lease) => worth_execution::ExecutionAllocationPolicy::Execution(lease)
                    .reserve_memory(bytes as u64)
                    .map_err(|cause| {
                        Failure::Projection(worth_relational::facade::mvcc::ValidatedMutationTouchProjectionError::Allocation(cause))
                    }),
                None => worth_execution::ExecutionMemoryReservation::reserve_in_scope(None, bytes as u64)
                    .map(Some)
                    .map_err(|stop| {
                        projection_stop(worth_execution::WorkCeilingDenial::Admission(stop).into())
                    }),
            };
            execution
                .consult()
                .map_err(|stop| projection_stop(stop.into()))?;
            reservation
        })
        .map_err(|stop| projection_stop(stop.into()))??;
    execution
        .consult()
        .map_err(|stop| projection_stop(stop.into()))?;
    let mask = AspectMask::<MutationMask>::new([grant.field_path().clone()]);
    contract
        .admits_mutation_mask(&mask)
        .map_err(|_| Failure::Contract)?;
    Ok(index)
}

fn projection_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Failure {
    Failure::Projection(
        worth_relational::facade::mvcc::ValidatedMutationTouchProjectionError::Execution(
            worth_relational::facade::transactions::CommitExecutionDenial {
                kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(
                    cause,
                ),
                partition_identity: None,
            },
        ),
    )
}
