use super::*;

#[test]
fn unchanged_source_after_unrelated_world_work_joins_the_active_demand() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let older = key_with_identity("producer", 5, 1, 90);
    let later_same_source = key_with_identity("producer", 6, 1, 90);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    registry.state.lock().unwrap().records.insert(
        older.clone(),
        DemandRecord {
            source_scope: Some(scope),
            ..record(occurrence(), DemandState::Scheduled, 1)
        },
    );

    let joined = registry
        .admit(
            later_same_source,
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
        )
        .expect("an unchanged semantic source joins its active demand");

    let state = registry.state.lock().unwrap();
    assert_eq!(state.records.len(), 1);
    assert_eq!(joined.key, older);
    assert_eq!(state.records[&older].interests, 2);
}

#[test]
fn semantic_join_deterministically_selects_the_latest_active_generation() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let oldest = key_with_identity("producer", 5, 1, 90);
    let latest = key_with_identity("producer", 6, 1, 90);
    let requested = key_with_identity("producer", 7, 1, 90);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let mut state = registry.state.lock().unwrap();
    for key in [&oldest, &latest] {
        state.records.insert(
            key.clone(),
            DemandRecord {
                source_scope: Some(scope),
                ..record(occurrence(), DemandState::Scheduled, 1)
            },
        );
    }
    drop(state);

    let joined = registry
        .admit(
            requested,
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
        )
        .expect("semantic join chooses one active owner deterministically");

    assert_eq!(joined.key, latest);
    assert_eq!(registry.state.lock().unwrap().records[&latest].interests, 2);
}

#[test]
fn changed_source_can_return_after_its_prior_cycle_was_superseded() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let source_a = key_with_identity("producer", 5, 1, 90);
    let source_b = key_with_identity("producer", 6, 1, 91);
    let returned_a = key_with_identity("producer", 7, 1, 90);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    registry.state.lock().unwrap().records.insert(
        source_a.clone(),
        DemandRecord {
            source_scope: Some(scope),
            ..record(occurrence(), DemandState::Scheduled, 1)
        },
    );

    let _source_b_interest = registry
        .admit(
            source_b.clone(),
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
        )
        .expect("the changed source supersedes its predecessor");
    let returned_interest = registry
        .admit(
            returned_a.clone(),
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
        )
        .expect("a later return is a new source cycle, not the stopped old record");

    let state = registry.state.lock().unwrap();
    assert!(matches!(
        &state.records[&source_a].state,
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert!(matches!(
        &state.records[&source_b].state,
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert_eq!(returned_interest.key, returned_a);
    assert_ne!(source_a, returned_a);
}

#[test]
fn unrelated_world_generation_does_not_order_equal_custody_sources() {
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let older = super::super::BoundOutputSource {
        scope,
        identity: key_with_identity("producer", 5, 1, 90).source,
    };
    let later_same_source = super::super::BoundOutputSource {
        scope,
        identity: key_with_identity("producer", 6, 1, 90).source,
    };

    assert_eq!(
        super::super::source_custody::root_revision_order(&older, &later_same_source),
        Some(std::cmp::Ordering::Equal)
    );
}

#[test]
fn distinct_source_queries_are_not_ordered_or_semantically_joined() {
    let first = key_with_query_identity("producer", 5, 1, 90, 1);
    let second = key_with_query_identity("producer", 6, 1, 90, 2);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let first_root = super::super::BoundOutputSource {
        scope,
        identity: first.source,
    };
    let second_root = super::super::BoundOutputSource {
        scope,
        identity: second.source,
    };

    assert!(!first.same_occurrence(&second));
    assert!(!first.same_semantic_source(&second));
    assert_eq!(first.replacement_order(&second), None);
    assert_eq!(
        super::super::source_custody::root_revision_order(&first_root, &second_root),
        None
    );
}

#[test]
fn recovery_rejoins_the_older_key_that_retains_newer_semantic_custody() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = receipt.product_branch().occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let older = key_with_identity("producer", 5, 1, 90);
    let retained = key_with_identity("producer", 6, 1, 90);
    let newer_non_owner = key_with_identity("producer", 7, 1, 90);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let mut retained_record = record(occurrence, DemandState::Scheduled, 0);
    retained_record.required = true;
    retained_record.source_scope = Some(scope);
    retained_record.source_commits.push(commit.clone());
    let mut state = registry.state.lock().unwrap();
    state.records.insert(older.clone(), retained_record);
    state.records.insert(
        newer_non_owner.clone(),
        DemandRecord {
            required: true,
            source_scope: Some(scope),
            ..record(occurrence, DemandState::Scheduled, 1)
        },
    );
    state.source_custody.insert(
        commit.clone(),
        super::super::SourceCustody {
            occurrence,
            root_kind: super::super::PreparedOutputRootKind::Discovered(
                std::any::TypeId::of::<()>(),
            ),
            // Recovery consults retained ownership and consumed identities; a
            // performed source payload is deliberately unnecessary here.
            source: None,
            discovery: None,
            bound_sources: Some(vec![super::super::BoundOutputSource {
                scope,
                identity: retained.source,
            }]),
            consumed_sources: vec![retained.source],
            retired_sources: Vec::new(),
            retired: None,
            token_count: 1,
            completed: false,
        },
    );
    drop(state);

    let recovered = registry
        .admit(
            retained,
            None,
            scope,
            occurrence,
            super::super::DemandAdmissionKind::Recovery,
            Some(&commit),
        )
        .expect("recovery finds the semantic record that owns retained custody");

    let state = registry.state.lock().unwrap();
    assert_eq!(recovered.key, older);
    assert_eq!(state.records.len(), 2);
    assert_eq!(state.records[&older].interests, 1);
    assert_eq!(state.records[&newer_non_owner].interests, 1);
}
