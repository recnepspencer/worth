//! Descriptive receipt replay cannot mark a new effect in this advance.
use super::*;

#[test]
fn only_a_new_commit_records_a_publication_in_this_advance() {
    graph::output_lineage::own_write_fixture::with_committed_own_write(|world, receipt, _, _| {
        let recorded = std::cell::Cell::new(0);
        let recovered = crate::domain_computation::primary_graph::tests::fixture::recorded_receipt::read_original(world, &receipt);
        let replay = commit_receipt(
            "published fixture",
            WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered),
        )
        .err()
        .expect("original data cannot mint a producer publication capability");
        assert!(replay.recovered_publication().is_some());
        assert_eq!(
            recorded.get(),
            0,
            "replay publishes nothing in this advance"
        );
        let publication = commit_receipt(
            "published fixture",
            WorthQueryApplicationCommitOutcome::Committed(receipt),
        )
        .unwrap_or_else(|_| panic!("a new commit retains its receipt"));
        publication.record_publication(|| recorded.set(recorded.get() + 1));
        assert_eq!(recorded.get(), 1, "the new effect marks exactly once");
    });
}
