//! Record actual old program outputs and reopen their captured bytes in another world.
use super::*;
pub(super) fn checkpoint() -> (
    WorthQueryApplicationCheckpoint,
    (query::Row, query::Row),
    ApplicationProgramRevision,
) {
    let host: WorthQueryProgramApplicationRuntime<Schema, original::Program> = world::install_program_with_value(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut outputs = request
        .demand(first::Demand("a".into()))
        .retain_program_outputs::<original::Program, original::Tree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    loop {
        outputs = match outputs.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(&request).unwrap();
                done.close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                break;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("old tree terminated"),
        };
    }
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let second = request
        .query(query::second_output::Read("second:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!((first.value, second.value), (30, 1003));
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    (checkpoint, (first, second), *host.owned_revision())
}
pub(super) fn record<'a, P>(
    host: &'a WorthQueryProgramApplicationRuntime<Schema, P>,
    request: &family::Request<'_, '_, '_>,
) -> (world::Document<'a>, WorthQueryHistoryNodeId)
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
{
    let document = world::attach(host);
    let before = document.current_step(request).unwrap().id();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 5,
        })
        .idempotency_owned(request, "cold-parent-model-edit".into())
        .unwrap()
        .retain_in_program::<P, super::super::super::program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    (document, before)
}
