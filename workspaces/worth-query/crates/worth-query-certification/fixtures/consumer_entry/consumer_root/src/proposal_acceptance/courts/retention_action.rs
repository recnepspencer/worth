//! Construction and close of the real installed source-completion action.
use super::super::{
    application::model::Model,
    authentication::Caller,
    host::*,
    retention,
    schema::{data::RetentionEdit, Schema},
};
use worth_query_decl::facade::application_program::ApplicationSourceCompletion;
use worth_query_host::facade::application_entry::*;

pub(super) type Operation = WorthQuerySourceOnlyOperation<
    Schema,
    RetentionEdit,
    Program,
    RetentionAction,
    ApplicationSourceCompletion,
>;
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
    host.request(&caller.principal, caller.scope)
        .on_branch(model.branch())
        .mutate(input)
        .without_source()
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, cause)| panic!("capture original input: {cause:?}"))
        .retain_operation::<Program, RetentionAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("resolve installed retention action: {cause:?}"))
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
