//! Preflight and lifetime custody for ordinary region reconstruction.
pub(in crate::storage::residency) mod aspect_state;
pub(in crate::storage::residency) mod region_image;
#[cfg(test)]
mod tests;
use super::super::{RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal;
use crate::durability::data::{
    EntityExtraImage, RelationExtraImage, VersionedEntityMetadataImage,
    VersionedRelationMetadataImage,
};
use crate::identity::data::PartitionId;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{SharedColumn, SharedMap, StoredValue};
use std::{alloc::Layout, collections::BTreeMap};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
pub(in crate::storage::residency) type Result<T> = std::result::Result<T, Failure>;
pub(in crate::storage::residency) use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload as Raw;

#[derive(Clone, Copy, Default)]
pub(in crate::storage::residency) struct Quote {
    pub(in crate::storage::residency) wire: u64,
    pub(in crate::storage::residency) native: u64,
}
impl Quote {
    pub(in crate::storage::residency) fn add(&mut self, other: Self) -> Result<()> {
        self.wire = sum(self.wire, other.wire)?;
        self.native = sum(self.native, other.native)?;
        Ok(())
    }
    pub(in crate::storage::residency) fn bytes(wire: u64, native: u64) -> Self {
        Self { wire, native }
    }
}
pub(in crate::storage::residency) fn overflow() -> Failure {
    Failure::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
}
pub(in crate::storage::residency) fn sum(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or_else(overflow)
}
pub(in crate::storage::residency) fn times(a: u64, b: usize) -> Result<u64> {
    a.checked_mul(b as u64).ok_or_else(overflow)
}
pub(in crate::storage::residency) fn array<T>(count: usize) -> Result<u64> {
    Layout::array::<T>(count)
        .ok()
        .and_then(|layout| u64::try_from(layout.size()).ok())
        .ok_or_else(overflow)
}
pub(in crate::storage::residency) fn shared<T>() -> Result<u64> {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .ok_or_else(overflow)
}
pub(in crate::storage::residency) fn vector<T>(count: usize) -> Result<u64> {
    if count == 0 {
        return Ok(0);
    }
    // Pinned Serde Vec growth plus old/new relocation coexistence.
    let capacity = count.checked_mul(2).ok_or_else(overflow)?.max(8);
    times(array::<T>(capacity)?, 2)
}
pub(in crate::storage::residency) fn tree<K, V>(count: usize) -> Result<u64> {
    if count == 0 {
        return Ok(0);
    }
    let node = Layout::new::<(usize, u16, u16)>()
        .extend(Layout::new::<[K; 11]>())
        .ok()
        .and_then(|(l, _)| l.extend(Layout::new::<[V; 11]>()).ok())
        .and_then(|(l, _)| l.pad_to_align().extend(Layout::new::<[usize; 12]>()).ok())
        .and_then(|(l, _)| u64::try_from(l.pad_to_align().size()).ok())
        .ok_or_else(overflow)?;
    times(node, count.div_ceil(5).checked_add(2).ok_or_else(overflow)?)
}
pub(in crate::storage::residency) fn column<T: StoredValue>(count: usize) -> Result<u64> {
    if count == 0 {
        return Ok(0);
    }
    // Dense bootstrap builds fewer than two nodes per page. The existing
    // adoption quote includes two nodes and the real 32-carrier leaf backing.
    let page = SharedColumn::<T>::default()
        .adoption_allocation_bound(0)
        .ok_or_else(overflow)?;
    sum(
        times(page, count.div_ceil(32))?,
        sum(
            times(T::allocation_bytes(), count)?,
            vector::<T::Value>(count)?,
        )?,
    )
}
pub(in crate::storage::residency) fn map<K: Ord + Copy, V: StoredValue>(
    count: usize,
) -> Result<u64> {
    let node = SharedMap::<K, V>::default()
        .adoption_allocation_bound()
        .ok_or_else(overflow)?;
    sum(times(node, count)?, times(V::allocation_bytes(), count)?)
}
pub(in crate::storage::residency) fn wire_stop(
    error: RelationalCanonicalCommitArtifactRefusal,
) -> Failure {
    match error {
        RelationalCanonicalCommitArtifactRefusal::Request(cause) => Failure::Request(cause),
        _ => damaged(),
    }
}
pub(in crate::storage::residency) fn damaged() -> Failure {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Region(PartitionId::main()),
        cause: RelationalStorageUnavailableCause::Damaged,
    }
    .into()
}
pub(in crate::storage::residency) fn record<'a>(
    raw: Raw<'a>,
    names: &[&str],
    request: &ExecutionRequest<'_, '_>,
    mut visit: impl FnMut(usize, Raw<'a>) -> Result<()>,
) -> Result<()> {
    if matches!(raw.0.first(), Some(0x90..=0x9f | 0xdc | 0xdd)) {
        let mut index = 0;
        raw.elements(request, |value| {
            if index >= names.len() {
                return Err(
                    crate::durability::canonical_commit_artifact::borrowed_payload::malformed(),
                );
            }
            let result = visit(index, value);
            index += 1;
            result.map_err(to_wire_stop)
        })
        .map_err(wire_stop)?;
    } else {
        raw.fields(request, |name, value| {
            let index = names.iter().position(|known| *known == name).ok_or_else(
                crate::durability::canonical_commit_artifact::borrowed_payload::malformed,
            )?;
            visit(index, value).map_err(to_wire_stop)
        })
        .map_err(wire_stop)?;
    }
    Ok(())
}
fn to_wire_stop(cause: Failure) -> RelationalCanonicalCommitArtifactRefusal {
    match cause {
        Failure::Request(cause) => RelationalCanonicalCommitArtifactRefusal::Request(cause),
        Failure::StorageUnavailable(_) => {
            crate::durability::canonical_commit_artifact::borrowed_payload::malformed()
        }
    }
}
pub(in crate::storage::residency) fn sequence(
    raw: Raw<'_>,
    request: &ExecutionRequest<'_, '_>,
    mut visit: impl FnMut(Raw<'_>) -> Result<()>,
) -> Result<usize> {
    raw.elements(request, |value| visit(value).map_err(to_wire_stop))
        .map_err(wire_stop)
}
pub(in crate::storage::residency) fn pairs(
    raw: Raw<'_>,
    request: &ExecutionRequest<'_, '_>,
    mut visit: impl FnMut(Raw<'_>, Raw<'_>) -> Result<()>,
) -> Result<usize> {
    raw.pairs(request, |key, value| {
        visit(key, value).map_err(to_wire_stop)
    })
    .map_err(wire_stop)
}
pub(in crate::storage::residency) fn text(
    raw: Raw<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64> {
    raw.text_length(request)
        .map(|n| n as u64)
        .map_err(wire_stop)
}
pub(in crate::storage::residency) fn work(
    request: &ExecutionRequest<'_, '_>,
    units: u64,
) -> Result<()> {
    let mut answer = None;
    let scope = request.run_native_with_checkpoint(|context| {
        answer = Some(crate::execution::request_stage::checkpoint(context, units));
    });
    match (scope, answer) {
        (_, Some(Err(cause))) => Err(Failure::Request(
            worth_execution::WorkCeilingDenial::Stopped(cause).into(),
        )),
        (Err(cause), _) => Err(Failure::Request(cause.into())),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed materialization charge executes its callback"),
    }
}
/// One ticket stays with the actual backing. Growth is admitted before allocation.
#[derive(Debug, Default)]
pub(in crate::storage::residency) struct Funding {
    memory: Option<ExecutionMemoryReservation>,
    bytes: u64,
}
impl Funding {
    pub(in crate::storage::residency) fn reserve(
        bytes: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self> {
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| Failure::Request(cause.into()))?;
        Ok(Self { memory, bytes })
    }
    pub(in crate::storage::residency) fn grow(
        &mut self,
        bytes: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<()> {
        work(request, 1)?;
        let total = sum(self.bytes, bytes)?;
        if let Some(memory) = &mut self.memory {
            memory.resize(total).map_err(|cause| {
                Failure::Request(
                    worth_execution::WorkCeilingDenial::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    )
                    .into(),
                )
            })?;
        } else {
            self.memory = request
                .reserve_retained_memory(total)
                .map_err(|cause| Failure::Request(cause.into()))?;
        }
        self.bytes = total;
        Ok(())
    }
    pub(in crate::storage::residency) fn into_memory(self) -> Option<ExecutionMemoryReservation> {
        self.memory
    }
}
pub(in crate::storage::residency) struct MaterializationFunding {
    pub(in crate::storage::residency) retained: Funding,
    _transient: Funding,
}
impl MaterializationFunding {
    pub(in crate::storage::residency) fn admit(
        quote: Quote,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self> {
        Ok(Self {
            retained: Funding::reserve(sum(quote.native, shared::<ResidentRegion>()?)?, request)?,
            _transient: Funding::reserve(quote.wire, request)?,
        })
    }
}
/// The catalog is a transient copy. Each admitted value separately owns its contract.
pub(in crate::storage::residency) use aspect_state::contracts;

pub(in crate::storage::residency) fn ordered(
    cause: worth_execution::ExecutionOrderedMapDenial,
) -> Failure {
    use worth_execution::{
        ExecutionAllocationDenialClass as Class, ExecutionOrderedMapDenial as Denial,
    };
    match cause {
        Denial::Request(cause) => Failure::Request(cause.into()),
        Denial::CapacityOverflow => overflow(),
        Denial::Allocation(cause) => match cause.classify() {
            Class::Request(cause) => Failure::Request(cause.into()),
            Class::Physical(_) => {
                Failure::Request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
            }
        },
    }
}

pub(in crate::storage::residency) fn current(
    partition: &PartitionState,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let inventory = partition.allocation_inventory();
    let native = sum(
        inventory.authoritative_bytes,
        sum(
            inventory.retention_metadata_bytes,
            inventory.diagnostic_bytes,
        )?,
    )?;
    // Existing shared native pages may survive through conversion. Their
    // complete physical inventory bounds that native lifetime, not wire size.
    let mut q = Quote::bytes(native, native);
    let e = &partition.entity_arena;
    let r = &partition.relation_arena;
    for state in e
        .metadata_history
        .iter()
        .flat_map(|rows| rows.iter())
        .map(|meta| meta.authoritative_aspect_state())
        .chain(
            r.metadata_history
                .iter()
                .flat_map(|rows| rows.iter())
                .map(|meta| meta.authoritative_aspect_state()),
        )
        .chain(
            e.extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state()),
        )
        .chain(
            r.extra
                .iter()
                .map(|extra| extra.authoritative_aspect_state()),
        )
    {
        work(request, 1)?;
        q.add(aspect_state::native(state, contract)?)?;
    }
    for rows in e.metadata_history.iter() {
        q.wire = sum(q.wire, vector::<VersionedEntityMetadataImage>(rows.len())?)?;
    }
    for rows in r.metadata_history.iter() {
        q.wire = sum(
            q.wire,
            vector::<VersionedRelationMetadataImage>(rows.len())?,
        )?;
    }
    // Every fixed wire column has one entry per physical row. Tuple padding
    // is conservative for the simultaneous separate scalar Vec backings.
    q.wire = sum(
        q.wire,
        super::super::region_capture::complete::current_scalar_wire([
            e.generations.len(),
            r.generations.len(),
        ])?,
    )?;
    for rows in [&e.aspect_versions, &r.aspect_versions] {
        q.wire = sum(
            q.wire,
            vector::<BTreeMap<crate::symbols::data::Symbol, u64>>(rows.len())?,
        )?;
        for row in rows.iter() {
            work(request, 1)?;
            q.wire = sum(
                q.wire,
                tree::<crate::symbols::data::Symbol, u64>(row.len())?,
            )?;
        }
    }
    for rows in [&e.field_revisions, &r.field_revisions] {
        q.wire = sum(
            q.wire,
            vector::<
                Option<
                    BTreeMap<
                        (crate::symbols::data::Symbol, crate::symbols::data::Symbol),
                        crate::storage::data::RelationalFieldRevision,
                    >,
                >,
            >(rows.len())?,
        )?;
        for row in rows.iter().flatten() {
            work(request, 1)?;
            q.wire = sum(
                q.wire,
                tree::<
                    (crate::symbols::data::Symbol, crate::symbols::data::Symbol),
                    crate::storage::data::RelationalFieldRevision,
                >(row.len())?,
            )?;
        }
    }
    q.wire = sum(q.wire, super::current_diagnostic_wire(partition, request)?)?;
    // Concrete owned wire column shells and per-row Vec headers coexist with
    // shared native columns. The metadata payloads above are separate owners.
    q.wire = sum(
        q.wire,
        sum(
            vector::<EntityExtraImage>(partition.entity_arena.extra.len())?,
            vector::<RelationExtraImage>(partition.relation_arena.extra.len())?,
        )?,
    )?;
    q.wire = sum(
        q.wire,
        sum(
            vector::<Vec<VersionedEntityMetadataImage>>(partition.entity_arena.extra.len())?,
            vector::<Vec<VersionedRelationMetadataImage>>(partition.relation_arena.extra.len())?,
        )?,
    )?;
    work(request, sum(q.wire, q.native)?)?;
    Ok(q)
}

pub(in crate::storage::residency) use super::{at_region, content, scalars};
