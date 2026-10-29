//! Adoption decides exactly the live workflow facts the selected branch holds
//! at publication: a start after preparation stales the adoption, and a fork
//! decides only its own live facts, never the history it copied.

pub use worth_query_host::facade::application_entry::{
    WorkflowProgressOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryWorkflowAdvancePreparationDenial,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryBranchAdoptionPreparationDenial,
};
pub use worth_query_host::facade::runtime::NoEffectCause;

pub use super::workflow_participant::{expect_started, live_instance_on_first_program};
pub use crate::facade::certification::document_retention_model::workflow::{
    advance_instance, advance_instance_on, prepare_second_program_adoption, publish_adoption,
    start_instance,
};
