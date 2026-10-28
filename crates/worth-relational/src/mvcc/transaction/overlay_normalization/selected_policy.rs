//! Actual native normalization and validation-dependency replacement use selected custody.
use crate::transactions::data::{CreateIntent, MutationIntent};
use crate::{facade::mvcc::RelationalTransactionIntent, tests::support::*};
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialKind,
    ExecutionAllocationPolicy as Policy, ExecutionAuthority, ExecutionAuthorityConfig, LeaseDenial,
    LeaseRequest, MemoryLimitDenial, MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

const CHILD: &str = "WORTH_NATIVE_NORMALIZATION_POLICY_CHILD";
const BUDGET: u64 = 128 * 1024; // deliberate test lease, no ordinary input eligibility rule
fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            // This allocation witness had an unbounded serial commit request.
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(memory), None),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
#[test]
fn native_normalization_and_validation_footprint_retain_selected_policy() {
    if std::env::var_os(CHILD).is_none() {
        let output = crate::process_deadline::output(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "native_normalization_and_validation_footprint_retain_selected_policy",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CHILD, "1"),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "isolated native owner proof failed: {}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return;
    }
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: None,
    })
    .unwrap();
    let parent = authority.request_lease(request(BUDGET)).unwrap();
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (head, basis) = runtime.observe_branch(&identity).unwrap();
    let name = "selected-normalization-native";
    assert!(!runtime
        .config()
        .identity
        .symbol_table
        .entries
        .iter()
        .any(|(_, value)| value == name));
    let original = batch_create(name);
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(original.clone(), Policy::Execution(&parent))
        .unwrap();
    let raw_index = transaction.overlay.index.clone();
    let raw_footprint = transaction.footprint().clone();
    let symbol_frontier_before = runtime
        .services
        .symbols
        .with_read(|symbols| symbols.identity_frontier());
    let limited = parent.child(request(1)).unwrap();
    let failure = transaction
        .prepare_merged_plan(
            &worth_execution::ExecutionRequest::leased(&limited),
            &runtime,
        )
        .unwrap_err();
    let crate::transactions::data::TransactionCommitError::Conflict { error: failure, .. } =
        failure
    else {
        panic!("selected normalization backing must retain its allocation conflict");
    };
    let cause = failure
        .allocation_denial()
        .expect("normalization retains original allocation cause");
    // Normalization has no reads for a CreateEntity; its first backing is
    // Author::CHUNK (32) RefCell<Option<WriteLocus>> slots, before index rebuilding.
    let normalization_bytes = (32
        * std::mem::size_of::<
            std::cell::RefCell<Option<crate::facade::mvcc::RelationalTransactionWriteLocus>>,
        >()) as u64;
    assert_eq!(
        cause.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity {
                requested: normalization_bytes,
                admitted: 1,
                level: MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    assert!(cause
        .requested_payload_bytes()
        .is_some_and(|bytes| bytes > 1));
    assert_eq!(transaction.batches(), &[original]);
    assert_eq!(transaction.overlay.index, raw_index);
    assert_eq!(transaction.footprint(), &raw_footprint);
    assert_eq!(transaction.overlay.normalization_generation, 0);
    assert!(transaction.last_merged_plan.is_none());
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, head);
    // D9 abandons meaning on refusal; D7 preserves the spent identity floor.
    let entries = runtime.config().identity.symbol_table.entries;
    assert!(!entries.iter().any(|(_, value)| value == name));
    assert_eq!(
        runtime
            .services
            .symbols
            .with_read(|symbols| symbols.symbol(name)),
        None
    );
    let refused_frontier = runtime
        .services
        .symbols
        .with_read(|symbols| symbols.identity_frontier());
    assert!(refused_frontier > symbol_frontier_before);
    drop(limited);
    drop((raw_index, raw_footprint));
    let retained;
    let plan;
    let symbol;
    {
        let child = parent.child(request(BUDGET)).unwrap();
        plan = transaction
            .prepare_merged_plan(&worth_execution::ExecutionRequest::leased(&child), &runtime)
            .unwrap()
            .clone();
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &plan.merged_intents[0] else {
            panic!("actual created entity")
        };
        symbol = spec
            .client_key
            .as_symbol()
            .expect("accepted preview has a reserved identity");
        assert!(
            symbol.0 >= refused_frontier,
            "retry cannot reuse a refused identity"
        );
        assert_eq!(
            transaction
                .pending_symbols
                .as_ref()
                .unwrap()
                .interner()
                .resolve(symbol),
            Some(name)
        );
        retained = transaction.footprint().clone();
    }
    assert_eq!(transaction.overlay.normalization_generation, 1);
    assert!(
        parent.reserve_memory(BUDGET).is_err(),
        "normalized backing outlives its borrowed child"
    );
    // A real validation dependency insertion must admit its typed backing too.
    let limited = parent.child(request(1)).unwrap();
    let mut footprint = retained.clone();
    let before = footprint.clone();
    let denial = footprint
        .derive_validation_dependencies(&plan, Policy::Execution(&limited))
        .unwrap_err();
    // The first new validation read is EntitySchema; Author allocates its
    // 32-slot RefCell<Option<ReadLocus>> chunk before publishing any dependency.
    let validation_bytes = (32
        * std::mem::size_of::<
            std::cell::RefCell<Option<crate::facade::mvcc::RelationalTransactionReadLocus>>,
        >()) as u64;
    assert_eq!(
        denial.allocation_denial().unwrap().kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial::Capacity {
                requested: validation_bytes,
                admitted: 1,
                level: MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    assert!(denial
        .allocation_denial()
        .unwrap()
        .requested_payload_bytes()
        .is_some_and(|bytes| bytes > 1));
    assert_eq!(
        footprint, before,
        "partial validation reads cannot replace the original footprint"
    );
    drop((limited, footprint, before));
    let stopped_token = CancellationSource::new();
    let mut stopped_request = request(BUDGET);
    stopped_request.cancellation = stopped_token.token();
    let stopped = parent.child(stopped_request).unwrap();
    stopped_token.cancel();
    let failure = transaction
        .prepare_merged_plan(
            &worth_execution::ExecutionRequest::leased(&stopped),
            &runtime,
        )
        .unwrap_err();
    let crate::transactions::data::TransactionCommitError::Execution { denial, .. } = failure
    else {
        panic!("preview must preserve cancellation from its commit-stage consult");
    };
    assert_eq!(
        denial.kind,
        crate::transactions::data::CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        )
    );
    assert_eq!(transaction.last_merged_plan.as_ref(), Some(&plan));
    drop(stopped);
    let committed = transaction
        .commit(
            &worth_execution::ExecutionRequest::leased(&parent),
            &runtime,
        )
        .unwrap();
    assert_eq!(changed_entities(&committed).len(), 1);
    assert_ne!(runtime.observe_branch(&identity).unwrap().0, head);
    release_test_commit_snapshot(&runtime, &committed);
    assert!(
        parent.reserve_memory(BUDGET).is_err(),
        "retained normalized footprint still owns backing"
    );
    drop((retained, plan, committed));
    drop(
        parent
            .reserve_memory(BUDGET)
            .expect("all retained typed backings release their charge"),
    );
}
