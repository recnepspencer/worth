//! A real read-set quote establishes a later, distinct native staging refusal.
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_execution::facade::provider_session::WorthQueryDecisionReadSetDenialKind;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialStage as Stage, WorthQueryApplicationUncommitted,
        WorthQueryInvariantExecutionDenialKind as InvariantKind,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy,
        ExecutionAuthority, ExecutionAuthorityConfig, LeaseDenial, LeaseRequest,
    },
};
use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as NativeDenial;

use crate::document_retention_model::{
    host::{publish_on_first_program, SEED_RETENTION},
    operator_identity::{authenticate_operator, request_scope},
    readback::{observe_head, read_retention},
    retention_entry::{SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::SetRetentionInput,
};

fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn ordinary_mutation_read_set_fits_before_native_staging_refuses_the_same_lease() {
    const CHILD: &str = "WORTH_QUERY_NATIVE_STAGING_PROOF_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = format!(
            "{}::ordinary_mutation_read_set_fits_before_native_staging_refuses_the_same_lease",
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
    // The fixture World has no execution authority. This caller supplies its
    // real process authority; no World identity gate or allocation fallback.
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: None,
    })
    .unwrap();
    let zero = authority.request_lease(request(0)).unwrap();
    let host = publish_on_first_program();
    let runtime = host.runtime();
    let branch = host.current_world();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let original = observe_head(runtime, branch);
    const RETENTION: u64 = 9;
    let key = 0xa110_1001;
    macro_rules! mutation {
        () => {
            runtime
                .request(&principal, &scope)
                .on_branch(branch)
                .mutate(SetRetentionIntent {
                    input: SetRetentionInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: RETENTION,
                    },
                })
                .without_source()
                .idempotency(&key)
        };
    }
    // This separate probe obtains the real checked provider Layout quote. It
    // is only read-set refusal evidence, never native staging evidence.
    let mut probe = mutation!();
    let candidate = match probe
        .prepare_in_program(&host, Policy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh ordinary candidate required"),
    };
    let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
        candidate.commit(Policy::Execution(&zero))
    else {
        panic!("read-set quote probe must refuse before native staging");
    };
    assert_eq!(denial.stage(), Stage::DecisionReadSet, "{denial:?}");
    let failure = denial
        .decision_read_set_failure()
        .expect("full read-set cause");
    assert_eq!(
        failure.kind(),
        WorthQueryDecisionReadSetDenialKind::AllocationDenied
    );
    let cause = failure.allocation_denial().expect("actual provider quote");
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial {
                requested: cause.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    let read_set_bytes = cause
        .requested_payload_bytes()
        .expect("checked Layout quote");
    assert!(read_set_bytes > 0);
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );

    // Exact nonzero caller budget admits that same read set. Its retained
    // backing then coexists with native staging, which needs additional payload.
    let stage_lease = authority.request_lease(request(read_set_bytes)).unwrap();
    let mut staging = mutation!();
    let candidate = match staging
        .prepare_in_program(&host, Policy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("refusal must not record idempotency"),
    };
    let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
        candidate.commit(Policy::Execution(&stage_lease))
    else {
        panic!("the later native staging owner must refuse additional backing");
    };
    assert_eq!(denial.stage(), Stage::InvariantExecution);
    assert!(denial.decision_read_set_failure().is_none());
    let failure = denial
        .invariant_execution_failure()
        .expect("full native cause survives");
    assert_eq!(failure.kind(), InvariantKind::AllocationDenied);
    let native = failure
        .relational_staging_denial()
        .expect("specifically native staging");
    let NativeDenial::AllocationDenied(native_cause) = native else {
        panic!("expected native physical admission, got {native:?}");
    };
    assert_eq!(failure.allocation_denial(), Some(native_cause));
    // The admitted read set occupies the complete caller policy budget.
    assert_eq!(
        native_cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_query_host::facade::runtime::MemoryLimitDenial {
                requested: native_cause.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_query_host::facade::runtime::MemoryLimitLevel::Policy { ancestor: 0 },
            }
        ))
    );
    assert!(native_cause
        .requested_payload_bytes()
        .is_some_and(|bytes| bytes > 0));
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );
    drop(
        stage_lease
            .reserve_memory(read_set_bytes)
            .expect("failed attempt released backing"),
    );

    let Outcome::Committed { receipt, .. } = mutation!()
        .execute_in_program(&host, Policy::SystemAllocation)
        .expect("same-key ordinary retry")
    else {
        panic!("physical refusal must not record a commit or reserve the key");
    };
    assert_eq!(read_retention(runtime, branch), RETENTION);
    let head = observe_head(runtime, branch);
    assert_ne!(head.selected_commit(), original.selected_commit());
    assert_eq!(receipt.product_branch(), branch);
}
