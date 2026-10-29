//! What a migration refuses: another workflow's definition, a resume that
//! could run before its producer, and every request for a migrated source
//! except the exact replay of its migration.

pub use worth_query_host::facade::application_entry::{
    WorkflowProposalPreparationDenial, WorthQueryWorkflowProposalPreparationDenial,
};

pub use super::super::document_retention_model::workflow::{
    migrate_instance, proposal_terminal_definition, reproposing_document_definition,
};
pub use super::instance_migration::{migration_denial, replace_definition, started, supersede};
pub use super::*;
