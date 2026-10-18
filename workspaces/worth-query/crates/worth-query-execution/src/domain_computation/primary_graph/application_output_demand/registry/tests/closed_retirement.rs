use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint;

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn ready(
) -> DemandState {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let completion = super::super::WorthQueryCompletedOutputDemand {
        authority: super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
        resources: None,
    };
    DemandState::Output(super::super::WorthQueryOutputProgress::new(
        WorthQueryOutputCheckpoint::Ready(super::super::ReadyCompletion::for_test(completion)),
    ))
}

/// A closed dependent claims a superseded row whose occurrence's newest row
/// is in `newest_state`. Returns the registry, the dependent's interest, and
/// the superseded and newest keys.
fn claim_on_superseded(
    newest_state: DemandState,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey,
    WorthQueryOutputDemandKey,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", 5, 1, 90);
    let newest = key_with_identity("upstream", 6, 1, 91);
    let dependent = key_with_identity("dependent", 5, 2, 50);
    let dependent_record = DemandRecord {
        prerequisites: vec![Arc::new(old.clone())],
        ..record(occurrence(), ready(), 1)
    };
    let wake = Arc::clone(&dependent_record.wake);
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(
            old.clone(),
            DemandRecord {
                framework_required_count: 1,
                ..record(
                    occurrence(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "",
                    )),
                    0,
                )
            },
        );
        state
            .records
            .insert(newest.clone(), record(occurrence(), newest_state, 1));
        state.required_keys.insert(Arc::new(newest.clone()));
        state.records.insert(dependent.clone(), dependent_record);
        retain_owned_key(&mut state, &dependent);
    }
    let interest = interest(&registry, dependent, wake);
    (registry, interest, old, newest)
}

#[test]
fn a_closing_dependent_keeps_its_claim_while_the_newest_row_still_refreshes() {
    let (registry, interest, old, newest) = claim_on_superseded(DemandState::Running);
    let dependent = interest.key.clone();
    drop(interest);
    let state = registry.state.lock().unwrap();
    // The superseded Ready stays for a World that supersedes the refresh.
    assert_eq!(state.records[&old].framework_required_count, 1);
    assert_eq!(state.records[&newest].framework_required_count, 0);
    assert_eq!(claims(&state.records[&dependent]), vec![old]);
}

#[test]
fn a_reclaim_keeps_a_superseded_row_whose_claim_did_not_move() {
    let (registry, interest, old, newest) = claim_on_superseded(ready());
    let dependent = interest.key.clone();
    {
        let mut state = registry.state.lock().unwrap();
        // The newest row is no required member: the claim has no key to move to.
        state.required_keys.clear();
        state.required_budget_bytes = 0;
    }
    registry
        .reclaim_cached_rows(1, &mut record_admission())
        .unwrap();
    {
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&old].framework_required_count, 1);
        assert_eq!(claims(&state.records[&dependent]), vec![old.clone()]);
    }
    // The dependent's own retirement releases the claim on a row still there.
    drop(interest);
    registry
        .reclaim_cached_rows(1, &mut record_admission())
        .unwrap();
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&dependent));
    assert!(!state.records.contains_key(&old));
    assert!(state.records.contains_key(&newest));
}

#[test]
fn an_unpublished_refresh_its_last_owner_lets_go_gives_the_occurrence_back() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let replaced = key_with_identity("upstream", 5, 1, 90);
    let refresh = key_with_identity("upstream", 6, 1, 91);
    let mut replaced_record = DemandRecord {
        framework_required_count: 1,
        ..record(occurrence(), ready(), 0)
    };
    let DemandState::Output(output) = &mut replaced_record.state else {
        unreachable!("the replaced row holds a Ready");
    };
    output.finish_stopped(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "",
    ));
    let refresh_record = DemandRecord {
        successor_of: Some(super::super::succession::Succession::new([0; 32])),
        ..record(occurrence(), DemandState::Scheduled, 1)
    };
    let wake = Arc::clone(&refresh_record.wake);
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(replaced.clone(), replaced_record);
        state.records.insert(refresh.clone(), refresh_record);
    }
    drop(interest(&registry, refresh.clone(), wake));
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&refresh));
    // The Ready the refresh replaced answers for the occurrence again.
    assert!(
        matches!(&state.records[&replaced].state, DemandState::Output(output)
        if matches!(output.advancement(), super::super::WorthQueryOutputAdvancement::Idle))
    );
}

fn claims(record: &DemandRecord) -> Vec<WorthQueryOutputDemandKey> {
    record
        .prerequisites
        .iter()
        .map(|claimed| claimed.as_ref().clone())
        .collect()
}

#[test]
fn a_closing_dependent_moves_its_claim_once_the_newest_row_is_ready() {
    let (registry, interest, old, newest) = claim_on_superseded(ready());
    let dependent = interest.key.clone();
    let reader = key("unrelated-reader", 1, 3);
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(
            reader.clone(),
            record(occurrence(), DemandState::Admitted, 1),
        );
        let member = state.prepare_required_member(&reader).unwrap();
        state.install_required_member(member);
    }
    drop(interest);
    let before = {
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&old].framework_required_count, 1);
        assert_eq!(state.records[&newest].framework_required_count, 0);
        assert_eq!(claims(&state.records[&dependent]), vec![old.clone()]);
        assert!(state.records[&dependent].closed_cleanup.queued());
        (
            state.required_reserved_bytes,
            state.obligation_reserved_bytes,
            state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
        )
    };
    let scope = source_custody::real_scope_and_occurrence().0;
    // Joining an already funded reader needs no preparation storage. The
    // foreign transfer does: its refused quote must not refuse this admission.
    let mut small = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: Some(1_000_000),
        maximum_preparation_bytes: 0,
    });
    let _reader = registry
        .admit(
            reader.clone(),
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
            None,
            &mut small,
        )
        .unwrap();
    {
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&old].framework_required_count, 1);
        assert_eq!(state.records[&newest].framework_required_count, 0);
        assert_eq!(claims(&state.records[&dependent]), vec![old.clone()]);
        assert!(state.records[&dependent].closed_cleanup.queued());
        assert_eq!(
            (
                state.required_reserved_bytes,
                state.obligation_reserved_bytes,
                state
                    .required_custody_retained_bytes
                    .load(std::sync::atomic::Ordering::Acquire)
            ),
            before
        );
    }
    let _funded_reader = registry
        .admit(
            reader,
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
            None,
            &mut record_admission(),
        )
        .unwrap();
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&old));
    assert_eq!(state.records[&newest].framework_required_count, 1);
    assert_eq!(claims(&state.records[&dependent]), vec![newest]);
}
