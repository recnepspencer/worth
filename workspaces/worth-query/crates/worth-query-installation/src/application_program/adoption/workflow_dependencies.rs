//! Workflow dependencies a program change stops supplying unchanged.
//!
//! A published workflow definition records the binding identities and
//! operation names its nodes act through. Adoption carries that definition only
//! while the target still supplies every one of them with unchanged meaning, so
//! installation names the exact dependency identities whose supplying action
//! the Operations family reports removed or changed.

use std::collections::BTreeSet;

use worth_query_declaration::facade::application_program::{
    ApplicationSemanticChangeKind, ApplicationSemanticDiff, ApplicationSemanticFamily,
};

use crate::application_program::support::WorthQueryProgramSupportEntry;

/// A binding identity or operation name a workflow definition node acts
/// through, as program-change comparison supplies it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryWorkflowDependencyName(String);

impl WorthQueryWorkflowDependencyName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identities no unchanged source action still supplies.
///
/// An operation name several actions share stays supplied while any of them
/// survives unchanged, because a node naming only the operation still resolves.
pub(super) fn changed_workflow_dependencies(
    source: &WorthQueryProgramSupportEntry,
    diff: &ApplicationSemanticDiff,
) -> Vec<WorthQueryWorkflowDependencyName> {
    let changed_subjects = diff
        .changes()
        .iter()
        .filter(|change| {
            change.family() == ApplicationSemanticFamily::Operations
                && matches!(
                    change.kind(),
                    ApplicationSemanticChangeKind::Removed | ApplicationSemanticChangeKind::Changed
                )
        })
        .map(|change| change.subject())
        .collect::<BTreeSet<_>>();
    let (changed, stable): (Vec<_>, Vec<_>) = source
        .action_dependencies()
        .iter()
        .partition(|dependency| changed_subjects.contains(dependency.subject.as_str()));
    let stable = stable
        .into_iter()
        .map(|dependency| dependency.identity.as_str())
        .collect::<BTreeSet<_>>();
    changed
        .into_iter()
        .map(|dependency| dependency.identity.as_str())
        .filter(|identity| !stable.contains(identity))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|identity| WorthQueryWorkflowDependencyName(identity.to_owned()))
        .collect()
}
