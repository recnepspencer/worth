//! The admitted subject's relation-backed birth in the selected lineage.
//!
//! A birth creates its subject in the same commit, so it can record the
//! subject only as `instance_subject_relation` (birth protocol, no coordinate
//! fields) and the coordinate-ordered definition index never holds it. A
//! candidate carries at most one birth and its subject is that candidate's own
//! creation, so a subject has at most one birth relation. This bounded native
//! adjacency read at the page's own snapshot is therefore the complete birth
//! source: no index, scan or reconstruction is consulted.
use super::*;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

/// The born instance of `subject` when its lineage is `lineage`. A birth into
/// another lineage of the same subject is not part of this page.
pub(super) fn born_instance(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    subject: EntityId,
    lineage: EntityId,
) -> Result<Option<EntityId>, Denial> {
    let births = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.instance_subject_relation,
        subject,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        2,
    )
    .map_err(Denial::Execution)?
    .ok_or(Denial::BasisUnavailable)?;
    let instance = match births.as_slice() {
        [] => return Ok(None),
        [birth] => birth.from,
        _ => return Err(Denial::Corrupt),
    };
    let owners = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.instance_lineage_relation,
        instance,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
    )
    .map_err(Denial::Execution)?
    .ok_or(Denial::BasisUnavailable)?;
    match owners.as_slice() {
        [owner] => Ok((owner.to == lineage).then_some(instance)),
        _ => Err(Denial::Corrupt),
    }
}
