//! Full retained descriptor DATA; no installed program, dispatch authority or cache.
use super::super::{
    plan::{CompiledWorkflowConnectionKind, CompiledWorkflowNode},
    publication_binding::ColdCompiledWorkflowDefinition,
};
use super::{
    cold, observed_u64,
    publication_header::{self, recorded},
};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationAttemptDenial,
    workflow::schema::WorthQueryWorkflowLayout,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDataFlow, ApplicationWorkflowDefinitionContentIdentity,
};
use worth_relational::facade::{
    identity::EntityId, runtime::RelationalRuntime, snapshots::SnapshotHandle,
};

pub(in crate::domain_computation::primary_graph) struct RecordedWorkflowDefinition {
    header: recorded::RecordedWorkflowPublication,
    members: ColdCompiledWorkflowDefinition,
    nodes: BTreeMap<EntityId, usize>,
    captured: BTreeMap<EntityId, Vec<EntityId>>,
}
impl RecordedWorkflowDefinition {
    pub(in crate::domain_computation::primary_graph) fn content(
        &self,
    ) -> &ApplicationWorkflowDefinitionContentIdentity {
        &self.header.content
    }
    pub(in crate::domain_computation::primary_graph) const fn lineage(&self) -> EntityId {
        self.header.lineage
    }
    pub(in crate::domain_computation::primary_graph) const fn published_under(
        &self,
    ) -> CanonicalDigestId {
        self.header.published_under
    }
    pub(in crate::domain_computation::primary_graph) fn effective_revision(
        &self,
    ) -> CanonicalDigestId {
        self.header
            .carried_to
            .unwrap_or(self.header.published_under)
    }
    pub(in crate::domain_computation::primary_graph) fn node(
        &self,
        id: EntityId,
    ) -> Option<&CompiledWorkflowNode> {
        self.nodes
            .get(&id)
            .map(|ordinal| &self.members.nodes[*ordinal])
    }
    pub(in crate::domain_computation::primary_graph) fn nodes(&self) -> &[CompiledWorkflowNode] {
        &self.members.nodes
    }
    pub(in crate::domain_computation::primary_graph) fn captured_targets(
        &self,
        source: EntityId,
    ) -> impl Iterator<Item = &CompiledWorkflowNode> {
        self.captured.get(&source).into_iter().flatten().map(|id| {
            self.node(*id)
                .expect("cold descriptor admitted each endpoint")
        })
    }
}

pub(in crate::domain_computation::primary_graph) fn read_recorded_definition(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
) -> Result<RecordedWorkflowDefinition, WorthQueryApplicationAttemptDenial> {
    let mut header = recorded::observe(execution, native, snapshot, layout, definition)?;
    let mut facts = Vec::new();
    let mut limit = |field| {
        let value = observed_u64(
            execution,
            native,
            snapshot,
            definition,
            layout.definition.entity_kind,
            field,
            &mut facts,
        )?;
        usize::try_from(value)
            .map_err(|_| super::denial("retained definition size exceeds this host"))
    };
    let maximum_nodes = limit(&layout.definition.maximum_nodes)?;
    let maximum_connections = limit(&layout.definition.maximum_connections)?;
    let observed = publication_header::ObservedWorkflowPublicationHeader {
        definition,
        lineage: header.lineage,
        facts: std::mem::take(&mut header.facts),
    };
    let (members, _) = cold::reconstruct_members(
        execution,
        native,
        snapshot,
        layout,
        observed,
        maximum_nodes,
        maximum_connections,
    )?;
    let nodes = members
        .nodes
        .iter()
        .enumerate()
        .map(|(ordinal, node)| (node.entity, ordinal))
        .collect();
    let mut captured = BTreeMap::<_, Vec<_>>::new();
    for connection in &members.connections {
        if matches!(
            connection.kind(),
            CompiledWorkflowConnectionKind::Data(
                ApplicationWorkflowDataFlow::CapturedOperationInput
            )
        ) {
            captured
                .entry(connection.source())
                .or_default()
                .push(connection.target());
        }
    }
    Ok(RecordedWorkflowDefinition {
        header,
        members,
        nodes,
        captured,
    })
}
