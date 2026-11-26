use std::cmp::Ordering;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryObservableInfluence, ApplicationQueryOrderingDirection,
};
use worth_query_installation::facade::{
    WorthQueryInstalledGraphOrdering, WorthQueryInstalledGraphReadContract,
};

use super::{
    projection_denial, ResultTreeWork, WorthQueryApplicationProjectionNode,
    WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::application_query::root_window::WorthQueryApplicationRootWindowBoundary;

pub fn order_collection(
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    collection_path: &str,
    rows: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let ordering = admitted_ordering(contract, governance, collection_path)?;
    if ordering.is_empty() || rows.len() < 2 {
        return Ok(());
    }
    heap_sort(rows, &ordering, work)
}

/// The collection's ordering terms, each admitted for ordering influence.
pub(super) fn admitted_ordering<'a>(
    contract: &'a WorthQueryInstalledGraphReadContract,
    governance: &'a crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    collection_path: &str,
) -> Result<Vec<AdmittedOrdering<'a>>, WorthQueryApplicationReadExecutionDenial> {
    contract
        .ordering()
        .iter()
        .filter(|term| term.collection_path() == collection_path)
        .map(|term| {
            governance
                .admit_internal_projection(
                    term.field(),
                    term.field_key(),
                    ApplicationQueryObservableInfluence::Ordering,
                )
                .map(|projection| AdmittedOrdering { term, projection })
                .ok_or_else(|| projection_denial(term.result_path()))
        })
        .collect()
}

pub(super) struct AdmittedOrdering<'a> {
    term: &'a WorthQueryInstalledGraphOrdering,
    projection: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationInternalProjectionAdmission<'a>,
}

fn heap_sort(
    rows: &mut [WorthQueryApplicationProjectionNode],
    ordering: &[AdmittedOrdering<'_>],
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    super::fallible_heap::sort_by(rows, |left, right| {
        compare_rows(left, right, ordering, work)
    })
}

fn compare_rows(
    left: &WorthQueryApplicationProjectionNode,
    right: &WorthQueryApplicationProjectionNode,
    ordering: &[AdmittedOrdering<'_>],
    work: &mut ResultTreeWork,
) -> Result<Ordering, WorthQueryApplicationReadExecutionDenial> {
    for admitted in ordering {
        let term = admitted.term;
        if admitted.projection.field_key() != term.field_key() {
            return Err(projection_denial(term.result_path()));
        }
        work.charge_ordering_comparison(term.result_path())?;
        let left_value = left
            .field(term.slot_type())
            .ok_or_else(|| projection_denial(term.result_path()))?
            .value();
        let right_value = right
            .field(term.slot_type())
            .ok_or_else(|| projection_denial(term.result_path()))?
            .value();
        let comparison = match term.direction() {
            ApplicationQueryOrderingDirection::Ascending => left_value.cmp(right_value),
            ApplicationQueryOrderingDirection::Descending => right_value.cmp(left_value),
        };
        if comparison != Ordering::Equal {
            return Ok(comparison);
        }
    }
    work.charge_ordering_comparison("result-identity-tie-break")?;
    Ok(left.entity_id().cmp(&right.entity_id()))
}

/// Whether `row` lies strictly after `boundary` in the same total order as
/// [`compare_rows`]: the admitted terms, then root identity.
pub(super) fn follows_boundary(
    row: &WorthQueryApplicationProjectionNode,
    boundary: &WorthQueryApplicationRootWindowBoundary,
    ordering: &[AdmittedOrdering<'_>],
    work: &mut ResultTreeWork,
) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
    for admitted in ordering {
        let term = admitted.term;
        work.charge_ordering_comparison(term.result_path())?;
        let row_value = row
            .field(term.slot_type())
            .ok_or_else(|| projection_denial(term.result_path()))?
            .value();
        let boundary_value = boundary
            .ordering
            .iter()
            .find(|(slot, _)| slot.as_ref() == term.slot_type())
            .map(|(_, value)| value)
            .ok_or_else(|| projection_denial(term.result_path()))?;
        let comparison = match term.direction() {
            ApplicationQueryOrderingDirection::Ascending => row_value.cmp(boundary_value),
            ApplicationQueryOrderingDirection::Descending => boundary_value.cmp(row_value),
        };
        if comparison != Ordering::Equal {
            return Ok(comparison == Ordering::Greater);
        }
    }
    work.charge_ordering_comparison("result-identity-tie-break")?;
    Ok(row.entity_id() > boundary.root)
}

/// The boundary just after `row`: its admitted ordering values and identity.
pub(super) fn boundary_after(
    row: &WorthQueryApplicationProjectionNode,
    ordering: &[AdmittedOrdering<'_>],
) -> Result<WorthQueryApplicationRootWindowBoundary, WorthQueryApplicationReadExecutionDenial> {
    let values = ordering
        .iter()
        .map(|admitted| {
            let slot = admitted.term.slot_type();
            row.field(slot)
                .map(|field| (std::sync::Arc::<str>::from(slot), field.value().clone()))
                .ok_or_else(|| projection_denial(admitted.term.result_path()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(WorthQueryApplicationRootWindowBoundary {
        ordering: values,
        root: row.entity_id(),
    })
}
