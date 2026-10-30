//! Original-request cumulative decoding and native completion veto before effects.
#[path = "canonical_admission/binding.rs"]
pub mod binding;
#[path = "canonical_admission/continuation.rs"]
pub mod continuation;
#[path = "canonical_admission/host.rs"]
pub mod host;
#[path = "canonical_admission/limits.rs"]
pub mod limits;
#[path = "canonical_admission/live.rs"]
pub mod live;
#[path = "canonical_value/model.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the admission fixture instantiates the accepted nested grammar under its opaque swallowing codec"
)]
pub mod model;
#[path = "canonical_admission/opaque.rs"]
pub mod opaque;
#[path = "canonical_admission/operation.rs"]
pub mod operation;
#[path = "canonical_admission/ordinary_attempt.rs"]
pub mod ordinary_attempt;
#[path = "canonical_admission/paired.rs"]
pub mod paired;
#[path = "canonical_admission/query.rs"]
pub mod query;
#[path = "canonical_admission/repeated_query.rs"]
pub mod repeated_query;
#[path = "canonical_admission/retained_snapshot.rs"]
pub mod retained_snapshot;
#[path = "canonical_admission/schema.rs"]
pub mod schema;

pub use model::{authored_value, NestedValue};
pub use operation::{observed, Reads};
pub use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

pub const FUNDED_WORK: u64 = 8_000_000;
// Both values satisfy the same canonical grammar. This host policy
// funds either decode independently, but cannot fund both in one original request.
pub const ONE_VALUE_WORK: u64 = 26_000;

pub fn value() -> NestedValue {
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

pub fn assert_source(host: &host::Host, counter: u64) {
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
pub fn assert_committed(
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

#[path = "canonical_admission/request_funding.rs"]
pub mod request_funding;
