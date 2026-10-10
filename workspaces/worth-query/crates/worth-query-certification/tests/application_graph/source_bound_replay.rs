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
    application_contribution::WorthQueryApplicationProducerProvider,
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationRequestExt,
        WorthQueryApplicationRequestMutationDenial as Denial,
    },
    primary_graph::{
        MutationHandlerExecutionDenial, WorthQueryInvariantProjectionDenialKind as InvariantKind,
        WorthQueryMutationHandlerWork, WorthQueryOperationProjectionDenialKind as ProjectionKind,
    },
    runtime::{
        ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy, LeaseDenial,
        LeaseRequest,
    },
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
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
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
    let authority = worth_query_host::facade::primary_graph::test_execution_authority();
    let token = worth_query_host::facade::runtime::CancellationSource::new();
    let zero = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 0, 1),
            ),
            deadline: None,
            cancellation: token.token(),
        })
        .unwrap();
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
    let (refused, fresh_work) = runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&fresh_key)
        .execute_in_program_report(&host, Policy::Execution(&zero))
        .into_parts();
    let denial = match refused {
        Err(Denial::Handler(MutationHandlerExecutionDenial::Projection(denial))) => denial,
        Err(denial) => {
            panic!("fresh source-bound execution refused at an unexpected owner: {denial:?}")
        }
        Ok(_) => panic!("fresh source-bound execution must refuse real payload admission"),
    };
    assert_eq!(
        denial.kind(),
        ProjectionKind::InvariantAdmission(InvariantKind::SourceRetentionDenied)
    );
    let invariant = denial
        .invariant_denial()
        .expect("original invariant refusal");
    assert_eq!(invariant.kind(), InvariantKind::SourceRetentionDenied);
    let allocation = invariant.allocation_denial().expect("exact lower cause");
    assert_eq!(denial.allocation_denial(), Some(allocation));
    assert_eq!(
        allocation.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial {
                requested: 38,
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    // The same actual Document identity predicate as the ordinary mutation control.
    assert_eq!(allocation.requested_payload_bytes(), Some(38));
    let WorthQueryMutationHandlerWork::Captured(work) = fresh_work else {
        panic!("the real decision projection executed before source retention refused");
    };
    assert!(work.handler_contacted());
    let projection = work.projection_work();
    assert_eq!(denial.projection_work(), Some(projection));
    assert_eq!(invariant.projection_work(), Some(projection));
    assert_eq!(projection.equality_lookups(), 1);
    assert_eq!(projection.index_candidates_examined(), 1);
    assert_eq!(projection.field_reads(), 0);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        initial.selected_commit()
    );
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);

    let committed_key = 0xb0ad_0003;
    let outcome = runtime
        .request(&principal, &scope)
        .on_branch(branch)
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
