//! Recording's healthy D parent cannot borrow its discovery grant from D.
use super::*;
mod program;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;

fn settle(host: &Host, request: &family::Request<'_, '_, '_>) {
    let mut tree = request
        .demand(first::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    for count in 0..128 {
        assert!(count < 127, "original tree did not settle");
        tree = match tree.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                done.deliver_current(request).unwrap();
                done.close(request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("tree terminated"),
        };
    }
}
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: Host = world::install_program_with_value(limits, 3);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&host, &request);
    let document = record!(
        request,
        host,
        world::attach(&host),
        member_edit::Intent {
            key: "m".into(),
            value: 7
        },
        super::super::super::program::MemberEditAction,
        "recording-denied-at-c"
    );
    let selection = document.selection(&request).unwrap();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_output_trees::<program::Program, program::Tree>(
                    &phase,
                    &request,
                    &host,
                    first::Demand("a".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let reads = source_projection::Observation::begin();
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 5,
        })
        .idempotency_owned(&request, "recording-d-grant-forbidden".into())
        .unwrap()
        .retain_in_program::<program::Program, super::super::super::program::MemberEditAction>(
            &host,
        )
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    for count in 0..512 {
        assert!(count < 511, "recording must refuse");
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(_) => {
                panic!("D discovery grant authorized recording")
            }
            WorthQueryHistoryEditAdvance::Refused {
                edit: owner,
                denial,
            } => {
                assert_eq!(
                    reads.discovery_reads(),
                    0,
                    "D discovery never projects before C permission"
                );
                let WorthQueryHistoryEditDenial::Outputs(
                    WorthQueryHistoryOutputDenial::Preparation(
                        WorthQueryRequiredOutputPreparationDenial::SourceQuery(
                            WorthQueryApplicationRequestQueryDenial::Admission(cause),
                        ),
                    ),
                ) = &denial
                else {
                    panic!("wrong control refusal: {denial:?}")
                };
                assert!(
                    matches!(
                        cause.kind(),
                        WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_)
                    ),
                    "{cause:?}"
                );
                assert_eq!(reads.reads(), 0, "healthy original source stays unopened");
                assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
                let document = owner
                    .discard(&request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                assert_eq!(document.selection(&request).unwrap(), selection);
                let at = host
                    .request(&principal, &scope)
                    .on_branch(selection.physical_endpoint().product_branch());
                assert_eq!(
                    at.query(query::member_public::Read("m".into()))
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    7
                );
                assert_eq!(
                    at.query(query::first_output::Read("first:a".into()))
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    30
                );
                assert_eq!(
                    at.query(query::second_output::Read("second:a".into()))
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    1003
                );
                host.with_host_advancement(|phase| document.close(&phase).unwrap())
                    .unwrap();
                return;
            }
        };
    }
}
#[test]
fn a_recording_parent_uses_current_discovery_permission_before_disclosing_d() {
    exercise(false);
}
#[test]
fn worker_only_a_recording_parent_uses_current_discovery_permission_before_disclosing_d() {
    exercise(true);
}
