use std::mem::size_of;

use worth_execution::{ExecutionResourceLease, MapKernelFailure};
use worth_foundational::facade::{AspectBinding, AspectShape, CanonicalFieldPath};

use crate::authority::mutation::{
    changed_fields, AdjacencyDelta, AdjacencyDeltaKind, CanonicalRecordAspectDelta,
    EvaluatedAspectBinding,
};
use crate::execution::{PacketBudgetDenial, PacketPreparationBudget};
use crate::history::data::{
    RelationalDescriptiveTouch as Touch, RelationalDescriptiveTouchGraph as Graph,
    RelationalTouchAdjacencyDirection as Direction,
};
use crate::indexes::data::DerivedIndexKind;
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;
use crate::storage::substrate::{partition_of, slot_of, EntityRecordKind, RelationRecordKind};
use crate::transactions::data::{RecordRef, TransactionCommitError};

type Failure = MapKernelFailure<PacketBudgetDenial>;

pub(super) enum TouchPreparationMode {
    Ordinary,
    NativeMerge,
    Unavailable,
}

enum TouchBudget<'a, 'b, 'c, 'd> {
    Checked(&'a mut PacketPreparationBudget<'b, 'c, 'd>),
    Legacy,
}

impl TouchBudget<'_, '_, '_, '_> {
    fn checkpoint(&mut self, units: u64) -> Result<(), Failure> {
        match self {
            Self::Checked(budget) => budget.checkpoint(units),
            Self::Legacy => Ok(()),
        }
    }

    fn claim(&mut self, bytes: u64) -> Result<(), Failure> {
        match self {
            Self::Checked(budget) => budget.claim(bytes),
            Self::Legacy => Ok(()),
        }
    }

    fn claimed_bytes(&self) -> u64 {
        match self {
            Self::Checked(budget) => budget.claimed_bytes(),
            Self::Legacy => 0,
        }
    }
}

pub(super) fn prepare(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    deltas: &[CanonicalRecordAspectDelta],
    adjacency: &[AdjacencyDelta],
    mode: TouchPreparationMode,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Graph, TransactionCommitError> {
    // A descriptor transition or unmaterialized branch reference cannot
    // establish exact native record revisions from this selected working root.
    if matches!(mode, TouchPreparationMode::Unavailable) {
        return Ok(Graph::unavailable());
    }
    match lease {
        Some(lease) => crate::execution::prepare_borrowed_artifact(
            lease,
            runtime.commit_work_budget.as_ref(),
            |checked| {
                build(
                    runtime,
                    selected,
                    working,
                    deltas,
                    adjacency,
                    &mode,
                    &mut TouchBudget::Checked(checked),
                )
            },
        )
        .map_err(|stop| TransactionCommitError::execution(stop.into())),
        None => build(
            runtime,
            selected,
            working,
            deltas,
            adjacency,
            &mode,
            &mut TouchBudget::Legacy,
        )
        .map(|(graph, _)| graph)
        .map_err(|_| {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::ResourceExhausted,
                partition_identity: None,
            })
        }),
    }
}

fn build(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    deltas: &[CanonicalRecordAspectDelta],
    adjacency: &[AdjacencyDelta],
    mode: &TouchPreparationMode,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<(Graph, u64), Failure> {
    let maximum = maximum_touch_count(runtime, deltas, adjacency, budget)?;
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
        append_delta(runtime, selected, working, delta, &mut touches, budget)?;
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
    if matches!(mode, TouchPreparationMode::NativeMerge)
        && !merge::append(runtime, selected, working, adjacency, &mut touches, budget)?
    {
        return Ok((Graph::unavailable(), budget.claimed_bytes()));
    }
    let comparisons = (touches.len() as u64)
        .checked_mul((usize::BITS - touches.len().max(1).leading_zeros()) as u64)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    budget.checkpoint(comparisons)?;
    let retained = budget.claimed_bytes();
    Ok((Graph::exact(touches), retained))
}

fn maximum_touch_count(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    deltas: &[CanonicalRecordAspectDelta],
    adjacency: &[AdjacencyDelta],
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<usize, Failure> {
    let mut total = adjacency
        .len()
        .checked_mul(3)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    for delta in deltas {
        budget.checkpoint(1)?;
        total = total
            .checked_add(1 + delta.changed_aspects.len())
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        for binding in &delta.evaluated_bindings {
            budget.checkpoint(1)?;
            if !binding.changed {
                continue;
            }
            visit_possible_fields(binding, |field| {
                budget.checkpoint(1)?;
                let definitions =
                    runtime
                        .index_definitions
                        .with_field(&binding.aspect_key, field, |defs| defs.len());
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

fn visit_possible_fields(
    binding: &EvaluatedAspectBinding,
    mut visit: impl FnMut(&worth_foundational::facade::FieldKey) -> Result<(), Failure>,
) -> Result<(), Failure> {
    match &binding.aspect_shape {
        AspectShape::Struct(shape) => {
            for field in shape.fields() {
                visit(field.key())?;
            }
        }
        AspectShape::Scalar(_) => match &binding.binding {
            AspectBinding::EntityField { field } | AspectBinding::RelationField { field } => {
                visit(field)?;
            }
            _ => {}
        },
        _ => {}
    }
    Ok(())
}

fn append_delta(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &crate::runtime::WorkingState,
    delta: &CanonicalRecordAspectDelta,
    touches: &mut Vec<Touch>,
    budget: &mut TouchBudget<'_, '_, '_, '_>,
) -> Result<(), Failure> {
    if !matches!(
        delta.structural_change,
        crate::publication::patch::data::RecordStructuralChange::Updated
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
        let mut possible_count = 0usize;
        let mut possible_heap = 0u64;
        visit_possible_fields(binding, |field| {
            budget.checkpoint(1)?;
            possible_count += 1;
            possible_heap =
                possible_heap.saturating_add(field.owned_allocation_capacity_bytes() as u64);
            Ok(())
        })?;
        let scratch = (possible_count as u64)
            .checked_mul(
                (size_of::<(
                    worth_foundational::facade::FieldKey,
                    crate::storage::data::RelationalFieldPresence,
                )>() * 2) as u64,
            )
            .and_then(|slots| slots.checked_add(possible_heap))
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        budget.claim(scratch)?;
        for (field, presence) in changed_fields(binding, delta.structural_change) {
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
            runtime
                .index_definitions
                .with_field(&binding.aspect_key, &field, |definitions| {
                    for definition in definitions {
                        append_index(
                            runtime, selected, working, delta, binding, &field, definition,
                            touches, budget,
                        )?;
                    }
                    Ok::<(), Failure>(())
                })?;
        }
    }
    Ok(())
}

mod index;
mod merge;
use index::append_index;
