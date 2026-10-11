//! A later request finishes member cleanup before unlinking its queued head.
use super::*;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

fn queued(seed: u64) -> (DemandRegistryState, Arc<WorthQueryOutputDemandKey>) {
    let mut state = DemandRegistryState::default();
    let selected = Arc::new(key("queued-member", seed + 1, 2));
    let mut row = record(
        occurrence(),
        DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Closed,
            "",
        )),
        0,
    );
    row.framework_required_count = 1;
    row.prerequisites.reserve_exact(1);
    state.required_reserved_bytes = std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
    if seed != 0 {
        let upstream = Arc::new(key("queued-prerequisite", seed + 1, 3));
        row.prerequisites.push(Arc::clone(&upstream));
        let mut prior = record(
            occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "",
            )),
            0,
        );
        prior.framework_required_count = 1;
        prior.prerequisites.reserve_exact(1);
        state.required_reserved_bytes +=
            prior.prerequisites.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
        state.records.insert(upstream.as_ref().clone(), prior);
        let member = state.prepare_required_member(&upstream).unwrap();
        state.install_required_member(member);
    }
    state.records.insert(selected.as_ref().clone(), row);
    let member = state.prepare_required_member(&selected).unwrap();
    state.install_required_member(member);
    // The committed publication releases its prior claim. It cannot refuse.
    state
        .records
        .get_mut(&selected)
        .unwrap()
        .framework_required_count -= 1;
    assert!(state.defer_terminal_cleanup(&selected, 0));
    assert_eq!(state.records[&selected].framework_required_count, 0);
    assert!(state.required_keys.contains(&selected));
    (state, selected)
}

#[test]
fn queued_member_cleanup_refuses_every_budget_then_matches_eager_release() {
    for seed in 0..2 {
        let (mut completed, selected) = queued(seed);
        let mut full = admission(u64::MAX);
        completed.drain_terminal_cleanup(&mut full).unwrap();
        assert!(
            completed.required_keys.is_empty(),
            "the drain must remove the released member"
        );
        assert!(completed.records.is_empty());
        assert_eq!(totals(&completed), (0, 0, 0));
        for k in 0..full.charged_work() {
            let (mut retry, _) = queued(seed);
            let before = totals(&retry);
            let denial = retry.drain_terminal_cleanup(&mut admission(k)).unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            );
            let head = retry
                .pending_cleanup_head
                .as_ref()
                .expect("refused work stays queued");
            assert!(retry.records[head].pending_cleanup_queued);
            if seed == 0 {
                assert_eq!(head.as_ref(), selected.as_ref());
                assert!(retry.required_keys.contains(&selected));
                assert_eq!(totals(&retry), before);
            }
            retry
                .drain_terminal_cleanup(&mut admission(u64::MAX))
                .unwrap();
            assert!(retry.records.is_empty());
            assert!(retry.required_keys.is_empty());
            assert_eq!(totals(&retry), totals(&completed));
        }
    }
}

fn totals(state: &DemandRegistryState) -> (usize, usize, usize) {
    (
        state.required_reserved_bytes,
        state.obligation_reserved_bytes,
        state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire),
    )
}

#[test]
fn queued_member_reader_refuses_before_using_the_retained_set_as_truth() {
    let (state, selected) = queued(0);
    let before = totals(&state);
    let mut full = admission(u64::MAX);
    assert!(state.queued_member_released(&selected, &mut full).unwrap());
    // Two occurrence seeks and the one inspected queued row.
    assert_eq!(full.charged_work(), 9); // Three searches, each two seeks plus one row.
    for k in 0..3 {
        let denial = state
            .queued_member_released(&selected, &mut admission(k))
            .unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert!(state.required_keys.contains(&selected));
        assert!(state.records[&selected].pending_cleanup_queued);
        assert_eq!(totals(&state), before);
        assert!(state
            .queued_member_released(&selected, &mut record_admission())
            .unwrap());
    }
}
