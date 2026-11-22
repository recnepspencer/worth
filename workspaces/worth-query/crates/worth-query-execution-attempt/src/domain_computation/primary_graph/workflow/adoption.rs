//! Program adoption for branch-local workflow facts.
//!
//! Adoption reads every current definition and every live instance this exact
//! branch incarnation authored, compares each against the target's installed
//! workflow vocabulary, and requires an explicit disposition for each before
//! the adoption transaction is prepared. Nothing is carried implicitly.

pub mod choices;
pub mod coverage;
pub mod inventory;
pub mod legality;
pub mod occurrence;
pub mod preparation;
pub mod staging;
pub mod truth;

pub mod request;
pub mod selected_population;
pub use choices::AdmittedWorkflowDispositions;
pub use coverage::WorkflowVocabularyCoverageRegistry;
pub use inventory::{
    inventory_workflows, inventory_workflows_from_records, WorkflowAdoptionInventoryRequest,
};
pub use occurrence::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowCompatibility,
    WorthQueryWorkflowDefinitionOccurrence, WorthQueryWorkflowIncompatibility,
    WorthQueryWorkflowInstanceCustody, WorthQueryWorkflowInstanceOccurrence,
};
pub use preparation::{prepare_workflow_dispositions, WorkflowDispositionPreparationDenial};
pub use request::WorthQueryWorkflowDispositions;
pub use selected_population::workflow_record_population_is_empty;
pub use truth::WorkflowAdoptionReadDenial;
pub use truth::{WorkflowAdoptionEntity, WorkflowAdoptionTruth};

pub use occurrence::{
    WorthQueryWorkflowDefinitionDisposition, WorthQueryWorkflowInstanceDisposition,
};
