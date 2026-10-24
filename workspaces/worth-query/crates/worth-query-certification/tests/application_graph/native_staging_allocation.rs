//! A real read-set quote establishes a later, distinct native staging refusal.
use std::num::NonZeroUsize;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_execution::facade::provider_session::WorthQueryDecisionReadSetDenialKind;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationCommitDenialStage as Stage, WorthQueryApplicationUncommitted,
        WorthQueryInvariantExecutionDenialKind as InvariantKind,
    },
    runtime::{
        CancellationToken, ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy,
        LeaseDenial, LeaseRequest, MemoryLimitDenial, MemoryLimitLevel,
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

/// Native staging refuses where the read set does: the attempt's control
/// child sits under the call's child of the tracked child, whose lease's held
/// room runs out.
const STAGING_ANCESTOR: u32 = 3;

/// What an owner before native staging still lacked when it refused for
/// lease memory: the commit's request authority, the read set's own capture,
/// or the proposal's source rebase retention. Native staging's refusal ends
/// the learning.
fn pre_staging_shortfall(
    denial: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenial,
) -> Option<u64> {
    if denial.invariant_execution_failure().is_some() {
        return None;
    }
    // The read set's principal fact comparison refuses as an execution stop.
    if let Kind::ExecutionResource {
        denial:
            Resource::MemoryLimit {
                requested,
                admitted,
                ..
            },
        ..
    } = denial.kind()
    {
        return Some(requested - admitted);
    }
    crate::lease_quote::commit_shortfall(denial)
        .or_else(|| crate::lease_quote::allocation_shortfall(denial.allocation_denial()?))
}

fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            // The lease funds the whole commit call, so its work admits it.
            ExecutionBudget::new(
                NonZeroUsize::new(1).unwrap(),
                Some(memory),
                Some(16_000_000),
            ),
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
    // The fixture World has no execution authority. This caller supplies its
    // real process authority; no World identity gate or allocation fallback.
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
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
                .advanced()
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
    macro_rules! candidate {
        ($request:ident) => {
            match $request
                .prepare_in_program(&host, Policy::SystemAllocation)
                .unwrap()
            {
                Preparation::Prepared(candidate) => candidate,
                Preparation::Settled(_) => panic!("a refusal must not record idempotency"),
            }
        };
    }
    // The lease funds the whole commit call: a zero lease cannot open it, and
    // its quote is the call's own admission.
    let mut unopened = mutation!();
    let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
        candidate!(unopened).commit(Policy::Execution(&zero))
    else {
        panic!("an unfunded commit call must remain public Uncommitted::Denied");
    };
    assert_eq!(denial.stage(), Stage::ResourceAdmission, "{denial:?}");
    let Kind::ExecutionResource {
        denial:
            Resource::MemoryLimit {
                requested: admission,
                admitted: 0,
                level: WorthQueryMemoryLimitLevel::Policy,
            },
        ..
    } = denial.kind()
    else {
        panic!("the call's own admission refuses: {denial:?}")
    };
    assert!(admission > 0);
    // This separate probe obtains the real checked provider Layout quote. It
    // is only read-set refusal evidence, never native staging evidence.
    // The commit also charges its provider plan before the read set; each
    // earlier owner's refusal quotes its share.
    let ((denial, admitted_only), funded) = crate::lease_quote::learn(
        admission,
        |funded| {
            let lease = authority
                .request_lease(request(crate::lease_quote::DECLARED_BUDGET))
                .unwrap();
            let mut probe = mutation!();
            let candidate = candidate!(probe);
            let call = crate::lease_quote::tracked(&lease);
            let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
                crate::lease_quote::with_room(&lease, funded, || {
                    candidate.commit(Policy::Execution(&call))
                })
            else {
                panic!("read-set quote probe must refuse before native staging");
            };
            (denial, lease)
        },
        |(denial, _)| crate::lease_quote::commit_shortfall(denial),
    );
    assert_eq!(denial.stage(), Stage::DecisionReadSet, "{denial:?}");
    let failure = denial
        .decision_read_set_failure()
        .unwrap_or_else(|| panic!("full read-set cause: {denial:?}"));
    assert_eq!(
        failure.kind(),
        WorthQueryDecisionReadSetDenialKind::AllocationDenied
    );
    let cause = failure.allocation_denial().expect("actual provider quote");
    // The primary read set reserves its whole checked capture before any
    // evidence; the attempt's control child sits under the call's child of
    // the tracked child, whose lease's held room refuses it.
    let ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
        MemoryLimitDenial::Capacity {
            requested: read_set_bytes,
            admitted: read_set_room,
            level: MemoryLimitLevel::Policy { ancestor: 3 },
        },
    )) = cause.kind()
    else {
        panic!("the read set's own capture quote refuses: {cause:?}")
    };
    assert_eq!(cause.requested_payload_bytes(), Some(read_set_bytes));
    assert!(read_set_room < read_set_bytes);
    assert_eq!(read_retention(runtime, branch), SEED_RETENTION);
    assert_eq!(
        observe_head(runtime, branch).selected_commit(),
        original.selected_commit()
    );

    // The learned pre-read-set charges plus the read set's and the source
    // rebase's own shortfalls admit both. The read set's retained backing then
    // coexists with native staging, which needs more.
    let ((denial, stage_lease), _) = crate::lease_quote::learn(
        funded + (read_set_bytes - read_set_room),
        |stage_budget| {
            let stage_lease = authority
                .request_lease(request(crate::lease_quote::DECLARED_BUDGET))
                .unwrap();
            let mut staging = mutation!();
            let candidate = candidate!(staging);
            let call = crate::lease_quote::tracked(&stage_lease);
            let Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) =
                crate::lease_quote::with_room(&stage_lease, stage_budget, || {
                    candidate.commit(Policy::Execution(&call))
                })
            else {
                panic!("the later native staging owner must refuse additional backing");
            };
            (denial, stage_lease)
        },
        |(denial, _)| pre_staging_shortfall(denial),
    );
    assert_eq!(denial.stage(), Stage::InvariantExecution, "{denial:?}");
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
    // One mutation inserts its first write into a CHUNK32 staging array
    // (worth-relational/staging_storage/author.rs), before later backing.
    let staging_quote = 32
        * std::mem::size_of::<
            std::cell::RefCell<
                Option<worth_relational::facade::mvcc::RelationalTransactionWriteLocus>,
            >,
        >() as u64;
    let ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
        MemoryLimitDenial::Capacity {
            requested,
            admitted,
            level:
                MemoryLimitLevel::Policy {
                    ancestor: STAGING_ANCESTOR,
                },
        },
    )) = native_cause.kind()
    else {
        panic!("native staging's own quote refuses at the lease: {native_cause:?}")
    };
    assert_eq!(requested, staging_quote);
    assert!(admitted < staging_quote, "{native_cause:?}");
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
            .reserve_memory(crate::lease_quote::DECLARED_BUDGET)
            .expect("failed attempt released backing"),
    );
    assert_eq!(admitted_only.charged_memory_bytes(), 0);

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
