//! Genuine production Host fixture and passive semantic observations.
pub mod branch_comparison;
pub mod checkpoint_causality;
pub mod comparison;
pub mod concurrent_mutation;
pub mod custody;
pub mod fates;
pub mod idempotency;
pub mod image_comparison;
pub mod native_semantic;
pub mod occurrence;
pub mod opening;
pub mod projection;
pub mod receipts;
pub mod reference_history;
pub mod requests;
mod semantic_text;
pub mod transition_comparison;
pub type OracleHome = worth_query_host::facade::WorthQueryApplicationHome<
    worth_query_testkit_host::facade::host::application_home::schema::HomeIdentitySchema,
    worth_query_testkit_host::facade::host::application_home::HomeProgram,
>;
pub type OracleRuntime =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        worth_query_testkit_host::facade::host::application_home::schema::HomeIdentitySchema,
        worth_query_testkit_host::facade::host::application_home::HomeProgram,
    >;
