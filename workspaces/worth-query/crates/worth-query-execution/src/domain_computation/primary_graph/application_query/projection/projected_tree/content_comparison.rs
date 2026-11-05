//! Exact Source support equality; disclosed-row equality has different meaning.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_query::content_comparison::ProjectionContentComparisonFault;

impl WorthQueryApplicationDisclosedProjectionTree {
    pub fn same_one_hop_content<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
        fault: &mut impl FnMut(ProjectionContentComparisonFault) -> E,
        frames: &mut dyn FnMut(
            &worth_relational::facade::content::RelationalContentFrame,
            &worth_relational::facade::content::RelationalContentFrame,
        ) -> Result<bool, E>,
    ) -> Result<bool, E> {
        let left = self.raw_rows();
        let right = other.raw_rows();
        admit(1)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (left, right) in left.iter().zip(right) {
            if !same_row(left, right, admit, fault, frames)?
                || left.relations.len() != right.relations.len()
            {
                return Ok(false);
            }
            for (left, right) in left.relations.iter().zip(&right.relations) {
                if !same_relation(left, right, admit, fault, frames)? {
                    return Ok(false);
                }
            }
        }
        admit(0)?;
        Ok(true)
    }
}

fn same_relation_header<E>(
    left: &WorthQueryApplicationProjectedRelation,
    right: &WorthQueryApplicationProjectedRelation,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<bool, E> {
    admit(1)?;
    if !Arc::ptr_eq(&left.slot_key, &right.slot_key)
        || left.cardinality != right.cardinality
        || left.predicate_sources.len() != right.predicate_sources.len()
        || left.rows.len() != right.rows.len()
        || !same_text(&left.result_path, &right.result_path, admit)?
        || !same_text(&left.slot_type, &right.slot_type, admit)?
    {
        return Ok(false);
    }
    for (left, right) in left.predicate_sources.iter().zip(&right.predicate_sources) {
        admit(32)?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

fn same_relation<E>(
    left: &WorthQueryApplicationProjectedRelation,
    right: &WorthQueryApplicationProjectedRelation,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    fault: &mut impl FnMut(ProjectionContentComparisonFault) -> E,
    frames: &mut dyn FnMut(
        &worth_relational::facade::content::RelationalContentFrame,
        &worth_relational::facade::content::RelationalContentFrame,
    ) -> Result<bool, E>,
) -> Result<bool, E> {
    if !same_relation_header(left, right, admit)? {
        return Ok(false);
    }
    for (left, right) in left.rows.iter().zip(&right.rows) {
        admit(1)?;
        // The installed qualification is one hop. Never omit a future depth.
        if !left.relations.is_empty()
            || !right.relations.is_empty()
            || !same_row(left, right, admit, fault, frames)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn same_row<E>(
    left: &WorthQueryApplicationProjectionNode,
    right: &WorthQueryApplicationProjectionNode,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    fault: &mut impl FnMut(ProjectionContentComparisonFault) -> E,
    frames: &mut dyn FnMut(
        &worth_relational::facade::content::RelationalContentFrame,
        &worth_relational::facade::content::RelationalContentFrame,
    ) -> Result<bool, E>,
) -> Result<bool, E> {
    admit(65)?;
    if left.entity_id != right.entity_id
        || left.incoming_relation != right.incoming_relation
        || left.source_dependencies_complete != right.source_dependencies_complete
        || left.fields.len() != right.fields.len()
        || !same_text(left.result_path(), right.result_path(), admit)?
    {
        return Ok(false);
    }
    for (left, right) in left.fields.iter().zip(&right.fields) {
        admit(1)?;
        // The enclosing verifier requires the same actual installation. Its
        // immutable slot contract is stronger than an output-name comparison.
        if !Arc::ptr_eq(&left.slot_key, &right.slot_key)
            || !same_text(&left.result_path, &right.result_path, admit)?
            || !same_text(&left.slot_type, &right.slot_type, admit)?
        {
            return Ok(false);
        }
        if !same_value(&left.value, &right.value, admit)? {
            return Ok(false);
        }
        match (&left.value, &left.content, &right.content) {
            (AspectValue::ContentRef(identity), Some(a), Some(b)) => {
                if a.identity() != *identity || b.identity() != *identity {
                    return Err(fault(
                        ProjectionContentComparisonFault::ContentInconsistency,
                    ));
                }
                if !frames(a, b)? {
                    return Ok(false);
                }
            }
            (AspectValue::ContentRef(_), _, _) => {
                return Err(fault(
                    ProjectionContentComparisonFault::ContentInconsistency,
                ))
            }
            (_, None, None) => {}
            _ => {
                return Err(fault(
                    ProjectionContentComparisonFault::ContentInconsistency,
                ))
            }
        }
    }
    Ok(true)
}

pub fn same_value<E>(
    left: &AspectValue,
    right: &AspectValue,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<bool, E> {
    admit_pair_width(
        left.semantic_byte_width(),
        right.semantic_byte_width(),
        admit,
    )?;
    Ok(left == right)
}

fn same_text<E>(
    left: &str,
    right: &str,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<bool, E> {
    admit_pair_width(left.len(), right.len(), admit)?;
    Ok(left == right)
}

pub fn admit_pair_width<E>(
    left: usize,
    right: usize,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
) -> Result<(), E> {
    // Each side is paid separately. Any accumulated-work overflow belongs to
    // the caller's original checkpoint, including the bool-erased reuse path.
    admit(left as u64)?;
    admit(right as u64)
}

mod traversal;
