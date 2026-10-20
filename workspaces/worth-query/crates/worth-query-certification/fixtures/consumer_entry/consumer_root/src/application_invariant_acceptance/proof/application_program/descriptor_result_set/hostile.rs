//! Advanced substitutions encounter the real original unpublished fixed owner.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRecoveryRequestDenial as Denial, WorthQueryProgramSourceRecoveryProgress,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryManagedApplicationRecoveryDenial, WorthQueryOutputDemandDenialKind,
    WorthQuerySourceExpectationDenialKind,
};
use worth_query_host::facade::runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause};
use worth_query_topology_entry::PlanarOutputRead;

#[test]
fn advanced_result_set_substitutions_preserve_original_partial() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let original = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let substituted = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    assert_ne!(original.rows()[0].body_key, substituted.rows()[0].body_key);
    let original_set = original.result_set_observation().unwrap().clone();
    let original_row = original.observed_sources()[0].clone();
    let substituted_set = substituted.result_set_observation().unwrap().clone();
    drop(original);
    drop(substituted);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    let key = 10_078;
    reset_planar_source_adjustment_contacts(&input);
    world
        .application
        .with_host_advancement(|phase| {
            world.application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let outcome = request
        .advanced()
        .mutate(input.clone())
        .preconditions(conditions("anchor-a"))
        .expect_result_set(original_set.clone())
        .idempotency(&key)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut recovery = match outcome {
        WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(recovery) => recovery,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("native no-effect outcome: {outcome:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { cleanup, .. } => {
            panic!(
                "this unarmed court met genuine managed cleanup refusal: {:?}",
                cleanup.failure()
            )
        }
        WorthQueryApplicationPerformedMutationOutcome::Blocked(blocked) => {
            panic!("native blocked outcome: {:?}", blocked.outcome())
        }
        _ => panic!("actual append failure must return the native required-source owner"),
    };
    assert_eq!(
        recovery.initial_cause(),
        ProductUnpublishedCause::SettlementPending
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    drop(principal);
    drop(scope);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    macro_rules! original_request {
        ($conditions:expr, $key:expr) => {
            request
                .advanced()
                .mutate(input.clone())
                .preconditions($conditions)
                .expect_result_set(original_set.clone())
                .idempotency($key)
        };
    }
    // A default is a different original contract even when the current fact is true.
    for changed in [Conditions::new(), conditions("anchor-b")] {
        let denial = original_request!(changed, &key)
            .recover_unpublished_required_in_program(&mut recovery, &world.application)
            .expect_err("changed or omitted original conditions must refuse");
        assert!(
            matches!(
                denial,
                Denial::Recovery(WorthQueryManagedApplicationRecoveryDenial::BindingMismatch)
            ),
            "native original binding refusal: {denial:?}"
        );
        assert!(recovery.performed().is_none());
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        assert_eq!(
            world
                .application
                .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
                .unwrap()
                .rows()
                .len(),
            1
        );
    }
    let wrong_key = key + 1;
    let denial = original_request!(conditions("anchor-a"), &wrong_key)
        .recover_unpublished_required_in_program(&mut recovery, &world.application)
        .expect_err("a replacement key must not take original custody");
    assert!(
        matches!(denial, Denial::Recovery(WorthQueryManagedApplicationRecoveryDenial::Demand(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::ForeignSource),
        "native keyed refusal: {denial:?}"
    );
    let denial = request
        .advanced()
        .mutate(input.clone())
        .preconditions(conditions("anchor-a"))
        .expect_source(original_row)
        .idempotency(&key)
        .recover_unpublished_required_in_program(&mut recovery, &world.application)
        .expect_err("a row cannot approximate the completed set");
    assert!(
        matches!(denial, Denial::Recovery(WorthQueryManagedApplicationRecoveryDenial::Demand(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::ForeignSource),
        "native form refusal: {denial:?}"
    );
    let denial = request
        .advanced()
        .mutate(input.clone())
        .preconditions(conditions("anchor-a"))
        .expect_result_set(substituted_set)
        .idempotency(&key)
        .recover_unpublished_required_in_program(&mut recovery, &world.application)
        .expect_err("another genuine selection cannot replace the original set");
    assert!(
        matches!(denial, Denial::Request(WorthQueryApplicationRequestMutationDenial::SourceExpectation(ref cause)) if cause.kind() == WorthQuerySourceExpectationDenialKind::ForeignModel),
        "real selected-model refusal: {denial:?}"
    );
    assert!(recovery.performed().is_none());
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert_eq!(
        world
            .application
            .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .len(),
        1
    );

    assert_eq!(
        original_request!(conditions("anchor-a"), &key)
            .recover_unpublished_required_in_program(&mut recovery, &world.application)
            .unwrap(),
        WorthQueryProgramSourceRecoveryProgress::Performed
    );
    assert!(recovery.performed().is_some());
    let current = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(current.rows()[0].y, length(2));
    let (denial, retained) = request
        .advanced()
        .mutate(input.clone())
        .preconditions(conditions("anchor-a"))
        .expect_source(current.observed_sources()[0].clone())
        .idempotency(&key)
        .promote_recovered_required_outputs(recovery, &world.application)
        .err()
        .expect("a newer row cannot replace original completed-set meaning");
    assert!(
        matches!(denial, Denial::Recovery(WorthQueryManagedApplicationRecoveryDenial::Demand(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::ForeignSource),
        "newer form refusal: {denial:?}"
    );
    recovery = retained;
    assert!(recovery.performed().is_some());
    let newer_other = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    let (denial, retained) = request
        .advanced()
        .mutate(input.clone())
        .preconditions(conditions("anchor-a"))
        .expect_result_set(newer_other.result_set_observation().unwrap().clone())
        .idempotency(&key)
        .promote_recovered_required_outputs(recovery, &world.application)
        .err()
        .expect("a newer different selection cannot replace the original completed set");
    assert!(
        matches!(denial, Denial::Request(WorthQueryApplicationRequestMutationDenial::SourceExpectation(ref cause)) if cause.kind() == WorthQuerySourceExpectationDenialKind::ForeignModel),
        "newer selected-model refusal: {denial:?}"
    );
    recovery = retained;
    assert!(recovery.performed().is_some());
    drop(newer_other);
    // Newness alone is not substitution: membership is unchanged after a value edit.
    let (preparation, cause, performed, prior_cleanup) = request
        .advanced()
        .mutate(input.clone())
        .preconditions(conditions("anchor-a"))
        .expect_result_set(current.result_set_observation().unwrap().clone())
        .idempotency(&key)
        .promote_recovered_required_outputs(recovery, &world.application)
        .unwrap_or_else(|(denial, _)| {
            panic!("same completed-set identity permits promotion: {denial:?}")
        })
        .into_parts();
    assert_eq!(cause, ProductUnpublishedCause::SettlementPending);
    assert!(prior_cleanup.is_empty());
    let (read, publication_failure, cleanup_failure, pending_cleanup) = performed.into_parts();
    assert!(publication_failure.is_none() && cleanup_failure.is_none());
    assert!(matches!(read.unwrap().into_resolution(), worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_)));
    let receipt = world
        .application
        .release_product_publication_recovery(
            pending_cleanup.expect("actual recovered ProgramSource owns old-record cleanup"),
            0,
        )
        .unwrap();
    assert!(receipt.is_complete());
    let mut outputs = preparation
        .start_required_outputs(
            &world.application,
            &request,
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap_or_else(|(_, denial)| panic!("original required demand: {denial:?}"));
    let settlement = super::super::super::settle(|| {
        match outputs.advance(&world.application, &request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settlement) => Some(settlement),
        }
    });
    assert_eq!(settlement.output_count(), 4);
    assert_eq!(settlement.root_producer_contacts_in_this_demand(), 1);
    let truth = request
        .at(settlement.observation())
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(truth.rows()[0].value, length(3));
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    assert!(world
        .application
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
