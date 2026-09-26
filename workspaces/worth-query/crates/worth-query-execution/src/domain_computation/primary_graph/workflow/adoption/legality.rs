//! The adoption law for retained workflow facts.
//!
//! A definition the target executes exactly may stay current; any definition
//! may be retired. An instance that has performed nothing may be cancelled,
//! and a compatible one may be carried. An instance with an approval whose
//! operation has not settled has no legal disposition: carrying would let the
//! target run an effect the source approved, and cancelling would abandon a
//! delivery or performed-effect recovery the source still owes. It must be
//! settled or recovered first, then re-inventoried.

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_query_installation::facade::{
    WorthQueryProgramAdoptionRequirements, WorthQueryWorkflowVocabularyCoverage,
};

use super::super::definition::WorkflowDefinitionDependencies;
use super::super::instance::WorkflowInventoriedTransition;
use super::{
    WorthQueryWorkflowCompatibility as Compatibility,
    WorthQueryWorkflowDefinitionDisposition as DefinitionDisposition,
    WorthQueryWorkflowIncompatibility as Incompatibility,
    WorthQueryWorkflowInstanceCustody as Custody,
    WorthQueryWorkflowInstanceDisposition as InstanceDisposition,
};

pub(super) fn definition_compatibility(
    coverage: Option<&WorthQueryWorkflowVocabularyCoverage>,
    requirements: &WorthQueryProgramAdoptionRequirements,
    dependencies: &WorkflowDefinitionDependencies,
) -> Compatibility {
    let Some(coverage) = coverage else {
        return Compatibility::Incompatible(Incompatibility::VocabularyUnsupported {
            spec: dependencies.spec.clone(),
        });
    };
    let mut nodes = dependencies
        .nodes
        .iter()
        .filter_map(|node| {
            node.dependency
                .as_ref()
                .map(|dependency| (node, dependency))
        })
        .collect::<Vec<_>>();
    nodes.sort_unstable_by(|(left, _), (right, _)| left.path.cmp(&right.path));
    for (node, dependency) in nodes {
        if let Some(name) = requirements.changed_workflow_node_dependency(dependency) {
            return Compatibility::Incompatible(Incompatibility::DependencyChanged {
                node_path: node.path.clone(),
                name: name.as_str().to_owned(),
            });
        }
        if !coverage.covers(dependency) {
            return Compatibility::Incompatible(Incompatibility::NodeUncovered {
                node_path: node.path.clone(),
            });
        }
    }
    Compatibility::Compatible
}

/// An approval is outstanding when its latest decision approved and no
/// operation it authorizes has settled a receipt after that decision.
pub(super) fn instance_custody(
    dependencies: &WorkflowDefinitionDependencies,
    transitions: &[WorkflowInventoriedTransition],
) -> Custody {
    let outstanding = dependencies
        .approval_authorities
        .iter()
        .filter(|(approval, operation)| {
            let latest = transitions
                .iter()
                .filter(|transition| transition.node == *approval)
                .max_by_key(|transition| transition.occurrence);
            latest.is_some_and(|decision| {
                matches!(
                    decision.outcome,
                    ApplicationWorkflowControlOutcome::Approved
                ) && !transitions.iter().any(|transition| {
                    transition.node == *operation
                        && transition.receipted
                        && transition.occurrence > decision.occurrence
                })
            })
        })
        .filter_map(|(approval, _)| dependencies.node(*approval))
        .map(|node| node.path.as_str())
        .min();
    if let Some(path) = outstanding {
        return Custody::ApprovalOutstanding {
            approval_node_path: path.to_owned(),
        };
    }
    if transitions.iter().any(|transition| transition.receipted) {
        Custody::Performed
    } else {
        Custody::Unperformed
    }
}

pub(super) fn definition_dispositions(
    compatibility: &Compatibility,
) -> &'static [DefinitionDisposition] {
    match compatibility {
        Compatibility::Compatible => &[DefinitionDisposition::Carry, DefinitionDisposition::Retire],
        Compatibility::Incompatible(_) => &[DefinitionDisposition::Retire],
    }
}

pub(super) fn instance_dispositions(
    compatibility: &Compatibility,
    custody: &Custody,
) -> &'static [InstanceDisposition] {
    match (compatibility.is_compatible(), custody) {
        (_, Custody::ApprovalOutstanding { .. }) | (false, Custody::Performed) => &[],
        (true, Custody::Unperformed) => &[InstanceDisposition::Carry, InstanceDisposition::Cancel],
        (true, Custody::Performed) => &[InstanceDisposition::Carry],
        (false, Custody::Unperformed) => &[InstanceDisposition::Cancel],
    }
}

#[cfg(test)]
#[path = "legality_tests.rs"]
mod tests;
