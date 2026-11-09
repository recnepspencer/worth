//! Business-neighbour selectors belong only to removal and linked-owner preflights.
use super::*;
use std::collections::BTreeSet;

pub(crate) fn selectors(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    destroyed: &[RecordRef],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<RecordRef>, TransactionCommitError> {
    let root = &basis.inner.root;
    let Some(storage) = &root
        .schema_authority()
        .registry()
        .current_publication_storage
    else {
        return Ok(Vec::new());
    };
    let view = InvariantStateView::new(
        root.as_ref(),
        crate::identity::data::VersionId(root.axes().map_or(0, |a| a.storage_version)),
    );
    let mut selected = BTreeSet::new();
    for record in destroyed {
        let RecordRef::Entity(id) = record else {
            continue;
        };
        root.demand_head_partition(execution, id.partition_id)
            .map_err(|e| crate::authority::commit::preparation::storage_read_denial(basis, e))?;
        let head = root
            .get_current_partition(id.partition_id)
            .ok_or_else(|| damaged(record))?;
        // Retained adjacency on an already retired source is historical, not
        // a business neighbourhood of a current destruction. Restoration may
        // legitimately select that old source; it has no live input edges.
        if view.entity_metadata(*id).is_none() {
            continue;
        }
        for edge in head
            .outgoing_candidates(*id)
            .iter()
            .chain(head.incoming_candidates(*id).iter())
        {
            checkpoint(context, 1)?;
            root.demand_head_partition(execution, edge.partition_id)
                .map_err(|e| {
                    crate::authority::commit::preparation::storage_read_denial(basis, e)
                })?;
            let partition = root
                .get_current_partition(edge.partition_id)
                .ok_or_else(|| damaged(&RecordRef::Relation(*edge)))?;
            let slot = partition
                .relation_arena()
                .get(edge)
                .ok_or_else(|| damaged(&RecordRef::Relation(*edge)))?;
            // Candidate adjacency also retains retired business links for audit.
            // Only a live relation is a current linked-owner input; missing live
            // metadata still refuses as storage damage below.
            if !slot.is_live() {
                continue;
            }
            let edge = view
                .relation_metadata(*edge)
                .ok_or_else(|| damaged(&RecordRef::Relation(*edge)))?;
            if !storage.metadata_kinds().contains(&edge.kind_id) {
                selected.insert(edge.source);
                selected.insert(edge.target);
            }
        }
    }
    Ok(selected.into_iter().map(RecordRef::Entity).collect())
}
