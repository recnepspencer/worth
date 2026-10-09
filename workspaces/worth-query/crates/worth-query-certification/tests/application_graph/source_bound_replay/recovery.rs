//! Genuine unpublished recovery retains the original row or completed-set proof.
use super::program;
use crate::document_retention_model::{
    assessment_output::{
        OrdinaryRetentionAssessmentIntent, RetentionAssessmentProducer, RetentionAssessmentProvider,
    },
    host::publish,
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{
        ordinary_counts, reset_ordinary_counts, DocumentRetentionRead, SetRetentionIntent,
        DOCUMENT_IDENTITY,
    },
    schema::{DocumentRetentionSchema, SetRetentionInput},
};
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
    },
    application_contribution::WorthQueryApplicationProducerProvider,
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationRecoveryRequestDenial, WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationIdempotencyResolution as Resolution, WorthQueryApplicationUncommitted,
        WorthQueryManagedApplicationRecoveryDenial, WorthQueryManagedApplicationRecoveryOutcome,
    },
    runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause},
};

#[test]
fn original_row_recovers_unpublished_effects_without_handler_reexecution() {
    recover_original_source(false, 8);
}

#[test]
fn original_result_set_recovers_unpublished_effects_without_becoming_a_row() {
    recover_original_source(true, 9);
}

fn recover_original_source(result_set: bool, retention: u64) {
    // These two ordinary-assessment input values are reserved for these contact
    // observers; the handler follows exactly the same path for every input.
    reset_ordinary_counts(retention);
    let host = publish(
        program::validated_program(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(crate::document_retention_model::programs::validated_second_program()),
    )
    .unwrap();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let seed_key = 0xb0ad_1000 + retention;
    let set_retention = |days| SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: days,
        },
    };
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .mutate(set_retention(retention))
            .without_source()
            .idempotency(&seed_key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    let original = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(original.rows().len(), 1);
    assert_eq!(original.observed_sources().len(), 1);
    assert_eq!(original.rows()[0].retention_days, retention);
    let input = |row: &_| {
        OrdinaryRetentionAssessmentIntent(
            <RetentionAssessmentProvider as WorthQueryApplicationProducerProvider<
                DocumentRetentionSchema,
                RetentionAssessmentProducer,
            >>::operation_input(&RetentionAssessmentProvider, row),
        )
    };
    let intent = input(&original.rows()[0]);
    let mut changed_row = original.rows()[0].clone();
    changed_row.retention_days += 1;
    let wrong_input = input(&changed_row);
    let row_source = original.observed_sources()[0].clone();
    let set_source = original
        .result_set_observation()
        .expect("the completed original query retains its full result-set proof")
        .clone();
    let before = observe_head(runtime, branch).selected_commit().clone();
    let key = 0xb0ad_2000 + retention;

    macro_rules! request {
        ($principal:expr, $scope:expr, $input:expr, $key:expr, $as_set:expr) => {{
            let request = runtime
                .request($principal, $scope)
                .on_branch(branch)
                .mutate($input);
            let request = if $as_set {
                request.expect_result_set(set_source.clone())
            } else {
                request.expect_source(row_source.clone())
            };
            request.idempotency($key)
        }};
    }

    runtime.fail_next_durable_append_for_test();
    let outcome = request!(&principal, &scope, intent.clone(), &key, result_set)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(partial)) = outcome
    else {
        panic!("real source-bound owner effects must retain unpublished custody: {outcome:?}");
    };
    assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
    assert_eq!(partial.owner_effect_count(), 1);
    assert_eq!(partial.expected_product().selected_commit(), &before);
    assert!(partial.relational_requires_settlement());
    assert_eq!(ordinary_counts(retention), (1, 1));
    let recovery = partial.into_recovery();

    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    );
    cancellation.cancel();
    assert!(matches!(
        request!(&principal, &stopped, intent.clone(), &key, result_set)
            .recover_unpublished_in_program(&recovery, &host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Interrupted(
            WorthQueryRequestInterruption::Cancelled
        ))
    ));
    assert!(recovery.inspect().unwrap().relational_requires_settlement());
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let wrong_key = key + 100;
    for (input, request_key, as_set) in [
        (wrong_input, &key, result_set),
        (intent.clone(), &wrong_key, result_set),
        // A complete-set proof is not interchangeable with a row proof, even
        // from the same original query, basis and single returned document.
        (intent.clone(), &key, !result_set),
    ] {
        assert!(matches!(
            request!(&fresh_principal, &fresh_scope, input, request_key, as_set)
                .recover_unpublished_in_program(&recovery, &host),
            Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
                WorthQueryManagedApplicationRecoveryDenial::BindingMismatch
            ))
        ));
        assert!(recovery.inspect().unwrap().relational_requires_settlement());
        assert_eq!(ordinary_counts(retention), (1, 1));
    }

    let outcome = request!(
        &fresh_principal,
        &fresh_scope,
        intent.clone(),
        &key,
        result_set
    )
    .recover_unpublished_in_program(&recovery, &host)
    .unwrap();
    let WorthQueryManagedApplicationRecoveryOutcome::Performed(performed) = outcome else {
        panic!("original source must recover its retained performed effects");
    };
    let (read, publication_failure, cleanup_failure) = performed.into_parts();
    assert!(publication_failure.is_none());
    assert!(cleanup_failure.is_none());
    let Resolution::AlreadyCommitted(receipt) = read.unwrap().into_resolution() else {
        panic!("recovery must deliver the actual original receipt");
    };
    let recovered_head = observe_head(runtime, branch).selected_commit().clone();
    assert_ne!(recovered_head, before);
    assert_eq!(read_retention(runtime, branch), retention);
    assert_eq!(ordinary_counts(retention), (1, 1));

    // Change an actual row operand through the ordinary installed action.
    // The completed-set proof covers membership, not projected row values:
    // this field edit changes the row proof but preserves the selected set.
    let edit_key = key + 200;
    assert!(matches!(
        runtime
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch)
            .mutate(set_retention(retention + 1))
            .without_source()
            .idempotency(&edit_key)
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    let changed = runtime
        .request(&fresh_principal, &fresh_scope)
        .on_branch(branch)
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(changed.rows()[0].retention_days, retention + 1);
    let changed_request = runtime
        .request(&fresh_principal, &fresh_scope)
        .on_branch(branch)
        .mutate(intent.clone());
    let changed_request = if result_set {
        changed_request.expect_result_set(
            changed
                .result_set_observation()
                .expect("the completed changed query retains its full result-set proof")
                .clone(),
        )
    } else {
        changed_request.expect_source(changed.observed_sources()[0].clone())
    };
    let changed_resolution = changed_request
        .idempotency(&key)
        .resolve_idempotency_in_program(&host)
        .unwrap()
        .into_resolution();
    if result_set {
        let Resolution::AlreadyCommitted(unchanged_membership) = changed_resolution else {
            panic!("unchanged set membership resolved as {changed_resolution:?}");
        };
        assert_eq!(
            unchanged_membership.commit_reference(),
            receipt.commit_reference()
        );
    } else {
        assert!(
            matches!(changed_resolution, Resolution::IntentDrift),
            "changed row source resolved as {changed_resolution:?}"
        );
    }
    let final_head = observe_head(runtime, branch).selected_commit().clone();
    let resolved = request!(&fresh_principal, &fresh_scope, intent, &key, result_set)
        .resolve_idempotency_in_program(&host)
        .unwrap();
    let Resolution::AlreadyCommitted(replayed) = resolved.into_resolution() else {
        panic!("original source remains the exact keyed historical request");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(observe_head(runtime, branch).selected_commit(), &final_head);
    assert_eq!(read_retention(runtime, branch), retention + 1);
    assert_eq!(ordinary_counts(retention), (1, 1));
}
