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
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
#[test]
fn native_normalization_and_validation_footprint_retain_selected_policy() {
    if std::env::var_os(CHILD).is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "native_normalization_and_validation_footprint_retain_selected_policy",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD, "1")
            .status()
            .unwrap();
        assert!(
            status.success(),
            "isolated native owner proof failed: {status}"
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
    let limited = parent.child(request(1)).unwrap();
    let failure = transaction
        .merged_plan(&runtime, Policy::Execution(&limited))
        .unwrap_err();
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
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            requested: normalization_bytes,
            admitted: 1,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        }))
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
    // The interner advanced before the typed replacement refused. Its actual
    // entries must still be published to the configuration snapshot.
    let entries = runtime.config().identity.symbol_table.entries;
    let symbol = entries
        .iter()
        .find(|(_, value)| value == name)
        .expect("actual new symbol is retained after refusal")
        .0;
    assert_eq!(
        runtime
            .preparation_runtime_snapshot()
            .services
            .symbols
            .resolve(symbol)
            .as_deref(),
        Some(name)
    );
    drop(limited);
    drop((raw_index, raw_footprint));
    let retained;
    let plan;
    {
        let child = parent.child(request(BUDGET)).unwrap();
        plan = transaction
            .merged_plan(&runtime, Policy::Execution(&child))
            .unwrap()
            .clone();
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &plan.merged_intents[0] else {
            panic!("actual created entity")
        };
        assert_eq!(spec.client_key.as_symbol(), Some(symbol));
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
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            requested: validation_bytes,
            admitted: 1,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        }))
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
        .merged_plan(&runtime, Policy::Execution(&stopped))
        .unwrap_err();
    assert_eq!(
        failure.allocation_denial().unwrap().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    assert_eq!(transaction.last_merged_plan.as_ref(), Some(&plan));
    drop(stopped);
    let committed = transaction
        .commit(&runtime, Policy::Execution(&parent))
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
