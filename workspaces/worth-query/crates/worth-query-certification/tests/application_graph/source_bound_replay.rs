//! Real source-bound mutation retry under a stopped zero-allocation policy.
use crate::document_retention_model::{
    assessment_output::{
        OrdinaryRetentionAssessmentIntent, RetentionAssessmentProducer, RetentionAssessmentProvider,
    },
    host::{publish, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{with_decision_hold, DocumentRetentionRead, DOCUMENT_IDENTITY},
    schema::DocumentRetentionSchema,
};
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Advancement, WorthQueryApplicationProducerProvider,
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
    },
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
    // The lease funds the whole call. Principal resolution's native query
    // completion reserves a result bound before the handler and releases it,
    // so every caller-lease room admitting that bound also admits the
    // projection's predicate retention. The lease therefore funds every owner
    // before the handler in full, and the decision holds all of its room but
    // what the projection keeps; each owner refusing before the retention
    // quotes its share of that keep (MENTALITY §7).
    let ((refused, fresh_work, admitted_only), _) = crate::lease_quote::learn(
        0,
        |keep| {
            let admitted_only = lease(
                crate::lease_quote::DECLARED_BUDGET,
                worth_query_host::facade::runtime::CancellationToken::new(),
            );
            let (refused, fresh_work) = with_decision_hold(
                crate::lease_quote::keep_at_decision(&admitted_only, keep),
                || fresh!(&admitted_only),
            );
            (refused, fresh_work, admitted_only)
        },
        |(refused, _, _)| {
            let denial = refused.as_ref().err()?;
            match denial {
                Denial::Handler(MutationHandlerExecutionDenial::Projection(denial))
                    if denial.kind()
                        != ProjectionKind::InvariantAdmission(
                            InvariantKind::SourceRetentionDenied,
                        ) =>
                {
                    crate::lease_quote::allocation_shortfall(denial.allocation_denial()?)
                }
                _ => None,
            }
        },
    );
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
            worth_query_host::facade::runtime::MemoryLimitDenial::Capacity {
                requested: 38,
                admitted: 0,
                // The charging worker sits two controlled levels below the
                // call's child of the lease that funds the whole call.
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 3 }
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
    assert_eq!(admitted_only.charged_memory_bytes(), 0);

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
