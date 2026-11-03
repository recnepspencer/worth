//! Predecode admissions mirror only the actual escaping merge leaf owners.
use super::*;
use crate::history::data::MergeBasisAncestors;
use crate::merge::data::{
    MergeBasisMemory, MergeIdentityMemory, MergeLeafMemory, MergeReadmissionMemory,
    RelationalMergeCorrespondenceWitnessRow, RelationalSchemaReconciliationWitnessRow,
};
mod policies;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(in super::super::super) fn prepare_readmission(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<MergeReadmissionMemory>, Refusal> {
    let canonical = field(BorrowedPayload(payload), "canonical", request)?;
    prepare_canonical_readmission(canonical, request)
}
pub(in super::super::super) fn prepare_canonical_readmission(
    canonical: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<MergeReadmissionMemory>, Refusal> {
    let mut merge = None;
    canonical.fields(request, |name, value| {
        if name == "merge_execution_authority" && !value.is_none() && merge.replace(value).is_some()
        {
            return Err(malformed());
        }
        Ok(())
    })?;
    let Some(merge) = merge else {
        return Ok(None);
    };
    let summary = field(merge, "execution_summary", request)?;
    let correspondence = field(
        field(summary, "correspondence_witness", request)?,
        "rows",
        request,
    )?;
    let schema = field(
        field(summary, "schema_reconciliation_witness", request)?,
        "rows",
        request,
    )?;
    let strategy = field(summary, "strategy_witness", request)?;
    // Quote the exact controlled tree buffers before creating any of them.
    let policy_tree_bytes = policies::tree_storage(strategy, request)?;
    let tree_bytes = tree_storage::<MergeIdentityMemory>(count(correspondence, request)?)?
        .checked_add(tree_storage::<MergeIdentityMemory>(count(
            schema, request,
        )?)?)
        .and_then(|n| n.checked_add(policy_tree_bytes))
        .ok_or_else(malformed)?;
    let tree_memory = request
        .reserve_retained_memory(tree_bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let basis_memory = basis(field(summary, "branch_basis", request)?, request)?;
    let mut correspondence_identities = Vec::with_capacity(count(correspondence, request)?);
    let mut correspondence_children = 0u64;
    correspondence.elements(request, |row| {
        let raw = field(row, "authority_basis", request)?;
        checked_sum(
            &mut correspondence_children,
            identities::basis(raw, request)?.retained,
        )?;
        correspondence_identities.push(identity(raw, request)?);
        Ok(())
    })?;
    let correspondence = row_memory::<RelationalMergeCorrespondenceWitnessRow>(
        correspondence,
        request,
        |row| rows::correspondence_row(row, request),
        correspondence_children,
    )?;
    let mut schema_identities = Vec::with_capacity(count(schema, request)?);
    let mut schema_children = 0u64;
    schema.elements(request, |row| {
        let link = row.optional_field("correspondence_linkage", request)?;
        let identity_memory = if link.is_none() {
            MergeIdentityMemory::default()
        } else {
            let raw = field(link, "basis", request)?;
            checked_sum(
                &mut schema_children,
                identities::basis(raw, request)?.retained,
            )?;
            identity(raw, request)?
        };
        schema_identities.push(identity_memory);
        Ok(())
    })?;
    let schema = row_memory::<RelationalSchemaReconciliationWitnessRow>(
        schema,
        request,
        |row| rows::schema_row(row, request),
        schema_children,
    )?;
    let strategy = policies::prepare(strategy, request)?;
    let packet = field(summary, "proof_packet", request)?;
    let packet_basis = basis(field(packet, "branch_basis", request)?, request)?;
    let packet_surface = row_memory::<crate::merge::data::RelationalMergeAdmittedSurfaceRow>(
        field(packet, "admitted_merge_surface", request)?,
        request,
        |_| Ok(OwnerAllocation::default()),
        0,
    )?;
    Ok(Some(MergeReadmissionMemory {
        basis: basis_memory,
        correspondence,
        correspondence_identities,
        schema,
        schema_identities,
        strategy,
        packet_basis,
        packet_surface,
        _tree_memory: tree_memory,
    }))
}

pub(super) fn retained_leaf_bytes(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let summary = field(value, "execution_summary", request)?;
    let mut bytes = basis_storage(field(summary, "branch_basis", request)?, request)?;
    let correspondence = field(
        field(summary, "correspondence_witness", request)?,
        "rows",
        request,
    )?;
    checked_sum(
        &mut bytes,
        shared_rows::<RelationalMergeCorrespondenceWitnessRow>(correspondence, request, |row| {
            rows::correspondence_row(row, request)
        })?
        .retained,
    )?;
    let schema = field(
        field(summary, "schema_reconciliation_witness", request)?,
        "rows",
        request,
    )?;
    checked_sum(
        &mut bytes,
        shared_rows::<RelationalSchemaReconciliationWitnessRow>(schema, request, |row| {
            rows::schema_row(row, request)
        })?
        .retained,
    )?;
    checked_sum(
        &mut bytes,
        policies::retained_leaf_bytes(field(summary, "strategy_witness", request)?, request)?,
    )?;
    let packet = field(summary, "proof_packet", request)?;
    checked_sum(
        &mut bytes,
        basis_storage(field(packet, "branch_basis", request)?, request)?,
    )?;
    checked_sum(
        &mut bytes,
        shared_rows::<crate::merge::data::RelationalMergeAdmittedSurfaceRow>(
            field(packet, "admitted_merge_surface", request)?,
            request,
            |_| Ok(OwnerAllocation::default()),
        )?
        .retained,
    )?;
    Ok(bytes)
}

fn basis_storage(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let base = field(value, "merge_base", request)?;
    let mut bytes = 0;
    for name in ["supporting_left_ancestors", "supporting_right_ancestors"] {
        checked_sum(
            &mut bytes,
            MergeBasisAncestors::storage_bytes(count(field(base, name, request)?, request)?)
                .ok_or_else(malformed)?,
        )?;
    }
    Ok(bytes)
}
fn basis(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<MergeBasisMemory, Refusal> {
    let base = field(value, "merge_base", request)?;
    let ancestor = |name| {
        let count = count(field(base, name, request)?, request)?;
        leaf(
            count,
            MergeBasisAncestors::storage_bytes(count).ok_or_else(malformed)?,
            request,
        )
    };
    Ok(MergeBasisMemory {
        left: ancestor("supporting_left_ancestors")?,
        right: ancestor("supporting_right_ancestors")?,
    })
}

fn identity(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<MergeIdentityMemory, Refusal> {
    let mut memory = MergeIdentityMemory::default();
    if value.is_map() {
        let bytes = identities::basis(value, request)?.retained;
        value.fields(request, |variant, raw| {
            match variant {
                "DeclaredKeySet" => memory.keys = Some(leaf(count(raw, request)?, bytes, request)?),
                "Custom" => {
                    let length = field(raw, "name", request)?.text(request)?.len();
                    memory.custom_name = Some(leaf(length, bytes, request)?);
                }
                _ => {}
            }
            Ok(())
        })?;
    }
    Ok(memory)
}

fn row_memory<T>(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    nested: impl FnMut(BorrowedPayload<'_>) -> Result<OwnerAllocation, Refusal>,
    children: u64,
) -> Result<MergeLeafMemory, Refusal> {
    let bytes = shared_rows::<T>(value, request, nested)?
        .retained
        .checked_sub(children)
        .ok_or_else(malformed)?;
    leaf(count(value, request)?, bytes, request)
}
fn leaf(
    count: usize,
    bytes: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<MergeLeafMemory, Refusal> {
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    Ok(MergeLeafMemory::new(count, bytes, memory))
}
fn field<'a>(
    value: BorrowedPayload<'a>,
    expected: &str,
    request: &ExecutionRequest<'_, '_>,
) -> Result<BorrowedPayload<'a>, Refusal> {
    let mut result = None;
    value.fields(request, |name, value| {
        if name == expected && result.replace(value).is_some() {
            return Err(malformed());
        }
        Ok(())
    })?;
    result.ok_or_else(malformed)
}
fn count(value: BorrowedPayload<'_>, request: &ExecutionRequest<'_, '_>) -> Result<usize, Refusal> {
    value.elements(request, |_| Ok(()))
}
fn checked_sum(total: &mut u64, bytes: u64) -> Result<(), Refusal> {
    *total = total.checked_add(bytes).ok_or_else(malformed)?;
    Ok(())
}
fn tree_storage<T>(count: usize) -> Result<u64, Refusal> {
    u64::try_from(
        std::alloc::Layout::array::<T>(count)
            .map_err(|_| malformed())?
            .size(),
    )
    .map_err(|_| malformed())
}
