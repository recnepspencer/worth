//! A real producer's handler predicate survives Directory Home close and reopen.
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationRequestExt,
    WorthQueryOutputCurrentnessDenial,
};
pub use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
pub use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
pub use worth_query_testkit_document_retention::facade::directory_home::unique_directory;
pub use worth_relational::facade::identity::EntityId;

pub use super::document_retention_model::{
    assessment_output::{AssessmentOutput, RetentionAssessmentDemand, RetentionAssessmentOutputs},
    host::{open_directory, open_directory_with_build},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    programs::validated_second_program,
    retention_entry::DOCUMENT_IDENTITY,
    settled_verdict::{settle, RetentionVerdict},
};

#[path = "producer_predicate_checkpoint/ordinary_source.rs"]
pub mod ordinary_source;
#[path = "producer_predicate_checkpoint/program.rs"]
pub mod program;
pub use program::{validated_program, AssessmentRoot};
