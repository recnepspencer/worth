//! Metered ordered native-cell equality excludes all row/dependency provenance.
use super::super::retained_projection::ProjectionComparisonDenial as Denial;
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use worth_execution::{MapKernelContext, MapKernelFailure, MapKernelStop};

type Failure = MapKernelFailure<Denial>;
type Decisions = [(Arc<ApplicationQueryResultSlotKey>, bool)];

struct Frame<'rows> {
    left: &'rows [WorthQueryApplicationProjectionNode],
    right: &'rows [WorthQueryApplicationProjectionNode],
    row: usize,
    fields_checked: bool,
    left_relation: usize,
    right_relation: usize,
}

impl<'rows> Frame<'rows> {
    fn new(
        left: &'rows [WorthQueryApplicationProjectionNode],
        right: &'rows [WorthQueryApplicationProjectionNode],
    ) -> Self {
        Self {
            left,
            right,
            row: 0,
            fields_checked: false,
            left_relation: 0,
            right_relation: 0,
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_query::projection) fn rows_equal(
    left: &[WorthQueryApplicationProjectionNode],
    right: &[WorthQueryApplicationProjectionNode],
    decisions: &Decisions,
    maximum_depth: usize,
    context: &mut MapKernelContext<'_, '_>,
    mut admit_memory: impl FnMut(u64) -> Result<(), Failure>,
) -> Result<bool, Failure> {
    let layout = std::alloc::Layout::array::<Frame<'_>>(maximum_depth).map_err(|_| overflow())?;
    let bytes = u64::try_from(layout.size()).map_err(|_| overflow())?;
    checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
    admit_memory(bytes)?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(maximum_depth)
        .map_err(|cause| MapKernelFailure::Domain(Denial::Allocation(cause)))?;
    stack.push(Frame::new(left, right));
    while let Some(frame) = stack.last_mut() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if frame.left.len() != frame.right.len() {
            return Ok(false);
        }
        if frame.row == frame.left.len() {
            stack.pop();
            continue;
        }
        let left_rows = frame.left;
        let right_rows = frame.right;
        let left = &left_rows[frame.row];
        let right = &right_rows[frame.row];
        if !frame.fields_checked {
            if !fields_equal(left, right, decisions, context)? {
                return Ok(false);
            }
            frame.fields_checked = true;
        }
        let next_left = next_relation(
            &left.relations,
            &mut frame.left_relation,
            decisions,
            context,
        )?;
        let next_right = next_relation(
            &right.relations,
            &mut frame.right_relation,
            decisions,
            context,
        )?;
        match (next_left, next_right) {
            (None, None) => {
                frame.row += 1;
                frame.fields_checked = false;
                frame.left_relation = 0;
                frame.right_relation = 0;
            }
            (Some(left), Some(right)) => {
                if !Arc::ptr_eq(&left.slot_key, &right.slot_key) {
                    return Ok(false);
                }
                if stack.len() == maximum_depth {
                    return Err(MapKernelFailure::Domain(
                        Denial::NestingExceedsDeclaredShape,
                    ));
                }
                stack.push(Frame::new(&left.rows, &right.rows));
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

fn fields_equal(
    left: &WorthQueryApplicationProjectionNode,
    right: &WorthQueryApplicationProjectionNode,
    decisions: &Decisions,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Failure> {
    let mut left_index = 0;
    let mut right_index = 0;
    loop {
        let left = next_field(&left.fields, &mut left_index, decisions, context)?;
        let right = next_field(&right.fields, &mut right_index, decisions, context)?;
        match (left, right) {
            (None, None) => return Ok(true),
            (Some(left), Some(right)) => {
                if !Arc::ptr_eq(&left.slot_key, &right.slot_key) {
                    return Ok(false);
                }
                let work = left
                    .value
                    .semantic_byte_width()
                    .checked_add(right.value.semantic_byte_width())
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or_else(overflow)?;
                checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
                if left.value != right.value {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }
}

fn next_field<'a>(
    fields: &'a [WorthQueryApplicationProjectedField],
    index: &mut usize,
    decisions: &Decisions,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<&'a WorthQueryApplicationProjectedField>, Failure> {
    while let Some(field) = fields.get(*index) {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        *index += 1;
        if disclosed(&field.slot_key, decisions, context)? {
            return Ok(Some(field));
        }
    }
    Ok(None)
}

fn next_relation<'a>(
    relations: &'a [WorthQueryApplicationProjectedRelation],
    index: &mut usize,
    decisions: &Decisions,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<&'a WorthQueryApplicationProjectedRelation>, Failure> {
    while let Some(relation) = relations.get(*index) {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        *index += 1;
        if disclosed(&relation.slot_key, decisions, context)? {
            return Ok(Some(relation));
        }
    }
    Ok(None)
}

fn disclosed(
    slot: &Arc<ApplicationQueryResultSlotKey>,
    decisions: &Decisions,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Failure> {
    for (expected, disclosed) in decisions {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if Arc::ptr_eq(slot, expected) {
            return Ok(*disclosed);
        }
    }
    Err(MapKernelFailure::Domain(
        Denial::DisclosureInterpretationChanged,
    ))
}

fn overflow() -> Failure {
    MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow)
}
