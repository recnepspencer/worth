use super::*;
use crate::domain_computation::primary_graph::application_output_demand::{
    BoundOutputSource, PreparedOutputRootKind, WorthQueryOutputCheckpoint,
    WorthQueryPerformedOutputDemandSource,
};
use crate::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn an_open_ordinary_interest_is_required_until_its_last_handle_closes() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let output = key("ordinary-interest", 1, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let admit = || {
        registry
            .admit(
                output.clone(),
                None,
                scope,
                occurrence(),
                super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
            )
            .unwrap()
            .0
    };
    let first = admit();
    let second = admit();
    drop(first);
    assert!(registry
        .state
        .lock()
        .unwrap()
        .required_keys
        .contains(&output));
    drop(second);
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&output));
    assert!(state.required_keys.is_empty());
}

#[test]
fn closing_a_required_interest_removes_it_from_the_required_set() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let output = key("required-interest", 1, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let first = registry
        .admit(
            output.clone(),
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Required,
            None,
            None,
        )
        .expect("a source-backed required interest admits")
        .0;
    assert!(registry.state.lock().unwrap().records[&output].is_required());
    assert!(registry
        .state
        .lock()
        .unwrap()
        .required_keys
        .contains(&output));
    drop(first);
    {
        let state = registry.state.lock().unwrap();
        assert!(!state.records.contains_key(&output));
        assert!(state.required_keys.is_empty());
    }

    let reopened = registry
        .admit(
            output.clone(),
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Required,
            None,
            None,
        )
        .expect("closing an interest permits exact reopening")
        .0;
    assert!(registry.state.lock().unwrap().records[&output].is_required());
    assert!(registry
        .state
        .lock()
        .unwrap()
        .required_keys
        .contains(&output));
    drop(reopened);
    assert!(!registry.state.lock().unwrap().records.contains_key(&output));
}

#[test]
fn performed_source_obligation_survives_handle_close_until_occurrence_retirement() {
    let mut receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let occurrence = receipt.product_branch().occurrence();
    let scope = receipt.principal_scope().scope();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let change = receipt
        .take_performed_relational_product_change()
        .expect("the real performed application retains its product change");
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation()
        .expect("the real performed application retains its read basis");
    let registry = WorthQueryOutputDemandRegistry::default();
    let preparation = registry.begin_source_preparation(occurrence);
    registry
        .retain_performed_source(
            WorthQueryPerformedOutputDemandSource {
                receipt,
                change: Arc::new(change),
                observation,
                output_source_identity: None,
            },
            &preparation,
            PreparedOutputRootKind::Required(TypeId::of::<()>()),
            None,
        )
        .expect("the performed source enters native custody");
    let output = key("performed-required", 1, 1);
    registry
        .ensure_prepared_output_source_bound(
            &commit,
            BoundOutputSource {
                scope,
                identity: output.source.clone(),
            },
        )
        .expect("the exact output source is bound");
    let interest = registry
        .admit_performed(output.clone(), &commit, scope, occurrence)
        .expect("the bound performed source admits one output obligation");
    assert_eq!(
        registry.state.lock().unwrap().records[&output]
            .performed_obligations
            .len(),
        1
    );
    drop(interest);
    {
        let state = registry.state.lock().unwrap();
        let record = &state.records[&output];
        assert_eq!(record.required_interests, 0);
        assert!(
            record.is_required(),
            "the performed operation still requires its output"
        );
        assert_eq!(record.performed_obligations[0].source_commit, commit);
        assert!(state.required_keys.contains(&output));
    }
    registry.release_product_occurrence(occurrence);
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&output));

    assert!(state.required_keys.is_empty());
}

#[test]
fn settlement_releases_only_the_ready_authority_it_accepted() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let replacement = receipt
        .clone()
        .with_provider_runtime_instance_id_for_test(u64::MAX);
    let output = key("ready-replaced", 1, 1);
    let expected = super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt);
    let actual = super::super::WorthQueryAcceptedOutputAuthority::Committed(replacement.clone());
    let completion = super::super::WorthQueryCompletedOutputDemand {
        authority: actual.clone(),
        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
    };
    let mut retained = record(
        occurrence(),
        DemandState::Output(super::super::WorthQueryOutputProgress::new(
            WorthQueryOutputCheckpoint::Ready(super::super::ReadyCompletion::for_test(completion)),
        )),
        1,
    );
    retained.performed_obligations = vec![super::super::PerformedOutputObligation {
        source_commit: replacement
            .committed_product_publication()
            .composite_commit()
            .clone(),
        source: output.source.clone(),
    }];

    let wake = Arc::clone(&retained.wake);
    let registry = WorthQueryOutputDemandRegistry::default();
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(output.clone(), retained);
    }
    let interest = interest(&registry, output.clone(), wake);
    let denial = registry
        .finish_settlement(&interest, &expected)
        .expect_err("a replaced Ready authority cannot discharge its performed source");
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    assert_eq!(
        registry.state.lock().unwrap().records[&output]
            .performed_obligations
            .len(),
        1
    );
    registry
        .finish_settlement(&interest, &actual)
        .expect("the exact accepted Ready authority may settle its obligation");
    let state = registry.state.lock().unwrap();
    assert!(state.records[&output].performed_obligations.is_empty());
}
