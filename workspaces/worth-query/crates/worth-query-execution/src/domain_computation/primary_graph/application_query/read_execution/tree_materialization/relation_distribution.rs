use worth_query_installation::facade::{
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};

use super::{
    allocate_claimed_result_vector, order_collection, projection_denial, ResultTreeWork,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::application_query::{
    projection::WorthQueryApplicationProjectedRelation,
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn distribute_relation_rows(
    parents: &mut [WorthQueryApplicationProjectionNode],
    relation: &WorthQueryInstalledGraphRelation,
    counts: Vec<usize>,
    children: Vec<WorthQueryApplicationProjectionNode>,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    work: &mut ResultTreeWork,
    already_ordered: bool,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
    predicate_counts: Vec<usize>,
    predicate_sources: Vec<worth_relational::facade::identity::EntityId>,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    let temporary_child_buffer_bytes = children
        .capacity()
        .saturating_mul(std::mem::size_of::<WorthQueryApplicationProjectionNode>());
    let mut children = children.into_iter();
    let mut predicate_sources = predicate_sources.into_iter();
    for ((parent, count), predicate_count) in parents.iter_mut().zip(counts).zip(predicate_counts) {
        let mut rows = allocate_claimed_result_vector::<WorthQueryApplicationProjectionNode>(
            result_buffer,
            count,
            relation.result_path(),
        )?;
        rows.extend(children.by_ref().take(count));
        let mut relation_predicate_sources =
            allocate_claimed_result_vector(result_buffer, predicate_count, relation.result_path())?;
        relation_predicate_sources.extend(predicate_sources.by_ref().take(predicate_count));
        if !already_ordered {
            order_collection(
                contract,
                governance,
                relation.result_path(),
                &mut rows,
                work,
            )?;
        }
        if rows.len() != count
            || relation_predicate_sources.len() != predicate_count
            || !parent.insert_relation(WorthQueryApplicationProjectedRelation::new(
                relation,
                relation_predicate_sources,
                rows,
            ))
        {
            return Err(projection_denial(relation.result_path()));
        }
    }
    if children.next().is_some() {
        return Err(projection_denial(relation.result_path()));
    }
    if predicate_sources.next().is_some() {
        return Err(projection_denial(relation.result_path()));
    }
    drop(children);
    result_buffer.release_temporary(temporary_child_buffer_bytes);
    Ok(())
}
