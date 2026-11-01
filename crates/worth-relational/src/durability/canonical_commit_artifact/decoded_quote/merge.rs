//! Published merge authority retains its actual request and proof owners.
use super::*;

mod admission;
mod policies;
mod rows;
pub(in super::super) use admission::prepare_canonical_readmission;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "execution_summary" {
            add(&mut bytes, execution(value, request)?)?;
        }
        Ok(())
    })?;
    bytes.retained = bytes
        .retained
        .checked_sub(admission::retained_leaf_bytes(value, request)?)
        .ok_or_else(malformed)?;
    Ok(bytes)
}

fn shared_rows<T>(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    nested: impl FnMut(BorrowedPayload<'_>) -> Result<OwnerAllocation, Refusal>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_vector::<T>(value, request, nested)?;
    add(
        &mut bytes,
        crate::merge::data::MergeWitnessRows::<T>::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}

fn execution(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "request" => rows::request(value, request)?,
                "branch_basis" => branches::merge_basis(value, request)?,
                "correspondence_witness" => rows::correspondence(value, request)?,
                "schema_reconciliation_witness" => rows::schema(value, request)?,
                "strategy_witness" => policies::strategy(value, request)?,
                "proof_packet" => policies::packet(value, request)?,
                "diagnostics_digest" | "execution_digest" => text(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    // Summary consistency recomputes the branch-basis digest at several
    // independent comparisons; each exact 64-byte result is temporary.
    bytes.transient = bytes.transient.checked_add(3 * 64).ok_or_else(malformed)?;
    Ok(bytes)
}
