use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
    maintain_primary_runtime_granular_collection_batch, WorthQueryCollectionPatchOperation,
    WorthQueryMaintenanceDenial, WorthQueryMaintenanceStrategy as Strategy,
    WorthQueryPrimaryGranularMaintenanceDenial, WorthQueryPrimaryGranularMaintenanceOutcome,
    WorthQueryPrimaryGranularMaintenancePerformed, WorthQueryPrimaryRuntimeInvalidationBinding,
    WorthQuerySemanticDependencyRole as Role,
};
use worth_query::facade::foundation::WorthQueryEntityIdentity;
use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

use super::{host::FinancialCourtroomWorld, query};

pub fn assert_ordered_portfolio_membership() {
    assert_indexed_maintenance_requires_retained_state();
    assert_off_window_value_survives_other_record_refill();
    let (mut host, mut query, binding) = world();
    assert_value_patch_round_trip(&mut host, &mut query, &binding);
    assert_membership_removal_and_refill(&mut host, &mut query, &binding);
    assert_membership_reentry(&mut host, &mut query, &binding);
    assert_stable_reorder(&mut host, &mut query, &binding);
    assert_window_boundary_refill(&mut host, &mut query, &binding);
}

fn assert_indexed_maintenance_requires_retained_state() {
    let (mut host, mut query, binding) = world();
    let prior = row_identities(&query);
    host.amend_portfolio_rank(2, 3);
    host.portfolio_clock_control.push(2, 11);
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt) = host
        .conditional_clock(&host.portfolio_clock)
        .unwrap()
        .observe()
    else {
        panic!("the current portfolio mutation must be observed")
    };
    let denial = match maintain_primary_runtime_granular_batch(
        &query.live,
        &mut query.workspace,
        &binding,
        receipt.take_granular_invalidation_batch(),
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("indexed work without retained collection state must deny"),
    };
    assert!(matches!(
        denial,
        WorthQueryPrimaryGranularMaintenanceDenial::Maintenance(
            WorthQueryMaintenanceDenial::PerformedEffectUnavailable
        )
    ));
    assert_eq!(row_identities(&query), prior);
}

fn assert_value_patch_round_trip(
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    let prior = row_identities(query);
    host.amend_portfolio_value(2, 5_120);
    host.portfolio_clock_control.push(2, 11);
    let performed = perform("value-forward", host, query, binding);
    assert_roles(&performed, &[Role::ProjectedValue]);
    let patch = performed.deliveries()[0]
        .effect()
        .projection_patch()
        .expect("a value-only change must remain a local field patch");
    assert!(!patch.fields().is_empty());
    assert_eq!(row_identities(query), prior);
    host.amend_portfolio_value(3, 5_100);
    host.portfolio_clock_control.push(3, 12);
    let reversed = perform("value-reverse", host, query, binding);
    assert_roles(&reversed, &[Role::ProjectedValue]);
    assert!(
        !reversed.deliveries()[0]
            .effect()
            .projection_patch()
            .expect("the reverse value change remains a local patch")
            .fields()
            .is_empty(),
        "B -> A must compare against the performed B baseline, not initial A"
    );
}

fn assert_membership_removal_and_refill(
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    let primary = row_identities(query)[0].clone();
    host.amend_portfolio_desk(4, "credit");
    host.portfolio_clock_control.push(4, 13);
    let performed = perform("membership-removal", host, query, binding);
    assert_roles(
        &performed,
        &[
            Role::ProjectedValue,
            Role::SelectionOrMembership,
            Role::Grouping,
        ],
    );
    let patch = indexed(&performed);
    let rates_group = vec![
        worth_foundational::facade::prepare_aspect_value_identity_basis(
            &worth_foundational::facade::AspectValue::String("rates".into()),
        )
        .as_str()
        .to_owned(),
    ];
    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        WorthQueryCollectionPatchOperation::Remove { entity, .. } if entity == &primary
    )));
    assert!(
        patch.operations().iter().any(|operation| matches!(
            operation,
            WorthQueryCollectionPatchOperation::Regroup { entity, from: Some(from), to: None }
                if entity == &primary && from == &rates_group
        )),
        "missing group transition in {:?}",
        patch.operations()
    );
    assert!(!row_identities(query).contains(&primary));
    assert_eq!(
        patch.rows(),
        query.collection.as_ref().unwrap().current_rows()
    );
    assert_matches_fresh(host, query);
}

fn assert_membership_reentry(
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    host.amend_portfolio_desk(5, "rates");
    host.portfolio_clock_control.push(5, 14);
    let performed = perform("membership-reentry", host, query, binding);
    assert!(indexed(&performed).operations().iter().any(|operation| {
        matches!(operation, WorthQueryCollectionPatchOperation::Insert { .. })
    }));
    assert_matches_fresh(host, query);
}

fn assert_stable_reorder(
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    let prior = row_identities(query);
    let primary = prior[0].clone();
    host.amend_portfolio_rank(6, 3);
    host.portfolio_clock_control.push(6, 15);
    let performed = perform("ordering", host, query, binding);
    assert_roles(
        &performed,
        &[Role::ProjectedValue, Role::Ordering, Role::WindowBoundary],
    );
    let patch = indexed(&performed);
    assert!(
        patch.operations().iter().any(|operation| matches!(
            operation,
            WorthQueryCollectionPatchOperation::Move { row, from: 0, to: 1 }
                if row.entity_identity() == &primary
        )),
        "missing stable move in {:?}",
        patch.operations()
    );
    assert_eq!(row_identities(query)[1], primary);
    assert_matches_fresh(host, query);
}

fn assert_window_boundary_refill(
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    let primary = row_identities(query)[1].clone();
    host.amend_portfolio_rank(7, 100_000);
    host.portfolio_clock_control.push(7, 16);
    let performed = perform("window", host, query, binding);
    let patch = indexed(&performed);
    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        WorthQueryCollectionPatchOperation::Remove { entity, .. } if entity == &primary
    )));
    assert!(patch.operations().iter().any(|operation| matches!(
        operation,
        WorthQueryCollectionPatchOperation::Insert { row, .. }
            if row.entity_identity() != &primary
    )));
    assert!(!row_identities(query).contains(&primary));
    assert_eq!(
        patch.rows(),
        query.collection.as_ref().unwrap().current_rows()
    );
    assert!(performed.maintenance_counters().window_rows() <= 5);
    assert_matches_fresh(host, query);
}

fn assert_off_window_value_survives_other_record_refill() {
    let (mut host, mut query, binding) = world();
    let sibling = WorthQueryEntityIdentity::from_bridge_record_projection(
        host.sibling_curve_record_identity(),
    );
    assert!(!row_identities(&query).contains(&sibling));
    host.amend_sibling_portfolio_value(2, 5_200);
    host.sibling_portfolio_gate.release();
    host.sibling_portfolio_clock_control.push(2, 11);
    let off_window = perform_sibling("off-window-value", &mut host, &mut query, &binding);
    assert_roles(&off_window, &[Role::ProjectedValue]);
    host.amend_portfolio_desk(3, "credit");
    host.portfolio_clock_control.push(3, 12);
    let refill = perform("other-record-refill", &mut host, &mut query, &binding);
    assert!(indexed(&refill).operations().iter().any(|operation| {
        matches!(
            operation, WorthQueryCollectionPatchOperation::Insert { row, .. }
                if row.entity_identity() == &sibling
        )
    }));
    assert!(indexed(&refill).collection_facts().iter().any(|fact| {
        fact.row_identity() == &sibling
            && fact.native_value().scalar()
                == Some(&worth_foundational::facade::AspectValue::UInt64(5_200))
    }));
    assert_matches_fresh(&host, &query);
}

fn perform_sibling(
    scenario_step: &str,
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) -> WorthQueryPrimaryGranularMaintenancePerformed {
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt) = host
        .conditional_clock(&host.sibling_portfolio_clock)
        .unwrap()
        .observe()
    else {
        panic!("the sibling portfolio mutation must be observed")
    };
    let outcome = maintain_primary_runtime_granular_collection_batch(
        &query.live,
        query
            .collection
            .as_mut()
            .expect("portfolio collection state"),
        &mut query.workspace,
        binding,
        receipt.take_granular_invalidation_batch(),
    )
    .expect("the sibling portfolio field change must admit maintenance");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the {scenario_step} sibling change must perform Query-owned maintenance")
    };
    performed
}

fn world() -> (
    FinancialCourtroomWorld,
    query::FinancialQueryWorld,
    WorthQueryPrimaryRuntimeInvalidationBinding,
) {
    let host = FinancialCourtroomWorld::publish_portfolio();
    let query = query::build_portfolio_with_unrelated_rows(&host, 64);
    assert_rank_is_private_maintenance_support(&query);
    drop(
        host.conditional_clock(&host.sibling_portfolio_clock)
            .unwrap(),
    );
    assert!(matches!(
        host.conditional_clock(&host.portfolio_clock)
            .unwrap()
            .observe(),
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    host.portfolio_gate.release();
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );
    (host, query, binding)
}

fn assert_rank_is_private_maintenance_support(query: &query::FinancialQueryWorld) {
    let facts = query.live.snapshot().authority().facts();
    let exposes_rank = facts
        .display_fields()
        .iter()
        .chain(facts.derived_fields())
        .filter_map(|fact| fact.field_path().canonical_field_path())
        .flat_map(|path| path.fields())
        .any(|field| field.as_str() == "PortfolioRankField");
    assert!(
        !exposes_rank,
        "rank must remain private ordering/window support, not public projection"
    );
}

fn perform(
    scenario_step: &str,
    host: &mut FinancialCourtroomWorld,
    query: &mut query::FinancialQueryWorld,
    binding: &WorthQueryPrimaryRuntimeInvalidationBinding,
) -> WorthQueryPrimaryGranularMaintenancePerformed {
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt) = host
        .conditional_clock(&host.portfolio_clock)
        .unwrap()
        .observe()
    else {
        panic!("the current portfolio mutation must be observed")
    };
    let outcome = maintain_primary_runtime_granular_collection_batch(
        &query.live,
        query
            .collection
            .as_mut()
            .expect("portfolio collection state"),
        &mut query.workspace,
        binding,
        receipt.take_granular_invalidation_batch(),
    )
    .expect("the exact portfolio field change must admit maintenance");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the {scenario_step} portfolio change must perform Query-owned maintenance")
    };
    assert_eq!(performed.shared_execution_count(), 1);
    assert_eq!(performed.consumer_publication_count(), 1);
    assert!(performed.maintenance_counters().prior_field_comparisons() <= 3);
    performed
}

fn assert_roles(performed: &WorthQueryPrimaryGranularMaintenancePerformed, required: &[Role]) {
    let delivery = &performed.deliveries()[0];
    for role in required {
        assert!(delivery.roles().contains(role), "missing role {role:?}");
    }
    let expected = required.iter().filter_map(|role| match role {
        Role::ProjectedValue | Role::ConditionalEligibilityOrSemanticCleanliness => {
            Some(Strategy::LocalProjectionPatch)
        }
        Role::SelectionOrMembership => Some(Strategy::MembershipSplice),
        Role::Ordering | Role::Grouping => Some(Strategy::StableReorderOrRegroup),
        Role::WindowBoundary => Some(Strategy::WindowRefill),
        _ => None,
    });
    for strategy in expected {
        assert!(delivery.strategies().contains(&strategy));
    }
}

fn indexed(
    performed: &WorthQueryPrimaryGranularMaintenancePerformed,
) -> &worth_query::facade::domain::WorthQueryPerformedIndexedLivePatch {
    performed.deliveries()[0]
        .effect()
        .indexed_live_patch()
        .expect("collection roles must publish an applied indexed patch")
}

fn row_identities(query: &query::FinancialQueryWorld) -> Vec<WorthQueryEntityIdentity> {
    query
        .collection
        .as_ref()
        .expect("portfolio collection state")
        .current_rows()
        .iter()
        .map(|row| row.entity_identity().clone())
        .collect()
}

fn assert_matches_fresh(
    host: &FinancialCourtroomWorld,
    incrementally_maintained: &query::FinancialQueryWorld,
) {
    let fresh = query::build_portfolio_with_unrelated_rows(host, 64);
    assert_eq!(
        row_identities(incrementally_maintained),
        row_identities(&fresh),
        "incremental collection state must equal a fresh authoritative query"
    );
}
