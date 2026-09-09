use std::collections::BTreeSet;

use super::super::WorthQueryOperationBindingCounters;

type DeclaredConditionalLocation<'a> = (Option<&'a str>, &'a str);

pub(super) enum ConditionalInventoryAdmission {
    Admitted,
    Missing,
    Drifted,
}

pub(super) struct ConditionalInventoryOwner {
    pub(super) runtime_authority: u64,
    pub(super) installation_generation: u64,
}

pub(super) fn admit_conditional_inventory(
    definition: &worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition,
    installed: &[std::sync::Arc<crate::domain_installation::WorthQueryInstalledConditionalNode>],
    owner: ConditionalInventoryOwner,
    counters: &mut WorthQueryOperationBindingCounters,
) -> ConditionalInventoryAdmission {
    let declared = declared_locations(definition, counters);
    if installed.len() != declared.len() {
        return ConditionalInventoryAdmission::Missing;
    }
    let nodes_admitted = installed.iter().all(|node| {
        counters.conditional_lowering_checks += 1;
        node.operation_identity == definition.canonical_identity()
            && node.runtime_authority == owner.runtime_authority
            && node.installation_generation == owner.installation_generation
            && declared.contains(&location_key(&node.location))
    });
    if nodes_admitted {
        ConditionalInventoryAdmission::Admitted
    } else {
        ConditionalInventoryAdmission::Drifted
    }
}

fn declared_locations<'a>(
    definition: &'a worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition,
    counters: &mut WorthQueryOperationBindingCounters,
) -> BTreeSet<DeclaredConditionalLocation<'a>> {
    let mut locations = BTreeSet::new();
    for node in &definition.semantics().conditional_nodes {
        counters.conditional_declarations_inspected += 1;
        locations.insert((None, node.identity()));
    }
    if let worth_query_installation::facade::WorthQueryOperationWorkflowContract::Declared(
        workflow,
    ) = &definition.semantics().workflow
    {
        for stage in workflow.stages() {
            counters.conditional_workflow_stages_inspected += 1;
            for node in &stage.semantics().conditional_nodes {
                counters.conditional_declarations_inspected += 1;
                locations.insert((Some(stage.identity()), node.identity()));
            }
        }
    }
    locations
}

fn location_key(
    location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
) -> DeclaredConditionalLocation<'_> {
    match location {
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::Operation {
            node_identity,
        } => (None, node_identity),
        worth_query_installation::facade::WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity,
            node_identity,
        } => (Some(stage_identity), node_identity),
    }
}
