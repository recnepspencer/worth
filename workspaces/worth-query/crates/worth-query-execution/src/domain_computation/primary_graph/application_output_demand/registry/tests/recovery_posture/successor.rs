use super::super::super::{ReadyCompletion, WorthQueryCompletedOutputDemand};
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputAdvancement;
use crate::domain_computation::primary_graph::application_output_demand::{
    DemandAdmissionKind, OutputRefreshPredecessor, WorthQueryPerformedOutputDemandSource,
};

#[test]
fn stale_ready_successor_admission_forces_a_new_execution_cycle() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("preserve", 8, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let wake = Arc::new(DemandWake {
        _record_capacity: test_record_capacity(),
        generation: Mutex::new(0),
        changed: Condvar::new(),
    });
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            _record_capacity: test_record_capacity(),
            source_commit_capacity: None,
            source_scope: Some(scope),
            wake: Arc::clone(&wake),
            ..record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Ready(ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
                        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
                        producer_commit_authority: None,
                        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
                        resources: None,
                    })),
                )),
                1,
            )
        },
    );

    let replacement = registry
        .admit(
            demand_key.clone(),
            None,
            scope,
            occurrence,
            DemandAdmissionKind::Ordinary,
            None,
            Some(OutputRefreshPredecessor::Committed(&receipt)),
            &mut record_admission(),
        )
        .expect("the exact stale ready receipt admits its forced successor");
    assert_eq!(replacement.key, demand_key);
    let state = registry.state.lock().unwrap();
    let record = &state.records[&demand_key];
    assert!(matches!(record.state, DemandState::Admitted));
    assert_eq!(
        record
            .successor_of
            .as_ref()
            .map(super::super::super::succession::Succession::predecessor),
        Some(*receipt.idempotency_binding().key_identity())
    );
    assert_eq!(record.interests, 2);
    drop(state);

    // A claimed refresh that stops without publishing returns the row to the
    // Ready it reopened, so the next wave certifies and claims it again. A
    // performed source attached during that refresh is newer than the
    // reopen and stays with the row.
    let mut performed = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let change = performed
        .take_performed_relational_product_change()
        .expect("the real performed application retains its product change");
    let observation = performed
        .committed_product_publication()
        .take_output_demand_observation()
        .expect("the real performed application retains its read basis");
    {
        let mut state = registry.state.lock().unwrap();
        let record = state.records.get_mut(&demand_key).unwrap();
        record.state = DemandState::Running;
        record.performed_source = Some(WorthQueryPerformedOutputDemandSource {
            receipt: performed,
            change: Arc::new(change),
            observation,
            output_source_identity: None,
        });
    }
    registry.relinquish_execution(&replacement);
    let state = registry.state.lock().unwrap();
    let record = &state.records[&demand_key];
    assert!(record.has_cached_ready());
    assert!(record.successor_of.is_none());
    assert!(
        record.performed_source.is_some(),
        "the performed source attached during the refresh is kept"
    );
}

#[test]
fn stopped_ready_record_rejects_successor_without_reviving_custody() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("preserve", 8, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let mut output = WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Ready(
        ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
            producer_commit_authority: None,
            readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
            resources: None,
        }),
    ));
    output.stop(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "newer source owns the cycle",
    ));
    registry.state.lock().unwrap().records.insert(
        demand_key.clone(),
        DemandRecord {
            _record_capacity: test_record_capacity(),
            source_commit_capacity: None,
            source_scope: Some(scope),
            ..record(occurrence, DemandState::Output(output), 1)
        },
    );

    let denial = match registry.admit(
        demand_key.clone(),
        None,
        scope,
        occurrence,
        DemandAdmissionKind::Ordinary,
        None,
        Some(OutputRefreshPredecessor::Committed(&receipt)),
        &mut record_admission(),
    ) {
        Ok(_) => panic!("a stopped ready record was revived"),
        Err(denial) => denial,
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    let state = registry.state.lock().unwrap();
    let record = &state.records[&demand_key];
    assert_eq!(record.interests, 1);
    assert!(matches!(
        &record.state,
        DemandState::Output(output)
            if matches!(output.advancement, WorthQueryOutputAdvancement::Stopped { .. })
                && matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(_)))
    ));
}

#[test]
fn failed_successor_admission_preserves_ready_custody() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let ready_key = key_with_identity("preserve", 8, 1, 80);
    let stale_request = key_with_identity("preserve", 7, 1, 70);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    registry.state.lock().unwrap().records.insert(
        ready_key.clone(),
        DemandRecord {
            _record_capacity: test_record_capacity(),
            source_commit_capacity: None,
            source_scope: Some(scope),
            ..record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
                    WorthQueryOutputCheckpoint::Ready(ReadyCompletion::for_test(WorthQueryCompletedOutputDemand {
                        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
                        producer_commit_authority: None,
                        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
                        resources: None,
                    })),
                )),
                1,
            )
        },
    );

    let denial = match registry.admit(
        stale_request,
        None,
        scope,
        occurrence,
        DemandAdmissionKind::Ordinary,
        None,
        Some(OutputRefreshPredecessor::Committed(&receipt)),
        &mut record_admission(),
    ) {
        Ok(_) => panic!("an older successor admission was accepted"),
        Err(denial) => denial,
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    let state = registry.state.lock().unwrap();
    let record = &state.records[&ready_key];
    assert_eq!(record.interests, 1);
    assert!(matches!(
        &record.state,
        DemandState::Output(output)
            if matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(_)))
    ));
}
