//! Public #382: a span install only ever widens the target's generation.
use super::span_widens;
use crate::history::access::relevant_writes::generation::AuthoredWriteGeneration;
use crate::history::data::CommitId;
use worth_execution::ExecutionAllocationPolicy;

fn span(commits: std::ops::RangeInclusive<u64>) -> AuthoredWriteGeneration {
    let mut generation = AuthoredWriteGeneration::default();
    for (ordinal, commit) in commits.enumerate() {
        generation
            .prefix
            .insert_cloned_native_in(
                &CommitId(commit),
                &(ordinal as u64 + 1),
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    generation
}

#[test]
fn a_span_install_never_narrows_the_installed_generation() {
    assert!(span_widens(None, CommitId(5)), "an empty target installs");
    assert!(
        span_widens(Some(&span(5..=7)), CommitId(3)),
        "an older original widens"
    );
    assert!(
        !span_widens(Some(&span(3..=7)), CommitId(5)),
        "a narrower seed that lost the race is discarded"
    );
    assert!(
        !span_widens(
            Some(&AuthoredWriteGeneration::unsupported_merge()),
            CommitId(3)
        ),
        "a merge marker is never replaced by a seed"
    );
}
