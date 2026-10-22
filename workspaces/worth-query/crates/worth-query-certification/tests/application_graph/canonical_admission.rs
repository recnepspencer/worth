//! Original-request cumulative decoding and native completion veto before effects.
#[path = "canonical_admission/binding.rs"]
mod binding;
#[path = "canonical_admission/continuation.rs"]
mod continuation;
#[path = "canonical_admission/host.rs"]
mod host;
#[path = "canonical_admission/limits.rs"]
mod limits;
#[path = "canonical_admission/live.rs"]
mod live;
#[path = "canonical_value/model.rs"]
#[expect(
    dead_code,
    reason = "the shared model's wire golden is exercised by canonical_value"
)]
#[expect(
    clippy::duplicate_mod,
    reason = "the admission fixture instantiates the accepted nested grammar under its opaque swallowing codec"
)]
mod model;
#[path = "canonical_admission/opaque.rs"]
mod opaque;
#[path = "canonical_admission/operation.rs"]
mod operation;
#[path = "canonical_admission/ordinary_attempt.rs"]
mod ordinary_attempt;
#[path = "canonical_admission/paired.rs"]
mod paired;
#[path = "canonical_admission/query.rs"]
mod query;
#[path = "canonical_admission/repeated_query.rs"]
mod repeated_query;
#[path = "canonical_admission/retained_snapshot.rs"]
mod retained_snapshot;
#[path = "canonical_admission/schema.rs"]
mod schema;

use model::{authored_value, NestedValue};
use operation::{observed, Reads};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

const FUNDED_WORK: u64 = 8_000_000;
// Both values satisfy the same canonical grammar. This host policy
// funds either decode independently, but cannot fund both in one original request.
const ONE_VALUE_WORK: u64 = 26_000;

fn value() -> NestedValue {
    let mut value = authored_value();
    value.children = (0..16)
        .map(|index| {
            let mut child = value.children[index % 2].clone();
            child.tag.0 = format!("{index:02}{}", "β".repeat(127));
            child
        })
        .collect();
    value
}

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
        worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionDenialKind::ValueDecode(
            worth_query_host::facade::primary_graph::WorthQueryValueDecodeDenial::Checkpoint(worth_execution::MapKernelStop::WorkCeiling))));
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

fn assert_source(host: &host::Host, counter: u64) {
    assert_eq!(
        host::read(host, "root"),
        query::Row {
            value: value(),
            counter
        }
    );
    assert_eq!(
        host::read(host, "child"),
        query::Row {
            value: value(),
            counter: 0
        }
    );
}
fn assert_committed(
    result: Result<
        WorthQueryApplicationMutationOutcome<operation::Denied, operation::Written>,
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
    >,
    counter: u64,
) {
    match result.expect("genuine funded request executes") {
        WorthQueryApplicationMutationOutcome::Committed { result, .. } => {
            assert_eq!(result.counter, counter)
        }
        outcome => panic!("funded canonical reads must commit: {outcome:?}"),
    }
}
