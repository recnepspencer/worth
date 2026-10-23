//! One authored numeric model shared by the correction example and its public court.
pub mod adoption;
pub mod adoption_courts;
pub mod calculation;
pub mod calculation_query;
pub mod calculation_readiness;
pub mod compensation_courts;
pub mod compensation_movement;
pub mod correction;
pub mod edit;
pub mod host;
pub mod other_edit;
pub mod output_program;
pub mod principal_status;
pub mod program;
pub mod publication_courts;
pub mod query;
pub mod required_output_courts;
pub mod required_output_recovery;
pub mod schema;
pub mod source_output_correction;
pub mod source_output_courts;
pub mod source_output_handlers;
pub mod source_output_model;

#[path = "../../tests/application_graph/document_retention_model/operator_identity.rs"]
pub mod authentication;
