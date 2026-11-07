//! Exact caller-admitted retained invoice over an actual Native capture.
use super::{create_entity_outcome, durable_runtime, TestDurableMedium};
use std::{alloc::Layout, num::NonZeroUsize, sync::Arc};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialClass,
    ExecutionAllocationPolicy as Policy, ExecutionArray, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionMemoryReservation, LeaseRequest, MemoryLimitDenial,
    MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    durability::{RelationalCheckpointMember, RelationalNativeCheckpointCaptureDenial as Denial},
    transactions::RelationalExecutionDenialCause,
};

fn request(bytes: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(bytes), Some(1_000_000)),
        ),
        deadline: None,
        cancellation,
    }
}

#[test]
fn native_checkpoint_admits_exact_backing_retains_clones_and_restores_editable_truth() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let committed = create_entity_outcome(execution, &runtime, "native-custody-source");
    let head = branch_head(&runtime);
    let system = runtime
        .durability_authority()
        .native_checkpoint(execution, Policy::SystemAllocation)
        .unwrap();
    let quote = u64::try_from(system.bytes().len()).unwrap();
    // The first original-controlled evidence/encoding scope enters the SDK before
    // emitting any envelope. Its current framework admission is an independent
    // literal invoice, not the eventual encoded checkpoint length.
    // The owned monotonic deadline adds 8 bytes to the kernel framework layout.
    const FIRST_SCOPE_BYTES: u64 = 416;
    const PEAK_BYTES: u64 = 1024 * 1024;
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(8).unwrap(),
        charged_memory_bytes: Some(512 * 1024 * 1024),
    })
    .unwrap();
    let parent = authority
        .request_lease(request(PEAK_BYTES, CancellationToken::new()))
        .unwrap();
    let zero = parent.child(request(0, CancellationToken::new())).unwrap();
    let error = runtime
        .durability_authority()
        .native_checkpoint(
            &worth_execution::ExecutionRequest::leased(&zero),
            Policy::Execution(&zero),
        )
        .unwrap_err();
    assert_eq!(
        error,
        Denial::Request(RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested: FIRST_SCOPE_BYTES,
            admitted: 0,
            ancestor: 0,
        })
    );
    assert_eq!(branch_head(&runtime), head);
    drop(zero);

    let cancellation = CancellationSource::new();
    let stopped = parent
        .child(request(PEAK_BYTES, cancellation.token()))
        .unwrap();
    cancellation.cancel();
    let Denial::Allocation(error) = runtime
        .durability_authority()
        .native_checkpoint(
            &worth_execution::ExecutionRequest::leased(&stopped),
            Policy::Execution(&stopped),
        )
        .unwrap_err()
    else {
        panic!("caller cancellation must remain typed");
    };
    assert_eq!(
        error.classify(),
        ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        ))
    );
    assert_eq!(error.requested_payload_bytes(), None);
    drop(stopped);

    let retained = {
        let child = parent
            .child(request(PEAK_BYTES, CancellationToken::new()))
            .unwrap();
        let captured = runtime
            .durability_authority()
            .native_checkpoint(
                &worth_execution::ExecutionRequest::leased(&child),
                Policy::Execution(&child),
            )
            .unwrap();
        assert_eq!(captured.bytes(), system.bytes());
        assert_eq!(captured.captured_sections(), system.captured_sections());
        let clone = captured.clone();
        assert_eq!(clone.bytes().as_ptr(), captured.bytes().as_ptr());
        drop(captured);
        clone
    };
    // New is destination placement. Ordinary external DATA has no caller
    // payload reservation; the original invoice counts admitted backing only.
    let member_quote = retained
        .members()
        .iter()
        .filter_map(|member| match member {
            worth_relational::facade::durability::RelationalCheckpointMember::New {
                bytes, ..
            } => bytes.charged_payload_bytes(),
            RelationalCheckpointMember::Carried { .. } => {
                panic!("embedded capture resolves every member")
            }
        })
        .sum::<u64>();
    // The member roster is retained DATA too, not an encoded member payload.
    // Quote its declared physical fields independently from the observed ledger.
    struct RosterInvoice {
        _entries: ExecutionArray<RelationalCheckpointMember>,
        _memory: Option<ExecutionMemoryReservation>,
    }
    // The canonical ACK cursor holds two preexisting Arc owners per covered
    // cell. Journal indexes already belong to the source caller and are shared.
    struct CanonicalCellInvoice {
        _index: Arc<()>,
        _cell: Arc<()>,
    }
    struct CursorInvoice {
        _cells: ExecutionArray<CanonicalCellInvoice>,
        _memory: Option<ExecutionMemoryReservation>,
    }
    // The external provider seals original canonical bytes with their own
    // immutable-byte control. Ordinary checkpoint buffers do not own this bill.
    struct OriginalByteInvoice {
        _bytes: ExecutionArray<u8>,
        _memory: Option<ExecutionMemoryReservation>,
    }
    fn arc_bytes<T>() -> usize {
        Layout::new::<(usize, usize)>()
            .extend(Layout::new::<T>())
            .unwrap()
            .0
            .pad_to_align()
            .size()
    }
    let canonical_count = medium.offered().len();
    assert_eq!(
        canonical_count, 1,
        "this capture has one actual Journal entry"
    );
    let roster_quote =
        u64::try_from(arc_bytes::<RosterInvoice>() + size_of_val(retained.members())).unwrap();
    let cursor_quote = u64::try_from(
        arc_bytes::<CursorInvoice>()
            + Layout::array::<CanonicalCellInvoice>(canonical_count)
                .unwrap()
                .size(),
    )
    .unwrap();
    let original_control_quote =
        u64::try_from(canonical_count * arc_bytes::<OriginalByteInvoice>()).unwrap();
    let retained_quote =
        quote + member_quote + roster_quote + cursor_quote + original_control_quote;
    eprintln!("checkpoint capture first_scope={FIRST_SCOPE_BYTES} outer={quote} retained_members={member_quote} roster={roster_quote} cursor={cursor_quote} original_byte_control={original_control_quote} parent_charged={}", parent.charged_memory_bytes());
    assert_eq!(parent.charged_memory_bytes(), retained_quote);
    assert_eq!(
        parent
            .reserve_memory(PEAK_BYTES - retained_quote + 1)
            .unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: PEAK_BYTES - retained_quote + 1,
            admitted: PEAK_BYTES - retained_quote,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    let mut recovered = durable_runtime(&medium);
    let prior_owner = recovered.runtime_instance_id();
    let restored = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &retained)
        .unwrap();
    assert_eq!(restored.latest_commit.as_ref(), Some(&committed.commit));
    assert_ne!(recovered.runtime_instance_id(), prior_owner);
    let later = create_entity_outcome(execution, &recovered, "native-custody-edit");
    assert_eq!(branch_head(&recovered).as_ref(), Some(&later.commit));
    drop(retained);
    // Recovered roots may retain the original encoded region backing. Their
    // last actual custody must end before expecting the parent's complete refund.
    super::release_test_commit_snapshot(&recovered, &later);
    drop(later);
    drop(recovered);
    assert_eq!(parent.charged_memory_bytes(), 0);
    drop(
        parent
            .reserve_memory(PEAK_BYTES)
            .expect("last clone releases all original custody"),
    );
}

fn branch_head(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> Option<worth_relational::facade::history::RelationalCommitReceipt> {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .unwrap()
}
