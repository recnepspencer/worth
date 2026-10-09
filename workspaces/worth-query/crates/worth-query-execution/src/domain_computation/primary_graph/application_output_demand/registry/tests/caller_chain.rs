use super::closed_retirement::ready;
use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop;

fn superseded() -> DemandState {
    DemandState::Failed(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "",
    ))
}

/// A dependent its caller holds, claiming a closed upstream row in
/// `upstream_state` that `upstream_interests` other demands hold, beside
/// `others`. Returns whether another demand holds custody for that caller.
fn another_demand_holds_custody(
    upstream_state: DemandState,
    upstream_interests: usize,
    others: Vec<(WorthQueryOutputDemandKey, DemandRecord)>,
) -> Result<bool, CompanionPreflightStop> {
    let registry = WorthQueryOutputDemandRegistry::default();
    let upstream = key_with_identity("upstream", 5, 1, 90);
    let dependent = key_with_identity("dependent", 5, 2, 50);
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(
            upstream.clone(),
            DemandRecord {
                framework_required_count: 1,
                ..record(occurrence(), upstream_state, upstream_interests)
            },
        );
        state.records.insert(
            dependent.clone(),
            DemandRecord {
                prerequisites: vec![Arc::new(upstream)],
                ..record(occurrence(), ready(), 1)
            },
        );
        state.records.extend(others);
    }
    registry.another_demand_holds_custody(&dependent, &mut record_admission())
}

#[test]
fn a_caller_alone_with_its_chain_waits_for_no_other_demand() {
    // A closed cached row outside the chain is nobody's to release.
    let cached = (
        key_with_identity("unrelated", 5, 3, 70),
        record(occurrence(), ready(), 0),
    );
    assert_eq!(
        another_demand_holds_custody(ready(), 0, vec![cached]),
        Ok(false)
    );
}

#[test]
fn a_demand_open_outside_the_chain_holds_custody_its_close_frees() {
    let open = (
        key_with_identity("unrelated", 5, 3, 70),
        record(occurrence(), ready(), 1),
    );
    assert_eq!(
        another_demand_holds_custody(ready(), 0, vec![open]),
        Ok(true)
    );
}

#[test]
fn a_demand_open_on_a_chain_row_counts_only_while_that_row_is_stale_or_refreshing() {
    // The caller's advance needs the current upstream held whoever is open
    // on it.
    assert_eq!(
        another_demand_holds_custody(ready(), 1, Vec::new()),
        Ok(false)
    );
    // A stale upstream its owner has yet to let go, and a refresh of the
    // upstream its owner has yet to finish.
    assert_eq!(
        another_demand_holds_custody(superseded(), 1, Vec::new()),
        Ok(true)
    );
    let refresh = (
        key_with_identity("upstream", 6, 1, 91),
        record(occurrence(), DemandState::Scheduled, 1),
    );
    assert_eq!(
        another_demand_holds_custody(superseded(), 0, vec![refresh]),
        Ok(true)
    );
    // The stale upstream only the caller's own claim holds waits for nobody.
    assert_eq!(
        another_demand_holds_custody(superseded(), 0, Vec::new()),
        Ok(false)
    );
}

#[test]
fn a_caller_yet_to_hold_a_row_waits_for_any_open_demand() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let key = key_with_identity("unrelated", 5, 3, 70);
    registry
        .state
        .lock()
        .unwrap()
        .records
        .insert(key.clone(), record(occurrence(), ready(), 0));
    assert_eq!(
        registry.a_demand_holds_custody(&mut record_admission()),
        Ok(false)
    );
    registry
        .state
        .lock()
        .unwrap()
        .records
        .get_mut(&key)
        .unwrap()
        .interests = 1;
    assert_eq!(
        registry.a_demand_holds_custody(&mut record_admission()),
        Ok(true)
    );
}

#[test]
fn a_walk_the_request_cannot_pay_for_establishes_nothing() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let dependent = key_with_identity("dependent", 5, 2, 50);
    registry
        .state
        .lock()
        .unwrap()
        .records
        .insert(dependent.clone(), record(occurrence(), ready(), 1));
    let mut short = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 1,
        maximum_preparation_bytes: 0,
    });
    assert!(matches!(
        registry.another_demand_holds_custody(&dependent, &mut short),
        Err(CompanionPreflightStop::WorkExhausted { .. })
    ));
}

#[test]
fn closed_cached_population_does_not_make_a_lone_caller_retry_on_walk_exhaustion() {
    for unrelated in [0, 1_000] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let caller = key_with_identity("caller", 5, 2, 50);
        {
            let mut state = registry.state.lock().unwrap();
            state
                .records
                .insert(caller.clone(), record(occurrence(), ready(), 1));
            for number in 0..unrelated {
                let cached =
                    key_with_identity(&format!("cached-{number}"), 5, 3, number as u64 + 70);
                state
                    .records
                    .insert(cached, record(occurrence(), ready(), 0));
            }
        }
        assert_eq!(
            registry.another_demand_holds_custody(&caller, &mut record_admission()),
            Ok(false)
        );
        let mut exhausted = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: 0,
            maximum_preparation_bytes: 0,
        });
        assert!(
            matches!(
                registry.another_demand_holds_custody(&caller, &mut exhausted),
                Err(CompanionPreflightStop::WorkExhausted { .. })
            ),
            "a refused global question reports its own work stop"
        );
    }
}
