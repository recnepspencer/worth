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

pub(super) fn order_collection(
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    collection_path: &str,
    rows: &mut [WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let ordering = contract
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
        .collect::<Result<Vec<_>, _>>()?;
    if ordering.is_empty() || rows.len() < 2 {
        return Ok(());
    }
    heap_sort(rows, &ordering, work)
}

struct AdmittedOrdering<'a> {
    term: &'a WorthQueryInstalledGraphOrdering,
    projection: crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationInternalProjectionAdmission<'a>,
}

fn heap_sort(
    rows: &mut [WorthQueryApplicationProjectionNode],
    ordering: &[AdmittedOrdering<'_>],
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    for root in (0..rows.len() / 2).rev() {
        sift_down(rows, root, rows.len(), ordering, work)?;
    }
    for end in (1..rows.len()).rev() {
        rows.swap(0, end);
        sift_down(rows, 0, end, ordering, work)?;
    }
    Ok(())
}

fn sift_down(
    rows: &mut [WorthQueryApplicationProjectionNode],
    mut root: usize,
    end: usize,
    ordering: &[AdmittedOrdering<'_>],
    work: &mut ResultTreeWork,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    loop {
        let left = root.saturating_mul(2).saturating_add(1);
        if left >= end {
            return Ok(());
        }
        let right = left + 1;
        let greater = if right < end
            && compare_rows(&rows[left], &rows[right], ordering, work)? == Ordering::Less
        {
            right
        } else {
            left
        };
        if compare_rows(&rows[root], &rows[greater], ordering, work)? != Ordering::Less {
            return Ok(());
        }
        rows.swap(root, greater);
        root = greater;
    }
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
