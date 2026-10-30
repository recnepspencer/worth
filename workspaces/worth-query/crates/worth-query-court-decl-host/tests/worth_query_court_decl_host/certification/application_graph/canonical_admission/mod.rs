pub(crate) use worth_query_testkit_certification_canonical::facade::certification::application_graph::canonical_admission::*;

#[test]
fn sufficient_native_admission_commits_nondefault_root_child_reads_and_restores() {
    let host = host::install(FUNDED_WORK);
    assert_committed(host::advance(&host, Reads::Both, 71_001), 1);
    assert_eq!(
        observed(),
        operation::Observed {
            root: true,
            child: true,
            failures: 0,
            candidates: 1
        }
    );
    assert_source(&host, 1);
    let checkpoint = host::capture(&host);
    drop(host);
    let restored = host::restore(checkpoint, FUNDED_WORK);
    assert_source(&restored, 1);
    assert_committed(host::advance(&restored, Reads::Both, 71_002), 2);
    assert_source(&restored, 2);
}

#[test]
fn cumulative_refusal_survives_a_swallowing_handler_and_does_not_poison_fresh_request() {
    let host = host::install(ONE_VALUE_WORK);
    // Ordinary root and child reads each fund this exact nondefault decoder
    // under the same policy, without an unrelated publication bill.
    assert_source(&host, 0);
    let denied = host::advance(&host, Reads::Both, 71_103);
    let Err(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(
        worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Projection(denial)
    )) = denied else { panic!("swallowed decode refusal must veto the authored effect: {denied:?}"); };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryOperationProjectionDenialKind::InvariantAdmission(
        worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted)));
    assert_eq!(
        observed(),
        operation::Observed {
            root: true,
            child: false,
            failures: 1,
            candidates: 0
        }
    );
    assert!(
        opaque::ignored_refusals() > 0,
        "the actual nested seed must swallow a genuine native callback refusal"
    );
    // Fresh ordinary requests must not inherit the previous refusal, and the
    // authored source remains unchanged after the swallowed mutation failure.
    assert_source(&host, 0);
    let checkpoint = host::capture(&host);
    drop(host);
    let restored = host::restore(checkpoint, FUNDED_WORK);
    assert_source(&restored, 0);
    // Retry the refused command itself with a fresh sufficient request.
    assert_committed(host::advance(&restored, Reads::Both, 71_103), 1);
    assert_source(&restored, 1);
}

#[test]
fn ordinary_query_completion_vetoes_success_after_a_swallowed_second_decode_refusal() {
    use worth_query_host::facade::{
        application_entry::WorthQueryApplicationRequestQueryDenial, primary_graph::*,
    };
    let funded = host::install(FUNDED_WORK);
    assert_eq!(
        host::read_repeated(&funded).unwrap(),
        query::Row {
            value: value(),
            counter: 0
        }
    );
    assert_eq!(repeated_query::returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    drop(funded);

    let host = host::install(ONE_VALUE_WORK);
    assert_eq!(
        host::read(&host, "root").value,
        value(),
        "a single decode fits the same real native allowance"
    );
    let result = host::read_repeated(&host);
    let Err(WorthQueryApplicationRequestQueryDenial::Execution(denial)) = result else {
        panic!("a swallowed second decode must refuse the ordinary read: {result:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::ValueDecode(
                WorthQueryValueDecodeDenial::Checkpoint(
                    worth_execution::MapKernelStop::WorkCeiling
                )
            )
        )
    );
    assert_eq!(
        repeated_query::returned(),
        1,
        "the real projector returned success after catching its second field error"
    );
    assert!(
        opaque::ignored_refusals() > 0,
        "the nested seed itself swallowed the actual native refusal"
    );
    assert_eq!(
        host::read(&host, "root"),
        query::Row {
            value: value(),
            counter: 0
        },
        "a fresh single-decode request succeeds without inherited refusal"
    );
}

pub(crate) mod continuation;

pub(crate) mod live;

pub(crate) mod ordinary_attempt;

pub(crate) mod paired;

pub(crate) mod retained_snapshot;
