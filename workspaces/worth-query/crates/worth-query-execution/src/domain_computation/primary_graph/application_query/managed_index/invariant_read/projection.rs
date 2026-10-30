//! Materialize candidates only through the current Native loan's occurrence and fields.
use super::super::WorthQueryManagedIndexSourceSlot;
use super::*;
use worth_relational::facade::runtime::{ManagedIndexSourceOccurrence, ManagedIndexSourceRows};
type Denial = WorthQueryManagedIndexInvariantDenial;

pub(super) fn read<S, I>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &ManagedIndexSourceRead<'_>,
    program: &worth_relational::facade::runtime::ManagedIndexSourceProgram,
    root: &ManagedSourceRoot<S, I>,
    rows: &ManagedIndexSourceRows<'_>,
    slots: &[WorthQueryManagedIndexSourceSlot],
) -> Result<Vec<ManagedSourceValue<S, I>>, Denial>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    ManagedSourceValue<S, I>:
        WorthQueryApplicationProjection<S, <I::SourceBinding as ApplicationQueryBinding<S>>::Query>,
{
    let budget = source.read_budget();
    let mut output = reserve_output::<ManagedSourceValue<S, I>>(&budget, slots.len())?;
    decoded_provider(execution, &root.bindings, |control, decode| {
        let control = control.under_source(&budget);
        let result: Result<(), Denial> = (|| {
            for slot in slots {
                control
                    .checkpoint(1)
                    .map_err(|cause| Denial::Provider(cause.into()))?;
                let occurrence = admit_occurrence(rows, root.rows.owner(), slot)?;
                let (tree, _allocation) = project_row(
                    execution,
                    program,
                    rows,
                    occurrence,
                    root.installed
                        .source_read()
                        .query()
                        .read_graph()
                        .projections(),
                )
                .map_err(|cause| Denial::Native(cause.into_native()))?;
                let row = WorthQueryApplicationProjectionRow::new(
                    tree.only().expect("one actual Native Source occurrence"),
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
fn admit_occurrence(
    rows: &ManagedIndexSourceRows<'_>,
    owner: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    slot: &WorthQueryManagedIndexSourceSlot,
) -> Result<ManagedIndexSourceOccurrence, Denial> {
    if slot.owner != owner || slot.ordinal != 0 {
        return Err(NativeDenial::ForeignSource.into());
    }
    let entity = slot.entity.ok_or(NativeDenial::ForeignSource)?;
    rows.occurrence(entity)
        .map_err(NativeDenial::from)?
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
