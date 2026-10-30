//! Assertions over real application mutation outcomes.
pub use super::operation::{Edited, GraphDenial};
pub use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome, primary_graph::*,
};

pub fn committed(outcome: WorthQueryApplicationMutationOutcome<GraphDenial, Edited>) {
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
pub fn denied(
    outcome: WorthQueryApplicationMutationOutcome<GraphDenial, Edited>,
) -> WorthQueryApplicationCommitDenial {
    match outcome {
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) => denial,
        other => panic!("expected the native commit law refusal: {other:?}"),
    }
}
