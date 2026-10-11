//! Interest closure retires undelivered output, with its own funded custody.
use super::super::{
    PreparedReadyBacking, RunningSelectedExecution, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandAdvanceAdmission as Admission, WorthQueryPendingOutputDelivery,
};
use super::*;

pub(super) fn admit(
    registry: &WorthQueryOutputDemandRegistry,
    key: &WorthQueryOutputDemandKey,
) -> WorthQueryOutputDemandInterest {
    registry.admit(key.clone(), None,
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1)),
        occurrence(), super::super::DemandAdmissionKind::Ordinary, None, None,
        &mut record_admission()).unwrap()
}

pub(super) fn publish(
    registry: &WorthQueryOutputDemandRegistry,
    interest: &WorthQueryOutputDemandInterest,
    receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
) {
    assert!(matches!(
        registry.begin(interest),
        Admission::Schedule(None)
    ));
    registry.finish_scheduling(
        interest,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
    );
    let finish = registry
        .prepare_selected_execution_finish(interest, &mut record_admission())
        .unwrap();
    assert!(matches!(
        registry.begin(interest),
        Admission::Execute { .. }
    ));
    let backing =
        PreparedReadyBacking::prepare(&registry.state.lock().unwrap(), &mut record_admission(), 0)
            .unwrap();
    RunningSelectedExecution::new(finish)
        .publish(WorthQueryOutputCheckpoint::Published {
            receipt,
            delivery: WorthQueryPendingOutputDelivery::NoChange,
            ready_backing: backing,
        })
        .unwrap_or_else(|_| panic!("the real prepared finish publishes"));
}

pub(super) fn balances(registry: &WorthQueryOutputDemandRegistry) -> (usize, usize, usize) {
    let state = registry.state.lock().unwrap();
    (
        state.required_reserved_bytes,
        state.obligation_reserved_bytes,
        state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire),
    )
}

#[test]
fn last_published_interest_refunds_its_own_claims_and_reservations() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let upstream_key = key("held-upstream", 1, 1);
        let upstream = admit(&registry, &upstream_key);
        let baseline = balances(&registry);
        let output_key = key("undelivered", 1, 2);
        let output = admit(&registry, &output_key);
        publish(&registry, &output, receipt);
        {
            // The local registry fixture declares one already funded consumed edge.
            // Native certification is not under test; release of that edge is.
            let mut state = registry.state.lock().unwrap();
            let claims = vec![Arc::new(upstream_key.clone())];
            state.required_reserved_bytes +=
                claims.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            state.records.get_mut(&output_key).unwrap().prerequisites = claims;
            state
                .records
                .get_mut(&upstream_key)
                .unwrap()
                .framework_required_count += 1;
        }
        assert!(balances(&registry).0 > baseline.0);
        drop(output);
        assert_eq!(
            balances(&registry),
            baseline,
            "all three reserved totals return to the baseline"
        );
        let state = registry.state.lock().unwrap();
        assert!(!state.records.contains_key(&output_key));
        assert_eq!(state.records[&upstream_key].framework_required_count, 0);
        assert_eq!(state.records[&upstream_key].interests, 1);
        drop(state);
        drop(upstream);
    });
}

#[test]
fn one_remaining_interest_keeps_the_undelivered_publication() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let _occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let closed_baseline = balances(&registry);
        let upstream_key = key("remaining-interest-upstream", 1, 1);
        let upstream = admit(&registry, &upstream_key);
        let output_key = key("two-interests", 1, 1);
        let first = admit(&registry, &output_key);
        let second = admit(&registry, &output_key);
        // The admitted rows already fund their index; the extra custody is
        // one prerequisite claim and one ready backing declared below.
        let baseline = balances(&registry);
        publish(&registry, &first, receipt);
        let claims = vec![Arc::new(upstream_key.clone())];
        let claim_bytes = claims.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
        {
            let mut state = registry.state.lock().unwrap();
            state.required_reserved_bytes += claim_bytes;
            state.records.get_mut(&output_key).unwrap().prerequisites = claims.clone();
            state
                .records
                .get_mut(&upstream_key)
                .unwrap()
                .framework_required_count += 1;
        }
        let before = balances(&registry);
        assert_eq!(
            before,
            (
                baseline.0 + claim_bytes,
                baseline.1,
                baseline.2 + PreparedReadyBacking::retained_bytes()
            ),
            "one prerequisite reservation and one funded Ready backing are held"
        );
        drop(first);
        assert_eq!(balances(&registry), before);
        {
            let state = registry.state.lock().unwrap();
            assert_eq!(state.records[&output_key].interests, 1);
            assert_eq!(
                state.records[&output_key].prerequisites, claims,
                "the remaining interest still owns the exact prerequisite claim"
            );
            assert_eq!(state.records[&upstream_key].framework_required_count, 1);
            assert_eq!(state.required_reserved_bytes, before.0);

            assert!(
                matches!(&state.records[&output_key].state, DemandState::Output(output)
                if matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Published { .. })))
            );
        }
        drop(second);
        assert!(!registry
            .state
            .lock()
            .unwrap()
            .records
            .contains_key(&output_key));
        assert_eq!(
            registry.state.lock().unwrap().records[&upstream_key].framework_required_count,
            0
        );
        drop(upstream);
        assert_eq!(balances(&registry), closed_baseline);
    });
}
