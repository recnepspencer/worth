use std::mem::{align_of, size_of};
use std::sync::Arc;

use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexKind};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;

use super::admission::IndexAdmission;
use super::{index_capacity, retention};

/// A historical mark can outlive the application lineage that first owned
/// these facts. Reserve their retained custody before making its Arc row.
pub(super) fn reserve(
    facts: &[WorthQueryApplicationObservedFact],
    read_basis: &PositionedRelationalSnapshot,
    resources: &WorthQueryInvalidationResources,
    admission: &mut impl IndexAdmission,
) -> Result<Arc<RetainedInvalidationCapacity>, CompanionPreflightStop> {
    let count =
        u64::try_from(facts.len()).map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
    admission.work(
        count
            .checked_add(1)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
    )?;

    let mut bytes = arc_slice_bytes::<WorthQueryApplicationObservedFact>(facts.len())
        .and_then(|base| {
            base.checked_add(index_capacity::arc_bytes::<PositionedRelationalSnapshot>()?)
        })
        .and_then(|base| base.checked_add(u64::try_from(read_basis.branch_id().0.capacity()).ok()?))
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    for fact in facts {
        let payload = fact_payload_bytes(fact, admission)?;
        bytes = bytes
            .checked_add(payload)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    }
    admission.bytes(bytes)?;
    retention::reserve(resources, bytes, admission)
}

pub(super) fn fact_payload_bytes(
    fact: &WorthQueryApplicationObservedFact,
    admission: &mut impl IndexAdmission,
) -> Result<u64, CompanionPreflightStop> {
    use WorthQueryApplicationObservedFact as Fact;
    match fact {
        Fact::SourceEntity { .. }
        | Fact::Entity { .. }
        | Fact::WorkflowDefinitionPredecessor { .. }
        | Fact::WorkflowDefinitionCurrent { .. } => Ok(0),
        Fact::SourceAspectRevision { aspect, .. } => {
            charge_usize(aspect.owned_allocation_capacity_bytes())
        }
        Fact::SourceFieldRevision { locator, .. } | Fact::AbsentField { locator, .. } => {
            admit_locator(locator, admission)?;
            charge_usize(locator.owned_allocation_capacity_bytes())
        }
        Fact::SourceAdjacencyRevision { endpoints, .. } => vec_bytes(endpoints),
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

pub(in crate::domain_computation::primary_graph::output_lineage) fn arc_slice_bytes<T>(
    len: usize,
) -> Option<u64> {
    let alignment = align_of::<T>().max(align_of::<usize>());
    let header = size_of::<usize>().checked_mul(2)?;
    let offset = header.checked_add(alignment - 1)? / alignment * alignment;
    let bytes = offset.checked_add(size_of::<T>().checked_mul(len)?)?;
    u64::try_from(bytes.checked_add(alignment - 1)? / alignment * alignment).ok()
}
