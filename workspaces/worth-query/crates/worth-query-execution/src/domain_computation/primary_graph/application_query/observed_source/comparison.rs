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
                if left.identity() != right.identity()
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
