//! Named host policy for direct seam controls in Bank's unit fixtures.
use bank_domain::schema::BankSchema;
use worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(crate) fn with_phase<Output>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    operation: impl FnOnce(&WorthQueryAdvancementPhase<'_>) -> Output,
) -> Output {
    let scope = crate::estate_capability_admission::fixture::request_scope();
    runtime
        .with_application_advancement(&scope, |phase| operation(&phase))
        .expect("the fixture host policy admits its own installed runtime")
}
