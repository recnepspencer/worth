//! Genuine production Host fixture and passive semantic observations.
pub mod branch_comparison;
pub mod comparison;
pub mod custody;
pub mod idempotency;
pub mod image_comparison;
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
