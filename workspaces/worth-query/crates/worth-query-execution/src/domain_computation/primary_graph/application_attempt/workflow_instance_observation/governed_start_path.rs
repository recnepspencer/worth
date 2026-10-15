//! Retained native resume-path selection and exact definition membership.
use super::*;
use crate::domain_computation::primary_graph::application_discovery::WorthQueryWorkflowReadDenial as Denial;
use worth_foundational::facade::InternedString;

pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    definition: EntityId,
    maximum_nodes: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, Denial> {
    let resume = observe_field_value(
        runtime,
        execution,
        snapshot,
        instance,
        layout.instance.entity_kind,
        &layout.instance.resume_node_path,
    )
    .map_err(Denial::Execution)?;
    let path = match resume {
        Some(AspectValue::String(InternedString::Raw(path))) => path,
        None => {
            let starts = adjacency_with_kind(
                execution,
                runtime,
                snapshot,
                layout.definition_start_relation,
                definition,
                2,
                "workflow start node unavailable",
                facts,
            )
            .map_err(Denial::Execution)?;
            let [start] = starts.as_slice() else {
                return Err(Denial::Corrupt);
            };
            match observe_field_value(
                runtime,
                execution,
                snapshot,
                *start,
                layout.node.entity_kind,
                &layout.node.path,
            )
            .map_err(Denial::Execution)?
            {
                Some(AspectValue::String(InternedString::Raw(path))) => path,
                _ => return Err(Denial::Corrupt),
            }
        }
        _ => return Err(Denial::Corrupt),
    };
    // Only this retained definition's bounded members, never a population scan.
    // Native adjacency and field owners charge their reads on the same execution.
    let nodes = adjacency_with_kind(
        execution,
        runtime,
        snapshot,
        layout.definition_node_relation,
        definition,
        maximum_nodes.checked_add(1).ok_or(Denial::Corrupt)?,
        "workflow definition nodes unavailable",
        facts,
    )
    .map_err(Denial::Execution)?;
    if nodes.len() > maximum_nodes {
        return Err(Denial::Corrupt);
    }
    let mut matches = 0;
    for node in nodes {
        let value = observe_field_value(
            runtime,
            execution,
            snapshot,
            node,
            layout.node.entity_kind,
            &layout.node.path,
        )
        .map_err(Denial::Execution)?;
        match value {
            Some(AspectValue::String(InternedString::Raw(candidate))) => {
                matches += usize::from(candidate == path);
            }
            _ => return Err(Denial::Corrupt),
        }
    }
    if matches != 1 {
        return Err(Denial::Corrupt);
    }
    Ok(path)
}
