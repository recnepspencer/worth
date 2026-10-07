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
        state.obligation_reserved_bytes = old_record.obligation_reserved_bytes();
        state.records.insert(old, old_record);
        state.records.insert(next, next_record);
    }
    (registry, old_interest, next_interest, commit)
}

#[test]
fn replacement_prepares_exact_commit_growth_and_transfers_matching_obligations() {
    let (registry, old, next, commit) = fixture();
    registry
        .finish_replaced_interest(&old, &next, "replacement", &mut record_admission())
        .unwrap();
    let state = registry.state.lock().unwrap();
    let successor = &state.records[&next.key];
    assert_eq!(successor.performed_obligations.len(), 2);
    assert_eq!(successor.source_commits, [commit]);
    assert!(successor.source_commit_capacity.is_some());
    assert!(state.required_keys.contains(&next.key));
    assert_eq!(state.records[&old.key].performed_obligations.len(), 0);
    assert_eq!(
        state.obligation_reserved_bytes,
        successor.obligation_reserved_bytes()
    );
}

#[test]
fn replacement_one_short_work_or_scratch_preserves_both_records_and_tickets() {
    let (registry, old, next, _) = fixture();
    let mut measured = record_admission();
    registry
        .finish_replaced_interest(&old, &next, "replacement", &mut measured)
        .unwrap();
    let work = measured.charged_work();
    let bytes = measured.charged_bytes();
    assert!(work > 0 && bytes > 0);
    for budget in [
        CompanionPreflightBudget {
            maximum_work_visits: work - 1,
            maximum_preparation_bytes: bytes,
        },
        CompanionPreflightBudget {
            maximum_work_visits: work,
            maximum_preparation_bytes: bytes - 1,
        },
    ] {
        let (registry, old, next, _) = fixture();
        let (obligation_bytes, record_bytes) = {
            let state = registry.state.lock().unwrap();
            (
                state.obligation_reserved_bytes,
                state
                    .record_retained_bytes
                    .load(std::sync::atomic::Ordering::Acquire),
            )
        };
        registry
            .finish_replaced_interest(
                &old,
                &next,
                "replacement",
                &mut InvalidationEditAdmission::new(budget),
            )
            .expect_err("one short stops before either record moves");
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&old.key].performed_obligations.len(), 3);
        assert!(state.records[&next.key].performed_obligations.is_empty());
        assert!(state.records[&next.key].source_commits.is_empty());
        assert!(state.records[&next.key].source_commit_capacity.is_none());
        assert!(state.required_keys.is_empty());
        assert_eq!(state.obligation_reserved_bytes, obligation_bytes);
        assert_eq!(
            state
                .record_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
            record_bytes
        );
    }
}

#[test]
fn replacement_aggregate_peak_denial_preserves_old_obligations() {
    let (registry, old, next, _) = fixture();
    {
        let mut state = registry.state.lock().unwrap();
        state.obligation_budget_bytes = state.obligation_reserved_bytes;
    }
    let denial = registry
        .finish_replaced_interest(&old, &next, "replacement", &mut record_admission())
        .expect_err("the old and prepared new backing must coexist before transfer");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
    );
    let state = registry.state.lock().unwrap();
    assert_eq!(state.records[&old.key].performed_obligations.len(), 3);
    assert!(state.records[&next.key].performed_obligations.is_empty());
    assert!(state.required_keys.is_empty());
}
