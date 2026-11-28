use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::{RelationalDescriptiveTouch as Touch, RelationalDescriptiveTouchGraph as Graph, RelationalDescriptiveTouchGraphOwnerPort as _, RelationalTouchAdjacencyDirection as Direction};
use std::mem::size_of;

use worth_execution::MapKernelFailure;
use worth_foundational::facade::{AspectBinding, AspectShape, CanonicalFieldPath};

use worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::field_changes::visit_changed_fields;
use contracts::authority::mutation::adjacency_delta::AdjacencyDelta;
use contracts::authority::mutation::adjacency_delta::AdjacencyDeltaKind;
use worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::data::{CanonicalRecordAspectDelta, EvaluatedAspectBinding};
use contracts::execution::read_only_packets::PacketBudgetDenial;
use contracts::execution::packet_preparation::PacketPreparationBudget;
use contracts::indexes::data::definition::DerivedIndexKind;
use contracts::storage::data::read_records::RecordLifecycleState;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::{EntityRecordKind, RelationRecordKind, partition_of, slot_of};
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;

type Failure = MapKernelFailure<PacketBudgetDenial>;

pub(super) enum TouchPreparationMode {
    Ordinary,
    Unavailable,
}

pub(super) fn prepare(
    work_budget: Option<&contracts::execution::request_work::RequestWorkBudget>,
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    selected: &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    working: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    deltas: &[CanonicalRecordAspectDelta],
    adjacency: &[AdjacencyDelta],
    mode: TouchPreparationMode,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Graph, TransactionCommitError> {
    // A descriptor transition or unmaterialized branch reference cannot
    // establish exact native record revisions from this selected working root.
    if matches!(mode, TouchPreparationMode::Unavailable) {
        return Ok(Graph::unavailable());
    }
    use contracts::execution::packet_preparation::{prepare_borrowed_artifact, prepare_serial_artifact};
    let measure = |budget: &mut PacketPreparationBudget<'_, '_, '_, '_>| {
        Ok((maximum_touch_count(catalog, deltas, adjacency, budget)?, 0))
    };
    let lease = contracts::execution::request_custody::request_lease(execution);
    let maximum = match lease {
        Some(lease) => prepare_borrowed_artifact(lease, work_budget, 0, measure)
            .map_err(TransactionCommitError::from)?,
        None => prepare_serial_artifact(execution, 0, measure)?,
    };
    let bound = extent::memory_bound(catalog, selected, working, deltas, maximum)?;
    let assemble = |budget: &mut PacketPreparationBudget<'_, '_, '_, '_>| {
        build(catalog, selected, working, deltas, adjacency, maximum, budget)
    };
    match lease {
        Some(lease) => prepare_borrowed_artifact(lease, work_budget, bound, assemble)
            .map_err(TransactionCommitError::from),
        None => prepare_serial_artifact(execution, bound, assemble),
    }
}

fn build(
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    selected: &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    working: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    deltas: &[CanonicalRecordAspectDelta],
    adjacency: &[AdjacencyDelta],
    maximum: usize,
    budget: &mut PacketPreparationBudget<'_, '_, '_, '_>,
) -> Result<(Graph, u64), Failure> {
    let slots = (maximum as u64)
        .checked_mul(size_of::<Touch>() as u64)
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    budget.claim(slots)?;
    let mut touches = Vec::new();
    touches
        .try_reserve_exact(maximum)
        .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
    for delta in deltas {
        append_delta(catalog, selected, working, delta, &mut touches, budget)?;
    }
    for delta in adjacency {
        budget.checkpoint(3)?;
        let (source, target) = match delta.kind {
            AdjacencyDeltaKind::Created { source, target }
            | AdjacencyDeltaKind::Deleted { source, target } => (source, target),
        };
        touches.push(Touch::RelationMembership {
            relation: delta.relation_id,
            kind: delta.kind_id,
            source,
            target,
        });
        touches.push(Touch::AdjacencyRevision {
            kind: delta.kind_id,
            anchor: source,
            direction: Direction::Outgoing,
        });
        touches.push(Touch::AdjacencyRevision {
            kind: delta.kind_id,
            anchor: target,
            direction: Direction::Incoming,
        });
    }
    let comparisons = (touches.len() as u64)
        .checked_mul((usize::BITS - touches.len().max(1).leading_zeros()) as u64)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    budget.checkpoint(comparisons)?;
    let retained = budget.claimed_bytes();
    Ok((Graph::exact(touches), retained))
}

fn maximum_touch_count(
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    deltas: &[CanonicalRecordAspectDelta],
    adjacency: &[AdjacencyDelta],
    budget: &mut PacketPreparationBudget<'_, '_, '_, '_>,
) -> Result<usize, Failure> {
    let mut total = adjacency
        .len()
        .checked_mul(3)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    for delta in deltas {
        budget.checkpoint(1)?;
        total = total
            .checked_add(1 + delta.changed_aspects.len())
            .and_then(|count| {
                count.checked_add(usize::from(delta.retained_relation_endpoints.is_some()))
            })
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        for binding in &delta.evaluated_bindings {
            budget.checkpoint(1)?;
            if !binding.changed {
                continue;
            }
            budget.checkpoint(possible_field_count(binding) as u64)?;
            visit_changed_fields(binding, delta.structural_change, |field, _| {
                let definitions = catalog
                    .catalog
                    .with_field(&binding.aspect_key, field, |defs| {
                        defs.iter()
                            .filter(|definition| {
                                definition.index_id.0 < catalog.next_index_id
                                    && index::targets_field(
                                        binding,
                                        &delta.target,
                                        field,
                                        definition,
                                    )
                            })
                            .count()
                    });
                total = total
                    .checked_add(1)
                    .and_then(|count| count.checked_add(definitions))
                    .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
                Ok::<(), Failure>(())
            })?;
        }
    }
    Ok(total)
}

fn append_delta(
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    selected: &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    working: &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    delta: &CanonicalRecordAspectDelta,
    touches: &mut Vec<Touch>,
    budget: &mut PacketPreparationBudget<'_, '_, '_, '_>,
) -> Result<(), Failure> {
    if !matches!(
        delta.structural_change,
        contracts::publication::patch::data::record_structural_change::RecordStructuralChange::Updated
    ) {
        budget.checkpoint(1)?;
        touches.push(match delta.target {
            RecordRef::Entity(entity) => Touch::EntityLifecycle {
                entity,
                kind: delta.kind_id,
            },
            RecordRef::Relation(relation) => Touch::RelationLifecycle {
                relation,
                kind: delta.kind_id,
            },
        });
    }
    if let (RecordRef::Relation(relation), Some((source, target))) =
        (&delta.target, delta.retained_relation_endpoints)
    {
        budget.checkpoint(1)?;
        touches.push(Touch::RelationMembership {
            relation: *relation,
            kind: delta.kind_id,
            source,
            target,
        });
    }
    for aspect in &delta.changed_aspects {
        budget.checkpoint(1)?;
        budget.claim(aspect.owned_allocation_capacity_bytes() as u64)?;
        touches.push(Touch::AspectRevision {
            record: delta.target.clone(),
            aspect: aspect.clone(),
        });
    }
    for binding in &delta.evaluated_bindings {
        budget.checkpoint(1)?;
        if !binding.changed {
            continue;
        }
        budget.checkpoint(possible_field_count(binding) as u64)?;
        visit_changed_fields(binding, delta.structural_change, |field, presence| {
            budget.checkpoint(1)?;
            let path_bytes = (binding.aspect_key.owned_allocation_capacity_bytes()
                + field.owned_allocation_capacity_bytes()
                + size_of::<worth_foundational::facade::FieldKey>())
                as u64;
            budget.claim(path_bytes)?;
            touches.push(Touch::FieldRevision {
                record: delta.target.clone(),
                kind: delta.kind_id,
                aspect: binding.aspect_key.clone(),
                path: CanonicalFieldPath::single(field.clone()),
                presence,
            });
            catalog
                .catalog
                .with_field(&binding.aspect_key, field, |definitions| {
                    for definition in definitions
                        .iter()
                        .filter(|definition| definition.index_id.0 < catalog.next_index_id)
                    {
                        index::append_index(
                            selected, working, delta, binding, field, definition, touches, budget,
                        )?;
                    }
                    Ok::<(), Failure>(())
                })?;
            Ok::<(), Failure>(())
        })?;
    }
    Ok(())
}

pub mod index;

pub mod extent;

// Field discovery prices the declared schema extent; the retained projection
// borrows only the changed fields and allocates only their emitted paths.
fn possible_field_count(binding: &EvaluatedAspectBinding) -> usize {
    match (&binding.aspect_shape, &binding.binding) {
        (AspectShape::Struct(shape), _) => shape.fields().len(),
        (
            AspectShape::Scalar(_),
            AspectBinding::EntityField { .. } | AspectBinding::RelationField { .. },
        ) => 1,
        _ => 0,
    }
}
