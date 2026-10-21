//! Registry transfer evidence; this does not certify output currentness.

use super::*;

type Commit = worth_runtime_world::facade::CompositeCommitIdentity;

fn fixture() -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    Commit,
) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    fixture_with_commit(commit)
}

fn fixture_with_commit(
    commit: Commit,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    Commit,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("replacement", 1, 1, 1);
    let next = key_with_identity("replacement", 2, 1, 1);
    let mut old_record = record(occurrence(), DemandState::Admitted, 1);
    for _ in 0..2 {
        old_record
            .performed_obligations
            .push(super::super::PerformedOutputObligation {
                source_commit: commit.clone(),
                source: old.source.clone(),
            });
    }
    old_record
        .performed_obligations
        .push(super::super::PerformedOutputObligation {
            source_commit: commit.clone(),
            source: key("unmatched", 1, 2).source,
        });
    let next_record = record(occurrence(), DemandState::Admitted, 1);
    let old_interest = interest(&registry, old.clone(), Arc::clone(&old_record.wake));
    let next_interest = interest(&registry, next.clone(), Arc::clone(&next_record.wake));
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(old, old_record);
        state.records.insert(next, next_record);
    }
    (registry, old_interest, next_interest, commit)
}

fn replacement_shape(
    commit: Commit,
    own: bool,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    Commit,
) {
    let fixture = fixture_with_commit(commit);
    let other = key_with_identity("replacement", 0, if own { 1 } else { 2 }, 1);
    assert_eq!(other.same_occurrence(&fixture.1.key), own);
    fixture
        .0
        .state
        .lock()
        .unwrap()
        .records
        .insert(other, record(occurrence(), DemandState::Admitted, 0));
    fixture
}

#[test]
fn replacement_prepares_exact_commit_growth_and_transfers_matching_obligations() {
    let (registry, old, next, commit) = fixture();
    registry
        .finish_replaced_interest(&old, &next, "replacement")
        .unwrap();
    let state = registry.state.lock().unwrap();
    let successor = &state.records[&next.key];
    assert_eq!(successor.performed_obligations.len(), 2);
    assert_eq!(successor.source_commits, [commit]);

    assert!(state.required_keys.contains(&next.key));
    assert_eq!(state.records[&old.key].performed_obligations.len(), 0);
}
