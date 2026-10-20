use std::mem::{align_of, size_of};
use std::sync::Arc;

use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexKind};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::application_checkpoint::facts::{
    FactPayload, FactRetentionAllocation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;

use super::admission::IndexAdmission;
use super::{index_capacity, retention};

/// Each holder funds its own native basis. The original payload's installed
/// ledger shares its external-custody ticket; a foreign installation admits
/// independent custody without borrowing that ledger's credit.
#[derive(Clone, Debug)]
pub(super) struct RetainedFactCapacity {
    _allocation: Arc<FactRetentionAllocation>,
    _basis: Arc<RetainedInvalidationCapacity>,
}

pub(super) fn reserve(
    facts: &FactPayload,
    read_basis: &PositionedRelationalSnapshot,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<RetainedFactCapacity, CompanionPreflightStop> {
    let allocation = reserve_payload(facts, resources, admission)?;
    let basis = reserve_basis(read_basis, resources, admission)?;
    Ok(RetainedFactCapacity {
        _allocation: allocation,
        _basis: basis,
    })
}

pub(super) fn reserve_basis(
    read_basis: &PositionedRelationalSnapshot,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<Arc<RetainedInvalidationCapacity>, CompanionPreflightStop> {
    let bytes = index_capacity::arc_bytes::<PositionedRelationalSnapshot>()
        .and_then(|bytes| {
            bytes.checked_add(u64::try_from(read_basis.branch_id().0.capacity()).ok()?)
        })
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.bytes(bytes)?;
    retention::reserve(resources, bytes, admission)
}

pub(super) fn reserve_payload(
    facts: &FactPayload,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<Arc<FactRetentionAllocation>, CompanionPreflightStop> {
    admission.work(1)?;
    // Declare this owner before the guard: on every return/error path the lock
    // must be released before a potentially final upgraded allocation drops.
    let existing;
    let mut slot = facts.lock_retention();
    existing = slot.allocation.upgrade();
    if let Some(ticket) = existing
        .as_ref()
        .filter(|ticket| ticket.capacity.belongs_to(resources))
    {
        return Ok(Arc::clone(ticket));
    }
    // Reserve the actual new wrapper/cache shape before allocation. A foreign
    // live ledger receives no credit and cannot replace the original weak slot.
    let bytes = FactPayload::retention_storage_bytes()
        .and_then(|bytes| {
            bytes.checked_sub(index_capacity::arc_bytes::<RetainedInvalidationCapacity>()?)
        })
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.bytes(bytes)?;
    let storage = retention::reserve(resources, bytes, admission)?;
    let children = facts
        .retained_children()
        .map(|(left, right)| {
            Ok::<_, CompanionPreflightStop>([
                reserve_payload(left, resources, admission)?,
                reserve_payload(right, resources, admission)?,
            ])
        })
        .transpose()?;
    let mut bytes = facts
        .retained_local_storage_bytes()
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    if let Some(leaf) = facts.retained_leaf() {
        admission.work(
            u64::try_from(leaf.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        for fact in leaf {
            bytes = bytes
                .checked_add(fact_payload_bytes(fact, admission)?)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        }
    }
    admission.bytes(bytes)?;
    let capacity = retention::reserve(resources, bytes, admission)?;
    let cached = existing.is_none();
    let ticket = facts
        .retention_storage()
        .retain(capacity, storage, cached, children);
    if cached {
        slot.allocation = Arc::downgrade(&ticket);
    }
    Ok(ticket)
}

pub(super) fn fact_payload_bytes(
    fact: &WorthQueryApplicationObservedFact,
    admission: &mut impl IndexAdmission,
) -> Result<u64, CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::retained_fact_size();
    use WorthQueryApplicationObservedFact as Fact;
    match fact {
        Fact::SourcePublication { .. }
        | Fact::SourceEntity { .. }
        | Fact::Entity { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. } => Ok(0),
        Fact::RetiredOutputEntity { read_locator, .. } => charge_usize(read_locator.capacity()),
        Fact::SourceAspectRevision { aspect, .. } | Fact::AspectPresence { aspect, .. } => {
            charge_usize(aspect.owned_allocation_capacity_bytes())
        }
        Fact::SourceFieldRevision { locator, .. }
        | Fact::AbsentField { locator, .. }
        | Fact::OptionalAspectAbsent { locator, .. } => {
            admit_locator(locator, admission)?;
            charge_usize(locator.owned_allocation_capacity_bytes())
        }
        Fact::SourceAdjacencyRevision { endpoints, .. } => {
            let bytes = endpoints
                .len()
                .checked_mul(size_of::<worth_relational::facade::identity::EntityId>())
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            charge_usize(bytes)
        }
        Fact::Field { locator, value, .. } => {
            admit_locator(locator, admission)?;
            checked_add(
                charge_usize(locator.owned_allocation_capacity_bytes())?,
                charge_usize(value.owned_allocation_capacity_bytes())?,
            )
        }
        Fact::Relation {
            matching_relations, ..
        } => vec_bytes(matching_relations),
        Fact::Adjacency { relations, .. } => vec_bytes(relations),
        Fact::IndexedEntitySelection {
            definition,
            locator,
            value,
            candidates,
            ..
        } => {
            admit_locator(locator, admission)?;
            let definition_bytes = index_definition_bytes(definition, admission)?;
            checked_add(
                checked_add(
                    checked_add(
                        definition_bytes,
                        charge_usize(locator.owned_allocation_capacity_bytes())?,
                    )?,
                    charge_usize(value.owned_allocation_capacity_bytes())?,
                )?,
                vec_bytes(candidates)?,
            )
        }
        Fact::WorkflowInstanceCapacity { instances, .. } => vec_bytes(instances),
        Fact::WorkflowHistoryBasis { snapshot, .. } => {
            charge_usize(snapshot.branch_id().0.capacity())
        }
    }
}

fn index_definition_bytes(
    definition: &DerivedIndexDefinition,
    admission: &mut impl IndexAdmission,
) -> Result<u64, CompanionPreflightStop> {
    let mut bytes = checked_add(
        index_capacity::arc_bytes::<DerivedIndexDefinition>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        charge_usize(definition.name.capacity())?,
    )?;
    match &definition.kind {
        DerivedIndexKind::EntityField { field_locator }
        | DerivedIndexKind::RelationField { field_locator } => {
            admit_locator(field_locator, admission)?;
            bytes = checked_add(
                bytes,
                charge_usize(field_locator.owned_allocation_capacity_bytes())?,
            )?;
        }
        DerivedIndexKind::RelatedEntityOrdering { ordering, .. } => {
            admission.work(
                u64::try_from(ordering.len())
                    .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
            bytes = checked_add(bytes, vec_bytes(ordering)?)?;
            for field in ordering {
                admit_locator(field.locator(), admission)?;
                bytes = checked_add(
                    bytes,
                    charge_usize(field.locator().owned_allocation_capacity_bytes())?,
                )?;
            }
        }
        DerivedIndexKind::RelationJoin(_) => {}
    }
    Ok(bytes)
}

fn admit_locator(
    locator: &worth_foundational::facade::AspectFieldLocator,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let fields = u64::try_from(locator.field_path().fields().len())
        .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
    admission.work(fields)
}

fn vec_bytes<T>(values: &Vec<T>) -> Result<u64, CompanionPreflightStop> {
    charge_usize(
        values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )
}

fn charge_usize(bytes: usize) -> Result<u64, CompanionPreflightStop> {
    u64::try_from(bytes).map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)
}

fn checked_add(left: u64, right: u64) -> Result<u64, CompanionPreflightStop> {
    left.checked_add(right)
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)
}

pub(in crate::domain_computation::primary_graph) fn arc_slice_bytes<T>(len: usize) -> Option<u64> {
    let alignment = align_of::<T>().max(align_of::<usize>());
    let header = size_of::<usize>().checked_mul(2)?;
    let offset = header.checked_add(alignment - 1)? / alignment * alignment;
    let bytes = offset.checked_add(size_of::<T>().checked_mul(len)?)?;
    u64::try_from(bytes.checked_add(alignment - 1)? / alignment * alignment).ok()
}

#[cfg(test)]
impl RetainedFactCapacity {
    pub(super) fn bytes(&self) -> u64 {
        self._allocation
            .bytes()
            .checked_add(self._basis.bytes())
            .expect("same live retained ledger")
    }
}
