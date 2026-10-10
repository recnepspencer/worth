//! Real durable-append failure, followed by fresh ordinary request recovery.

use std::time::{Duration, Instant};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRecoveryRequestDenial,
    WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial,
    WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolution as Resolution, WorthQueryApplicationUncommitted,
    WorthQueryManagedApplicationRecoveryDenial, WorthQueryManagedApplicationRecoveryOutcome,
};
use worth_query_host::facade::runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause};

use super::document_retention_model::{
    host::{publish_on_first_program, publish_on_second_program},
    operator_identity::{authenticate_operator, request_scope},
    programs::RemovedOperationRetentionProgram,
    readback::{observe_head, read_retention},
    retention_entry::{
        candidate_count, decision_count, reset_candidate_count, reset_decision_count,
        ReviewedSetRetentionIntent, SetRetentionIntent, DOCUMENT_IDENTITY,
    },
    schema::SetRetentionInput,
};

fn intent(retention_days: u64) -> SetRetentionIntent {
    SetRetentionIntent {
        input: SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days,
        },
    }
}

#[test]
fn fresh_request_recovers_the_actual_unpublished_commit_without_rerunning_handler() {
    // This value is reserved here for the fixture's real decision/candidate observers.
    const RETENTION: u64 = 18;
    reset_candidate_count(RETENTION);
    reset_decision_count(RETENTION);
    let host = publish_on_second_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let before = observe_head(runtime, branch).selected_commit().clone();
    let key = 0xaced_1801;

    runtime.fail_next_durable_append_for_test();
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(intent(RETENTION))
        .without_source()
        .idempotency(&key)
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .expect("the request reaches the real durable append fault");
    let Outcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(partial)) = outcome
    else {
        panic!("the actual owner effect must retain unpublished custody: {outcome:?}");
    };
    assert_eq!(partial.cause(), ProductUnpublishedCause::SettlementPending);
    assert_eq!(partial.owner_effect_count(), 1);
    assert_eq!(partial.expected_product().selected_commit(), &before);
    assert!(partial.relational_requires_settlement());
    assert_eq!(decision_count(RETENTION), 1);
    assert_eq!(candidate_count(RETENTION), 1);
    let recovery = partial.into_recovery();

    let stopped = WorthQueryCancellationSource::new();
    let stopped_scope =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(30), stopped.token());
    stopped.cancel();
    assert!(matches!(
        runtime
            .request(&principal, &stopped_scope)
            .on_branch(branch)
            .mutate(intent(RETENTION))
            .without_source()
            .idempotency(&key)
            .recover_unpublished_in_program(&recovery, &host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))))
    ));
    assert!(recovery.inspect().unwrap().relational_requires_settlement());

    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    assert!(matches!(
        runtime
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch)
            .mutate(intent(RETENTION + 1))
            .without_source()
            .idempotency(&key)
            .recover_unpublished_in_program(&recovery, &host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
            WorthQueryManagedApplicationRecoveryDenial::BindingMismatch
        ))
    ));
    assert!(recovery.inspect().unwrap().relational_requires_settlement());

    let wrong_key = key + 1;
    assert!(matches!(
        runtime
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch)
            .mutate(intent(RETENTION))
            .without_source()
            .idempotency(&wrong_key)
            .recover_unpublished_in_program(&recovery, &host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(
            WorthQueryManagedApplicationRecoveryDenial::BindingMismatch
        ))
    ));
    assert!(recovery.inspect().unwrap().relational_requires_settlement());

    let recovered = runtime
        .request(&fresh_principal, &fresh_scope)
        .on_branch(branch)
        .mutate(intent(RETENTION))
        .without_source()
        .idempotency(&key)
        .recover_unpublished_in_program(&recovery, &host)
        .expect("fresh current authorization recovers the original intent");
    let WorthQueryManagedApplicationRecoveryOutcome::Performed(performed) = recovered else {
        panic!("settled owner effects must produce a performed product successor");
    };
    let (read, publication_failure, cleanup_failure) = performed.into_parts();
    assert!(publication_failure.is_none());
    assert!(cleanup_failure.is_none());
    let Resolution::AlreadyCommitted(receipt) = read.unwrap().into_resolution() else {
        panic!("performed recovery must resolve its original committed receipt");
    };
    let after = observe_head(runtime, branch).selected_commit().clone();
    assert_ne!(after, before);
    assert_eq!(read_retention(runtime, branch), RETENTION);

    let resolved = runtime
        .request(&fresh_principal, &fresh_scope)
        .on_branch(branch)
        .mutate(intent(RETENTION))
        .without_source()
        .idempotency(&key)
        .resolve_idempotency_in_program(&host)
        .unwrap();
    let Resolution::AlreadyCommitted(replayed) = resolved.into_resolution() else {
        panic!("fresh resolution must retain the actual original commit");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert!(matches!(
        runtime
            .request(&fresh_principal, &fresh_scope)
            .on_branch(branch)
            .mutate(intent(RETENTION + 1))
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&host)
            .unwrap()
            .into_resolution(),
        Resolution::IntentDrift
    ));
    assert_eq!(observe_head(runtime, branch).selected_commit(), &after);
    assert_eq!(
        decision_count(RETENTION),
        1,
        "recovery never calls decide again"
    );
    assert_eq!(
        candidate_count(RETENTION),
        1,
        "recovery never rebuilds a candidate"
    );
}

#[test]
fn recovery_reads_require_the_current_program_and_refuse_workflow_bound_actions() {
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let target = *host
        .supported_program::<RemovedOperationRetentionProgram>()
        .unwrap()
        .owned_revision();
    let programs = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let comparison = programs.compare(&target).unwrap();
    assert!(matches!(
        programs.adopt(&comparison).prepare(64).unwrap().publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    let before = observe_head(runtime, branch).selected_commit().clone();
    let key = 0xaced_1802;
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .mutate(intent(9))
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&host),
        Err(WorthQueryApplicationRecoveryRequestDenial::Request(
            WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
        ))
    ));
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .mutate(ReviewedSetRetentionIntent {
                input: intent(9).input
            })
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&host),
        Err(WorthQueryApplicationRecoveryRequestDenial::WorkflowUnsupported)
    ));
    let foreign = publish_on_second_program();
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(branch)
            .mutate(intent(9))
            .without_source()
            .idempotency(&key)
            .resolve_idempotency_in_program(&foreign),
        Err(WorthQueryApplicationRecoveryRequestDenial::Request(
            WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch
        ))
    ));
    assert_eq!(observe_head(runtime, branch).selected_commit(), &before);
}
