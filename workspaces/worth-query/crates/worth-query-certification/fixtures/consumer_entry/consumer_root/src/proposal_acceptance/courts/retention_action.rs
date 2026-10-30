//! Construction and close of the real installed source-completion action.
use super::super::{
    application::{model::Model, retention_action},
    authentication::Caller,
    host::*,
    retention,
    schema::{data::RetentionEdit, Schema},
};
use worth_query_host::facade::application_entry::*;

pub(super) use retention_action::Operation;
pub(super) fn edit(model: &Model<'_>, days: u64) -> RetentionEdit {
    RetentionEdit {
        model: model.identity.clone(),
        target: model.target.clone(),
        days,
    }
}
pub(super) fn construct(
    host: &Host,
    model: &Model<'_>,
    caller: &Caller<'_>,
    input: RetentionEdit,
    key: u64,
) -> Operation {
    retention_action::construct(
        host,
        model.branch(),
        caller,
        &input,
        retention::StepKey {
            attempt: key,
            ordinal: 0,
        },
    )
    .unwrap_or_else(|cause| panic!("construct ordinary retention action: {cause:?}"))
}
pub(super) fn complete(
    operation: Operation,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    expected_days: u64,
) {
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(request) else {
        panic!("lawful ordinary retention edit must complete")
    };
    match completed.evidence() {
        WorthQuerySourceCompletion::Fresh { result, .. } => {
            let retention::Changed { days } = result;
            assert_eq!(*days, expected_days);
        }
        WorthQuerySourceCompletion::Recovered { .. } => panic!("first execution must be fresh"),
    }
    completed
        .close(request)
        .unwrap_or_else(|(_, cause)| panic!("close completed retention action: {cause:?}"));
}
