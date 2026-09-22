//! Immutable installation binding for authored workflow vocabulary.

mod definition_contract;
mod vocabulary;

#[cfg(test)]
mod tests;

pub use definition_contract::{
    WorthQueryInstalledWorkflowApprovalBinding, WorthQueryInstalledWorkflowDefinitionContract,
};
pub use vocabulary::{
    WorthQueryApplicationWorkflowInstallationDenial,
    WorthQueryApplicationWorkflowInstallationDenialKind,
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
    WorthQueryInstalledApplicationWorkflowSpec, WorthQueryWorkflowHistoryReconstructionBudget,
};
