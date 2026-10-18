//! Complete bounded record revalidation; workflow migration remains unsupported.
use super::denial;
#[path = "selection_storage.rs"]
mod storage;
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_relational::facade::{
    identity::{EntityId, RelationId},
    runtime::VisibilityProjectionView,
};

pub(super) struct RevalidationSelection {
    pub(super) entities: Vec<EntityId>,
    pub(super) relations: Vec<RelationId>,
    // Last to drop: original request backing outlives both selected ID arrays.
    _storage: worth_execution::ExecutionMemoryReservation,
}

pub(super) fn select(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch: &VisibilityProjectionView,
    layout: &WorthQueryPrimaryGraphLayout,
    maximum_work: usize,
    mut work: usize,
) -> Result<RevalidationSelection, WorthQueryPrimaryGraphInstallationDenial> {
    let mut storage = storage::reserve(execution)?;
    let workflow = layout.workflow();
    for kind in [
        workflow.lineage.entity_kind,
        workflow.definition.entity_kind,
        workflow.node.entity_kind,
        workflow.connection.entity_kind,
        workflow.instance.entity_kind,
        workflow.transition.entity_kind,
        workflow.proposal.entity_kind,
        workflow.proposal_coverage.entity_kind,
        workflow.assessment_evidence.entity_kind,
        workflow.approval.entity_kind,
        workflow.evidence_dependency.entity_kind,
    ] {
        let read = branch
            .bounded_entities_of_kind(execution, kind, maximum_work.saturating_sub(work))
            .map_err(super::super::request_read_denial::read)?
            .map_err(|_| denial("checkpoint transition workflow selection exceeded its bound"))?;
        work = work.saturating_add(read.work_units());
        if !read.into_records().is_empty() {
            return Err(denial(
                "checkpoint transition requires workflow migration dispositions",
            ));
        }
    }
    let mut entities = Vec::new();
    for kind in layout.application_entity_kinds() {
        let read = branch
            .bounded_entities_of_kind(execution, kind, maximum_work.saturating_sub(work))
            .map_err(super::super::request_read_denial::read)?
            .map_err(|_| denial("checkpoint transition entity selection exceeded its bound"))?;
        work = work.saturating_add(read.work_units());
        storage::append(
            execution,
            &mut storage,
            &mut entities,
            read.into_records()
                .into_iter()
                .map(|record| record.entity_id),
        )?;
    }
    let mut relations = Vec::new();
    for kind in layout.application_relation_kinds() {
        let read = branch
            .bounded_relations_of_kind(execution, kind, maximum_work.saturating_sub(work))
            .map_err(super::super::request_read_denial::read)?
            .map_err(|_| denial("checkpoint transition relation selection exceeded its bound"))?;
        work = work.saturating_add(read.work_units());
        storage::append(
            execution,
            &mut storage,
            &mut relations,
            read.into_records()
                .into_iter()
                .map(|record| record.relation_id),
        )?;
    }
    Ok(RevalidationSelection {
        entities,
        relations,
        _storage: storage,
    })
}
