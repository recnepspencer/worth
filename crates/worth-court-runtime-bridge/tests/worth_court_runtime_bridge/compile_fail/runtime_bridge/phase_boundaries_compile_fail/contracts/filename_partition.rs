use worth_testkit_compiler::ProductionSemanticDenialContract as Contract;

pub(in super::super) fn between(first: u8, last: u8) -> Vec<Contract<'static>> {
    [
        super::async_sources::contracts(),
        super::committed_patch::contracts(),
        super::conditional::contracts(),
        super::construction::contracts(),
        super::diagnostics::contracts(),
        super::preview::contracts(),
        super::routing::contracts(),
        super::subscription_certification::contracts(),
        super::subscription_delivery::contracts(),
        super::subscription_preview::contracts(),
        super::temporal::contracts(),
        super::writeback::contracts(),
    ]
    .into_iter()
    .flatten()
    .filter(|contract| (first..=last).contains(&contract.fixture.as_bytes()[0]))
    .collect()
}
