//! Real source-bound mutation retry under a stopped zero-allocation policy.
use crate::document_retention_model::{
    assessment_output::{
        OrdinaryRetentionAssessmentIntent, RetentionAssessmentProducer, RetentionAssessmentProvider,
    },
    host::{publish, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{DocumentRetentionRead, DOCUMENT_IDENTITY},
    schema::DocumentRetentionSchema,
};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement, WorthQueryApplicationProducerProvider,
        WorthQueryManagedComputationInterruption as Interruption,
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as Denial,
    },
    primary_graph::WorthQueryMutationHandlerWork,
    runtime::{ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy, LeaseRequest},
};

#[path = "source_bound_replay/program.rs"]
mod program;
#[path = "source_bound_replay/recovery.rs"]
mod recovery;

#[test]
fn source_bound_recorded_retry_preserves_checked_identity_before_fresh_allocation() {
    const CHILD: &str = "WORTH_QUERY_SOURCE_BOUND_REPLAY_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = format!(
            "{}::source_bound_recorded_retry_preserves_checked_identity_before_fresh_allocation",
            module_path!().split_once("::").unwrap().1
        );
        let output = crate::process_deadline::output(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &name, "--nocapture"])
                .env(CHILD, "1"),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        return;
    }
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let token = worth_query_host::facade::runtime::CancellationSource::new();
    let lease = |memory, cancellation| {
        authority
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    // The lease funds the whole call, so its work admits the handler.
                    ExecutionBudget::new(
                        NonZeroUsize::new(1).unwrap(),
                        Some(memory),
                        Some(16_000_000),
                    ),
                ),
                deadline: None,
                cancellation,
            })
            .unwrap()
    };
    let zero = lease(0, token.token());
    let stopping = worth_query_host::facade::runtime::CancellationSource::new();
    let stopped = lease(16 * 1024 * 1024, stopping.token());
    stopping.cancel();
    let host = publish(
        program::validated_program(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(crate::document_retention_model::programs::validated_second_program()),
    )
    .expect("the source-bound ordinary action program must install");
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let observed = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .query(DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(observed.rows().len(), 1);
    assert_eq!(observed.observed_sources().len(), 1);
    assert_eq!(observed.rows()[0].retention_days, SEED_RETENTION);
    let input = OrdinaryRetentionAssessmentIntent(
        <RetentionAssessmentProvider as WorthQueryApplicationProducerProvider<
            DocumentRetentionSchema,
            RetentionAssessmentProducer,
        >>::operation_input(&RetentionAssessmentProvider, &observed.rows()[0]),
    );
    let source = observed.observed_sources()[0].clone();
    let initial = observe_head(runtime, branch);
    let fresh_key = 0xb0ad_0001;
    macro_rules! fresh {
        ($lease:expr) => {
            runtime
                .request(&principal, &scope)
                .on_branch(branch)
                .advanced()
                .mutate(input.clone())
                .expect_source(source.clone())
                .idempotency(&fresh_key)
                .execute_in_program_report(&host, Policy::Execution($lease))
                .into_parts()
        };
    }
    // A zero lease cannot open the call it would fund; its quote is the call's
    // own admission, refused before any read.
    let (unopened, unopened_work) = fresh!(&zero);
    let Some(Denial::ExecutionRequest(Advancement::Resource(Resource::MemoryLimit {
        requested: admission,
        admitted: 0,
        level: WorthQueryMemoryLimitLevel::Policy,
    }))) = unopened.err()
    else {
        panic!("the call's own admission refuses first")
    };
    assert!(admission > 0);
    assert_eq!(unopened_work, WorthQueryMutationHandlerWork::NotStarted);
    // A stopped lease interrupts the call it would fund, at its opening.
    let (interrupted, interrupted_work) = fresh!(&stopped);
    assert!(
        matches!(
            interrupted,
            Err(Denial::ExecutionRequest(Advancement::Interrupted(
                Interruption::Cancelled
            )))
        ),
        "{interrupted:?}"
    );
    assert_eq!(interrupted_work, WorthQueryMutationHandlerWork::NotStarted);
    // Neither refusal published or charged its lease.
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        initial.selected_commit()
    );
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(zero.charged_memory_bytes(), 0);
    assert_eq!(stopped.charged_memory_bytes(), 0);

    let committed_key = 0xb0ad_0003;
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&committed_key)
        .execute_in_program(&host, Policy::SystemAllocation)
        .unwrap();
    let Outcome::Committed { receipt, .. } = outcome else {
        panic!("real source-bound commit required");
    };
    let head = observe_head(runtime, branch);
    token.cancel();
    assert_eq!(
        Policy::Execution(&zero).check_live().unwrap_err().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    let (retried, work) = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(input)
        .expect_source(source)
        .idempotency(&committed_key)
        .execute_in_program_report(&host, Policy::Execution(&zero))
        .into_parts();
    let commit = match retried.expect("checked source-bound recorded retry") {
        Outcome::AlreadyCommitted(recorded) => recorded.commit_reference().commit_id,
        Outcome::PreviouslyCommitted(recorded) => recorded.commit_id(),
        _ => panic!("recorded receipt required"),
    };
    assert_eq!(commit, receipt.commit_reference().commit_id);
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        head.selected_commit()
    );
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
}
