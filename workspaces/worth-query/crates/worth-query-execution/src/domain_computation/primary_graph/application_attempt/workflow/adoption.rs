//! Program adoption for branch-local workflow facts.
//!
//! Adoption reads every current definition and every live instance this exact
//! branch incarnation authored, compares each against the target's installed
//! workflow vocabulary, and requires an explicit disposition for each before
//! the adoption transaction is prepared. Nothing is carried implicitly.

mod choices;
mod coverage;
mod inventory;
mod legality;
mod occurrence;
mod staging;
mod truth;

mod request;
mod selected_population;
pub(in crate::domain_computation::primary_graph) use choices::AdmittedWorkflowDispositions;
pub(in crate::domain_computation::primary_graph) use coverage::WorkflowVocabularyCoverageRegistry;
pub(in crate::domain_computation::primary_graph) use inventory::{
    inventory_workflows, WorkflowAdoptionInventoryRequest,
};
pub(in crate::domain_computation::primary_graph) use occurrence::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowCompatibility,
    WorthQueryWorkflowDefinitionOccurrence, WorthQueryWorkflowIncompatibility,
    WorthQueryWorkflowInstanceCustody, WorthQueryWorkflowInstanceOccurrence,
};
pub use request::WorthQueryWorkflowDispositions;
pub(in crate::domain_computation::primary_graph) use staging::stage_workflow_dispositions;
pub(in crate::domain_computation::primary_graph) use truth::{
    WorkflowAdoptionEntity, WorkflowAdoptionReadDenial, WorkflowAdoptionTruth,
};

pub use occurrence::{
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowInstanceDisposition,
};
