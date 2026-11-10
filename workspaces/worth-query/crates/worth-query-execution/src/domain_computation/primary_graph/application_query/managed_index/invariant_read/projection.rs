//! Materialize candidates only through the current Native loan's occurrence and fields.
use super::super::WorthQueryManagedIndexSourceSlot;
use super::*;
use worth_relational::facade::runtime::ManagedIndexSourceRows;
type Denial = WorthQueryManagedIndexInvariantDenial;

pub(super) fn read<S, I>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &ManagedIndexSourceRead<'_>,
    program: &worth_relational::facade::runtime::ManagedIndexSourceProgram,
    root: &ManagedSourceRoot<S, I>,
    rows: &ManagedIndexSourceRows<'_>,
    selected: crate::domain_computation::primary_graph::access_strategy::coverage::ReceivedSelection,
) -> Result<Vec<ManagedSourceValue<S, I>>, Denial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    ManagedSourceValue<S, I>:
        WorthQueryApplicationProjection<S, <I::SourceBinding as ApplicationQueryBinding<S>>::Query>,
{
    if root.rows.program() != program
        || rows.program() != program
        || root.rows.scope() != rows.scope()
    {
        return Err(NativeDenial::ForeignSource.into());
    }
    // The fresh Native loan owns currency and audience. The sealed root owns
    // complete material; reconstructing membership here would walk its history.
    let slots = selected
        .complete_slots()
        .map_err(|cause| Denial::Provider(cause.into()))?;
    let budget = source.read_budget();
    let mut output = reserve_output::<ManagedSourceValue<S, I>>(&budget, slots.len())?;
    decoded_provider(execution, &root.bindings, |control, decode| {
        let control = control.under_source(&budget);
        let result: Result<(), Denial> = (|| {
            for slot in slots {
                control
                    .checkpoint(1)
                    .map_err(|cause| Denial::Provider(cause.into()))?;
                let entity = admit_slot(root.rows.owner(), slot)?;
                let node = root.rows.get(entity).ok_or(NativeDenial::ForeignSource)?;
                let row = WorthQueryApplicationProjectionRow::new(
                    node,
                    &WorthQueryApplicationQueryGovernance::Public,
                    decode,
                );
                output.push(ManagedSourceValue::<S, I>::project(&row).map_err(Denial::Projection)?);
            }
            Ok(())
        })();
        control.poll()?;
        Ok(result)
    })
    .map_err(Denial::Provider)??;
    Ok(output)
}
fn admit_slot(
    owner: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    slot: &WorthQueryManagedIndexSourceSlot,
) -> Result<worth_relational::facade::identity::EntityId, Denial> {
    if slot.owner != owner || slot.ordinal != 0 {
        return Err(NativeDenial::ForeignSource.into());
    }
    slot.entity
        .ok_or_else(|| NativeDenial::ForeignSource.into())
}
fn reserve_output<T>(
    budget: &worth_relational::facade::runtime::CustomInvariantReadBudget<'_>,
    count: usize,
) -> Result<Vec<T>, Denial> {
    use worth_relational::facade::{
        runtime::DerivedRelationReadDenial, transactions::RelationalExecutionDenialCause as Cause,
    };
    let bytes = count
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| {
            Denial::Native(DerivedRelationReadDenial::Request(Cause::ChargedBytesOverflow).into())
        })?;
    budget
        .claim_retained(bytes)
        .map_err(|cause| Denial::Native(DerivedRelationReadDenial::Structural(cause).into()))?;
    let mut output = Vec::new();
    output.try_reserve_exact(count).map_err(|_| {
        Denial::Native(DerivedRelationReadDenial::Request(Cause::AllocationFailed).into())
    })?;
    Ok(output)
}
