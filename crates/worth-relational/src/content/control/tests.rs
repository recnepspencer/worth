//! Literal refusal precedence is independent of request execution machinery.
use super::{final_denial, Denial};
use crate::execution::RelationalExecutionDenialCause as Cause;

#[test]
fn content_stop_precedence_preserves_exact_inner_only_for_nested_handoff() {
    let nested = Denial::Execution(Cause::NestedStopped);
    let work = Denial::Execution(Cause::WorkExhausted);
    let cancel = Denial::Execution(Cause::Cancelled);
    let memory = Denial::Execution(Cause::PolicyMemoryExhausted {
        requested: 101,
        admitted: 17,
        ancestor: 2,
    });
    for (inner, outer, expected) in [
        (Some(work), nested, work),
        (Some(Denial::FormatMismatch), nested, Denial::FormatMismatch),
        (
            Some(Denial::SizeLimit {
                observed: 97,
                maximum: 31,
            }),
            nested,
            Denial::SizeLimit {
                observed: 97,
                maximum: 31,
            },
        ),
        (None, nested, nested),
        (Some(Denial::FormatMismatch), cancel, cancel),
        (Some(cancel), memory, memory),
        (Some(Denial::EncodingInconsistency), work, work),
    ] {
        assert_eq!(final_denial(inner, outer), expected);
    }
}
