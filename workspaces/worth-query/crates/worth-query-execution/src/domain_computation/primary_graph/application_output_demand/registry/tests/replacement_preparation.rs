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
        state.obligation_reserved_bytes = old_record.obligation_reserved_bytes();
        state.records.insert(old, old_record);
        state.records.insert(next, next_record);
    }
    (registry, old_interest, next_interest, commit)
}

#[test]
fn replacement_member_walk_refuses_every_budget_and_retries_by_value() {
    let (_, _, _, commit) = fixture();
    let run = |registry: &WorthQueryOutputDemandRegistry,
               old: &WorthQueryOutputDemandInterest,
               next: &WorthQueryOutputDemandInterest,
               admission: &mut InvalidationEditAdmission| {
        registry.finish_replaced_interest(old, next, "replacement", admission)
    };
    let (foreign, old, next, _) = replacement_shape(commit.clone(), false);
    let mut foreign_work = record_admission();
    run(&foreign, &old, &next, &mut foreign_work).unwrap();
    let (completed, old, next, _) = replacement_shape(commit.clone(), true);
    let mut measured = record_admission();
    run(&completed, &old, &next, &mut measured).unwrap();
    // Same map size and transfer; only the extra occurrence row differs.
    // Its quote and prepaid execution allowance each price it once.
    assert_eq!(measured.charged_work(), foreign_work.charged_work() + 2);
    let expected = replacement_value(&completed.state.lock().unwrap());
    for k in 0..measured.charged_work() {
        let (retry, old, next, _) = replacement_shape(commit.clone(), true);
        let before = replacement_value(&retry.state.lock().unwrap());
        let mut limited = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: Some(k),
            maximum_preparation_bytes: None,
        });
        let denial = run(&retry, &old, &next, &mut limited).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert_eq!(replacement_value(&retry.state.lock().unwrap()), before);
        run(&retry, &old, &next, &mut record_admission()).unwrap();
        assert_eq!(
            replacement_value(&retry.state.lock().unwrap()),
            expected,
            "k={k}"
        );
    }
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

#[derive(Debug, PartialEq, Eq)]
struct ReplacementValue {
    rows: Vec<ReplacementRow>,
    members: Vec<WorthQueryOutputDemandKey>,
    totals: [usize; 3],
    records: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct ReplacementRow {
    key: WorthQueryOutputDemandKey,
    denial: Option<WorthQueryOutputDemandDenialKind>,
    interests: [usize; 3],
    performed: Vec<(super::super::SourceEpoch, Commit)>,
    commits: Vec<Commit>,
    slots: usize,
    has_commit_capacity: bool,
}

fn replacement_value(state: &DemandRegistryState) -> ReplacementValue {
    use std::sync::atomic::Ordering::Acquire;
    assert!(state.source_custody.is_empty());
    ReplacementValue {
        rows: state
            .records
            .iter()
            .map(|(key, row)| {
                assert!(row.prerequisites.is_empty());
                assert!(row.settlements.is_empty());
                let denial = match &row.state {
                    DemandState::Admitted => None,
                    DemandState::Failed(denial) => Some(denial.kind()),
                    _ => panic!("unexpected replacement state"),
                };
                ReplacementRow {
                    key: key.clone(),
                    denial,
                    interests: [
                        row.interests,
                        row.required_interests,
                        row.framework_required_count,
                    ],
                    performed: row
                        .performed_obligations
                        .iter()
                        .map(|obligation| {
                            (obligation.source.clone(), obligation.source_commit.clone())
                        })
                        .collect(),
                    commits: row.source_commits.clone(),
                    slots: row.performed_obligations.capacity(),
                    has_commit_capacity: row.source_commit_capacity.is_some(),
                }
            })
            .collect(),
        members: state
            .required_keys
            .iter()
            .map(|key| key.as_ref().clone())
            .collect(),
        totals: [
            state.required_reserved_bytes,
            state.obligation_reserved_bytes,
            state.required_custody_retained_bytes.load(Acquire),
        ],
        records: state.record_retained_bytes.load(Acquire),
    }
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
    for (budget, expected_kind) in [
        (
            CompanionPreflightBudget {
                maximum_work_visits: Some(work - 1),
                maximum_preparation_bytes: None,
            },
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        ),
        (
            CompanionPreflightBudget {
                maximum_work_visits: Some(work),
                maximum_preparation_bytes: Some(bytes - 1),
            },
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        ),
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
        let denial = registry
            .finish_replaced_interest(
                &old,
                &next,
                "replacement",
                &mut InvalidationEditAdmission::new(budget),
            )
            .expect_err("one short stops before either record moves");
        assert_eq!(denial.kind(), expected_kind);
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
        state.obligation_budget_bytes = Some(state.obligation_reserved_bytes);
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

#[test]
fn registry_supersession_reuses_its_prepaid_member_walk() {
    let (_, _, _, commit) = fixture();
    let (registry, old, _next, _) = fixture_with_commit(commit);
    let old_key = old.key.clone();
    {
        let mut state = registry.state.lock().unwrap();
        state.records.get_mut(&old_key).unwrap().state = closed_retirement::ready();
        let member = state.prepare_required_member(&old_key).unwrap();
        state.install_required_member(member);
    }
    // A real release leaves the cached row required by its performed output.
    drop(old);
    let before = super::super::record_map::occurrence_rows_visited();
    let (claims, member) = {
        let mut state = registry.state.lock().unwrap();
        assert_eq!(state.records[&old_key].interests, 0);
        assert!(state.required_keys.contains(&old_key));
        super::super::refreshed_rejoin::price_member_release(
            &mut state,
            &mut record_admission(),
            &old_key,
        )
        .unwrap()
        .complete(|priced| {
            let (_, claims) = priced
                .state_mut()
                .finish_superseded_row(&old_key, "replacement");
            Ok((claims, priced.release_member(&old_key)))
        })
    };
    let refund = member.refund_required_bytes;
    drop(claims);
    drop(member);
    let mut state = registry.state.lock().unwrap();
    state.required_reserved_bytes -= refund;
    assert!(!state.required_keys.contains(&old_key));
    // Two occurrence rows: one quote and one prepaid member search, 2 + 2.
    assert_eq!(
        super::super::record_map::occurrence_rows_visited() - before,
        4
    );
}
