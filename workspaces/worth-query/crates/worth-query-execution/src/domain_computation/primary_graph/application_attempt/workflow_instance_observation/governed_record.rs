//! Read-only reconstruction of a located record, never a command eligibility check.
use super::*;
use crate::domain_computation::primary_graph::application_discovery::{
    WorthQueryWorkflowObservedState as State, WorthQueryWorkflowReadDenial as Denial,
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

pub(in crate::domain_computation::primary_graph) struct WorkflowRecord {
    pub(in crate::domain_computation::primary_graph) state: State,
    pub(in crate::domain_computation::primary_graph) instance: EntityId,
    pub(in crate::domain_computation::primary_graph) lineage: EntityId,
    pub(in crate::domain_computation::primary_graph) subject: EntityId,
    pub(in crate::domain_computation::primary_graph) original_start_revision:
        worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    pub(in crate::domain_computation::primary_graph) occurrence: u64,
    pub(in crate::domain_computation::primary_graph) live: bool,
    pub(in crate::domain_computation::primary_graph) history: ReadOnlyWorkflowHistory,
    pub(in crate::domain_computation::primary_graph) dependencies:
        worth_execution::ExecutionArray<EntityId>,
    pub(in crate::domain_computation::primary_graph) reference:
        Option<PublishedWorkflowInstanceRef>,
}
/// Capability admission and full subject/Spec validation occur before this native record read.
pub(in crate::domain_computation::primary_graph) fn observe<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    branch: crate::basis::WorthQueryProductBranch,
    instance: EntityId,
    is_current_observation: bool,
    lineage: EntityId,
    revision: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    support: &crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>,
    resources: WorthQueryApplicationWorkflowResourceCeiling,
) -> Result<WorkflowRecord, Denial> {
    // Historical data is decoded against the admitted roster, NOT re-authorized as active.
    let original_start_revision=crate::domain_computation::primary_graph::workflow::instance::start_provenance::read(
        execution,runtime,snapshot,layout,instance,support).map_err(|cause| {
            use crate::domain_computation::primary_graph::workflow::instance::start_provenance::StartProvenanceReadDenial as Native;
            match cause {Native::Corrupt=>Denial::Corrupt,Native::UnsupportedProtocol(version)=>Denial::UnsupportedInstanceProtocol(version),Native::Execution(cause)=>Denial::Relational(cause)}
        })?;
    let kind = layout.instance.entity_kind;
    let mut facts = Vec::new();
    let state = exact_u64(
        execution,
        runtime,
        snapshot,
        instance,
        kind,
        &layout.instance.state,
        &mut facts,
    )
    .map_err(Denial::Execution)?;
    let state = match WorkflowInstanceState::from_persisted_tag(state).ok_or(Denial::Corrupt)? {
        WorkflowInstanceState::Ready => State::Ready,
        WorkflowInstanceState::Completed => State::Completed,
        WorkflowInstanceState::Cancelled => State::Cancelled,
        WorkflowInstanceState::Migrated => State::Migrated,
        WorkflowInstanceState::Terminated => State::Terminated,
    };
    let occurrence = exact_u64(
        execution,
        runtime,
        snapshot,
        instance,
        kind,
        &layout.instance.branch_occurrence,
        &mut facts,
    )
    .map_err(Denial::Execution)?;
    let memberships = adjacency_with_kind(
        execution,
        runtime,
        snapshot,
        layout.live_instance_lineage_relation,
        instance,
        2,
        "workflow live membership unavailable",
        &mut facts,
    )
    .map_err(Denial::Execution)?;
    let live = match memberships.as_slice() {
        [] => false,
        [target] if *target == lineage => true,
        _ => return Err(Denial::Corrupt),
    };
    if live && state != State::Ready {
        return Err(Denial::Corrupt);
    }
    let reference = if is_current_observation && live && occurrence == branch.occurrence_ordinal() {
        let recorded = observe_field_value(
            runtime,
            execution,
            snapshot,
            instance,
            kind,
            &layout.instance.program_revision,
        )
        .map_err(Denial::Relational)?;
        if recorded != Some(text(revision.to_string())) {
            return Err(Denial::PolicyMismatch);
        }
        let content = observe_field_value(
            runtime,
            execution,
            snapshot,
            instance,
            kind,
            &layout.instance.definition_content_identity,
        )
        .map_err(Denial::Relational)?;
        let Some(AspectValue::String(worth_foundational::facade::InternedString::Raw(content))) =
            content
        else {
            return Err(Denial::Corrupt);
        };
        let content = worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity::from_recorded(&content).ok_or(Denial::Corrupt)?;
        let definitions = adjacency_with_kind(
            execution,
            runtime,
            snapshot,
            layout.instance_definition_relation,
            instance,
            2,
            "workflow definition unavailable",
            &mut facts,
        )
        .map_err(Denial::Execution)?;
        let [definition] = definitions.as_slice() else {
            return Err(Denial::Corrupt);
        };
        let owner = observe_adjacency(
            runtime,
            execution,
            snapshot,
            layout.lineage_definition_relation,
            *definition,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            2,
        )
        .map_err(Denial::Execution)?
        .ok_or(Denial::BasisUnavailable)?;
        let [owner] = owner.as_slice() else {
            return Err(Denial::Corrupt);
        };
        if owner.from != lineage {
            return Err(Denial::Corrupt);
        }
        let recorded_content = observe_field_value(
            runtime,
            execution,
            snapshot,
            *definition,
            layout.definition.entity_kind,
            &layout.definition.content_identity,
        )
        .map_err(Denial::Relational)?;
        if recorded_content != Some(text(content.to_string())) {
            return Err(Denial::Corrupt);
        }
        let path = super::governed_start_path::observe(
            execution,
            runtime,
            snapshot,
            layout,
            instance,
            *definition,
            usize::from(resources.maximum_definition_nodes()),
            &mut facts,
        )?;
        Some(PublishedWorkflowInstanceRef::observed_current(
            branch,
            instance,
            *definition,
            content,
            original_start_revision,
            path,
        ))
    } else {
        None
    };
    let subject =
        super::subject::read_subject(execution, runtime, snapshot, layout, instance, &mut facts)
            .map_err(Denial::Execution)?;
    let history = read_only::observe_read_only_history(
        execution,
        runtime,
        snapshot,
        layout,
        instance,
        resources.maximum_retained_transitions_per_instance() as usize,
        resources.history_reconstruction_budget(),
    )
    .map_err(Denial::Execution)?;
    let dependencies = super::read_dependencies::observe(
        execution, runtime, snapshot, layout, instance, lineage, &history,
    )?;
    Ok(WorkflowRecord {
        dependencies,
        state,
        instance,
        lineage,
        subject,
        original_start_revision,
        occurrence,
        live,
        history,
        reference,
    })
}
