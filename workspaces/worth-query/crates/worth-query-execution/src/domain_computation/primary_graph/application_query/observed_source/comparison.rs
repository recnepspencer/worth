//! Compare row/set source bodies without repeating a shared walk per row.
use super::*;
use crate::domain_computation::primary_graph::{
    source_read_equivalence::SourceReadEquivalence, structural_walk::StructuralWalkObservation,
};
use std::sync::Arc;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation::primary_graph::application_query) fn equal(
    left: &WorthQueryObservedSourceFootprint,
    right: &WorthQueryObservedSourceFootprint,
    walks: &mut SourceReadEquivalence,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &dyn Fn() -> Result<(), Cause>,
) -> Result<bool, Cause> {
    compare(left, right, None, walks, execution, control)
}

/// Semantic interning may retain an older equivalent association. Only the
/// actual admitted invocation can identify the joined publication on the right.
pub(super) fn equal_published_source(
    left: &WorthQueryObservedSourceFootprint,
    right: &WorthQueryObservedSourceFootprint,
    current_source: &crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
    walks: &mut SourceReadEquivalence,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &dyn Fn() -> Result<(), Cause>,
) -> Result<bool, Cause> {
    compare(left, right, Some(current_source), walks, execution, control)
}

fn compare(
    left: &WorthQueryObservedSourceFootprint,
    right: &WorthQueryObservedSourceFootprint,
    current_source: Option<
        &crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
    >,
    walks: &mut SourceReadEquivalence,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &dyn Fn() -> Result<(), Cause>,
) -> Result<bool, Cause> {
    let result = execution.run_native_with_checkpoint(|context| {
        StructuralWalkObservation::checkpoint(context)?;
        control()?;
        if left.root != right.root
            || left.complete != right.complete
            || left.entities != right.entities
            || left.aspects != right.aspects
            || left.adjacencies != right.adjacencies
        {
            return Ok(false);
        }
        match (&left.root_selection, &right.root_selection) {
            (None, None) => Ok(true),
            (Some(left), Some(right)) => {
                if Arc::ptr_eq(left, right) {
                    return Ok(true);
                }
                // The applied acquisition descriptor cannot become a positioned
                // descriptor before publication. A genuine joined whole Source
                // can establish their correspondence; all physical and other
                // selection facts below still compare independently.
                let published_source = match (&left.managed_index, &right.managed_index) {
                    (Some(a), Some(b)) => {
                        a.same_published_source(b, current_source, &mut |units| {
                            if context.has_accounting() {
                                context.checkpoint(units)
                            } else {
                                context.poll()
                            }
                            .map_err(Cause::from)
                        })?
                    }
                    _ => false,
                };
                if (left.identity() != right.identity() && !published_source)
                    || left.entities != right.entities
                    || left.aspects != right.aspects
                    || left.adjacencies != right.adjacencies
                {
                    return Ok(false);
                }
                match (&left.orbit, &right.orbit) {
                    (None, None) => {}
                    (Some(left), Some(right)) => {
                        if left.meaning != right.meaning
                            || left.binding != right.binding
                            || !walks.equal_aggregate(
                                &left.aggregate,
                                &right.aggregate,
                                context,
                                control,
                            )?
                        {
                            return Ok(false);
                        }
                    }
                    _ => return Ok(false),
                }
                match (&left.components, &right.components) {
                    (None, None) => {}
                    (Some(a), Some(b)) if a.meaning == b.meaning && a.binding == b.binding => {
                        if !walks.equal_component(&a.body, &b.body, context, control)? {
                            return Ok(false);
                        }
                    }
                    _ => return Ok(false),
                }
                match (&left.managed_index, &right.managed_index) {
                    (None, None) => {}
                    (Some(a), Some(b)) => {
                        if !a.same_truth(
                            b,
                            current_source,
                            walks,
                            execution,
                            control,
                            &mut |units| {
                                if context.has_accounting() {
                                    context.checkpoint(units)
                                } else {
                                    context.poll()
                                }
                                .map_err(Into::into)
                            },
                        )? {
                            return Ok(false);
                        }
                    }
                    _ => return Ok(false),
                }
                let a = left.derived.as_ref().map_or(&[][..], |d| d.values());
                let b = right.derived.as_ref().map_or(&[][..], |d| d.values());
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (a, b) in a.iter().zip(b) {
                    if !walks.equal_derived(a, b, execution, context, control)? {
                        return Ok(false);
                    }
                }
                let left = left.walks.as_ref().map_or(&[][..], |w| w.values());
                let right = right.walks.as_ref().map_or(&[][..], |w| w.values());
                if left.len() != right.len() {
                    return Ok(false);
                }
                for (left, right) in left.iter().zip(right) {
                    if !walks.equal_walk(left, right, execution, context, control)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    });
    let result = match result {
        Ok((value, _)) => value,
        Err(stop) => Err(stop.into()),
    };
    control()?;
    result
}
