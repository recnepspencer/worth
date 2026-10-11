//! The deferred queue and the accepted eager cascade end with identical values.
use super::super::super::refreshed_rejoin::infallible;
use super::super::custody_note::differential::value;
use super::*;

fn fixture(
    seed: u64,
    ready: &DemandState,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", seed + 1, 1, 10);
    let newest = key_with_identity("upstream", seed + 2, 1, 11);
    let dependent = key("dependent", seed + 1, 2);
    let copy_ready = || match ready {
        DemandState::Output(output) => {
            DemandState::Output(super::super::super::WorthQueryOutputProgress::new(
                super::super::super::WorthQueryOutputCheckpoint::Ready(
                    match output.checkpoint.as_ref().unwrap() {
                        super::super::super::WorthQueryOutputCheckpoint::Ready(ready) => {
                            ready.clone()
                        }
                        _ => panic!("Ready fixture"),
                    },
                ),
            ))
        }
        _ => panic!("Ready fixture"),
    };
    let mut state = registry.state.lock().unwrap();
    let mut old_row = record(
        occurrence(),
        DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Superseded,
            "newer row",
        )),
        0,
    );
    old_row.framework_required_count = 1;
    state.records.insert(old.clone(), old_row);
    let row = record(occurrence(), copy_ready(), 1);
    let newest_wake = Arc::clone(&row.wake);
    state.records.insert(newest.clone(), row);
    let mut row = record(occurrence(), copy_ready(), 1);
    row.prerequisites.push(Arc::new(old.clone()));
    state.required_reserved_bytes +=
        row.prerequisites.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
    let wake = Arc::clone(&row.wake);
    state.records.insert(dependent.clone(), row);
    for key in [&old, &newest, &dependent] {
        let member = state.prepare_required_member(key).unwrap();
        state.install_required_member(member);
    }
    for neighbor in 0..seed % 5 {
        state.records.insert(
            key("neighbor", seed + 1, 20 + neighbor as u8),
            record(occurrence(), DemandState::Admitted, 1),
        );
    }
    retain_owned_key(&mut state, &dependent);
    retain_owned_key(&mut state, &newest);
    drop(state);
    let owner = interest(&registry, dependent, wake);
    let newest_owner = interest(&registry, newest.clone(), newest_wake);
    (registry, owner, newest_owner, newest)
}

/// Execute the accepted eager cascade body, without the new payment plan.
fn eager(registry: &WorthQueryOutputDemandRegistry) {
    let mut state = registry.state.lock().unwrap();
    let Some(key) = state.closed_cleanup.head.as_ref().map(Arc::clone) else {
        return;
    };
    let note = state.take_closed_note(Arc::clone(&key));
    let retired = note.prune(|state| {
        let retired = infallible(
            state.retire_closed_rows(vec![key.as_ref().clone()], &mut OracleWalk::after_effect()),
        );
        infallible(state.remove_required_member_if_released(&key, &mut OracleWalk::after_effect()));
        retired
    });
    drop(state);
    drop(retired);
}

#[test]
fn registry_drop_cascade_debt_retries_and_matches_the_eager_value_for_16_seeds() {
    let ready = closed_retirement::ready();
    for seed in 0..16 {
        let (registry, owner, newest_owner, newest) = fixture(seed, &ready);
        let (reference, eager_owner, eager_newest, _) = fixture(seed, &ready);
        let (priced, priced_owner, priced_newest, _) = fixture(seed, &ready);
        for (reg, owner, newest_owner) in [
            (&registry, owner, newest_owner),
            (&reference, eager_owner, eager_newest),
            (&priced, priced_owner, priced_newest),
        ] {
            #[cfg(feature = "test-query-execution-observer")]
            let whole = super::super::super::record_map::whole_rows();
            drop(owner);
            drop(newest_owner);
            #[cfg(feature = "test-query-execution-observer")]
            assert_eq!(super::super::super::record_map::whole_rows() - whole, 0);
            let state = reg.state.lock().unwrap();
            assert_eq!(state.closed_cleanup.len, 1);
            assert!(state.required_keys.contains(&newest));
        }
        eager(&reference);
        let mut full = record_admission();
        priced.drain_closed_cleanup_admitted(&mut full).unwrap();
        let work = full.charged_work();
        let before = value(&registry);
        let mut short = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: work - 1,
            maximum_preparation_bytes: u64::MAX,
        });
        let denial = registry
            .drain_closed_cleanup_admitted(&mut short)
            .unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert_eq!(registry.state.lock().unwrap().closed_cleanup.len, 1);
        assert_eq!(value(&registry), before, "short quote, seed {seed}");
        registry
            .drain_closed_cleanup_admitted(&mut record_admission())
            .unwrap();
        assert_eq!(registry.state.lock().unwrap().closed_cleanup.len, 0);
        assert_eq!(value(&registry), value(&reference), "eager, seed {seed}");
        assert_eq!(value(&registry), value(&priced), "retry, seed {seed}");
    }
}

#[test]
fn registry_drop_cascade_foreign_debt_is_afford_or_skip_after_own_admission() {
    let ready = closed_retirement::ready();
    let (baseline, owner, newest_owner, _) = fixture(0, &ready);
    let (registry, deferred_owner, deferred_newest, _) = fixture(0, &ready);
    let scope = super::super::source_custody::real_scope_and_occurrence().0;
    let requested = key("reader", 1, 250);
    let admit = |registry: &WorthQueryOutputDemandRegistry,
                 work: &mut InvalidationEditAdmission| {
        registry
            .admit(
                requested.clone(),
                None,
                scope,
                occurrence(),
                super::super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
                work,
            )
            .unwrap()
    };
    let mut own = record_admission();
    let first = admit(&baseline, &mut own);
    drop(deferred_owner);
    drop(deferred_newest);
    let mut exact = InvalidationEditAdmission::new(CompanionPreflightBudget {
        // A pending foreign note adds the own-occurrence probe: two seeks, no rows.
        maximum_work_visits: own.charged_work() + 2,
        maximum_preparation_bytes: u64::MAX,
    });
    let short = admit(&registry, &mut exact);
    assert_eq!(registry.state.lock().unwrap().closed_cleanup.len, 1);
    let next = admit(&registry, &mut record_admission());
    assert_eq!(registry.state.lock().unwrap().closed_cleanup.len, 0);
    drop(short);
    drop(next);
    drop(first);
    drop(owner);
    drop(newest_owner);
    eager(&baseline);
    assert_eq!(value(&registry), value(&baseline));
}
