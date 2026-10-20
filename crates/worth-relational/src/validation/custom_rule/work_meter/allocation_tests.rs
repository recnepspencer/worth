//! Read-only planning work and retained allocations are separate declarations.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn planning_visits_claim_only_the_allocation_they_declare() {
    for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, Some(1024), Some(100)),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let meter = CustomPreparationBudget::new(lease.status(), Some(100), Some(12));
        assert!(
            meter.try_visit(20),
            "a borrowed visit retains no allocation"
        );
        assert_eq!(meter.claimed_memory(), 0);
        assert!(meter.try_plan_item(12));
        assert_eq!(meter.claimed_memory(), 12);
        assert_eq!(meter.charged_work(), 20 + 1);
        assert!(
            !meter.try_plan_item(1),
            "the next actual allocation is unfunded"
        );
        assert_eq!(meter.stop(), Some(CustomPreparationStop::MemoryExhausted));
        assert_eq!(meter.claimed_memory(), 12);
    }
}

#[test]
fn authored_checkpoints_do_not_allocate_structural_nodes() {
    use crate::validation::data::CustomInvariantLeaseBudget;
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, Some(1024), Some(100)),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let budget = CustomPreparationBudget::new(lease.status(), Some(100), Some(0));
    let work =
        CustomInvariantWorkMeter::new(Some(NonZeroU64::new(100).unwrap())).with_lease(&budget);
    assert!(
        work.checkpoint(20),
        "twenty authored arithmetic units have no heap allocation"
    );
    assert_eq!(budget.claimed_memory(), 0);
    assert_eq!(budget.charged_work(), 20);
    assert!(!work.claim_scratch(1));
    assert_eq!(budget.stop(), Some(CustomPreparationStop::MemoryExhausted));
}

#[test]
fn tracked_preparation_clone_keeps_original_payer_and_typed_refusal() {
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, Some(4096), Some(100)),
            ),
            cancellation: CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let budget = CustomPreparationBudget::tracked(&lease).unwrap();
    let metadata = lease.charged_memory_bytes();
    assert!(metadata > 0);
    assert!(budget.try_plan_item(96));
    assert_eq!(lease.charged_memory_bytes(), metadata + 96);
    let retained = budget.clone();
    drop(budget);
    assert_eq!(lease.charged_memory_bytes(), metadata + 96);
    assert!(!retained.try_plan_item(4096));
    assert!(matches!(
        retained.stop(),
        Some(CustomPreparationStop::MemoryAdmission(
            worth_execution::MemoryLimitDenial::Capacity {
                requested, admitted: 4096,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
            }
        )) if requested > 4096
    ));
    assert_eq!(retained.claimed_memory(), 96);
    assert_eq!(lease.charged_memory_bytes(), metadata + 96);
    drop(retained);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn structural_and_direct_claims_share_the_original_live_packet_payer() {
    use crate::execution::{execute_read_only_packets, PacketMemory, ReadOnlyPacket};
    use worth_foundational::PartitionIdentity;

    let lease = crate::tests::support::test_execution_lease();
    let retained = std::sync::Mutex::new(None);
    execute_read_only_packets(
        vec![ReadOnlyPacket {
            identity: PartitionIdentity::new(1),
            value: (),
            input_bytes: 0,
            memory: PacketMemory::Tracked,
        }],
        &lease,
        |_, context| {
            context.claim_scratch(32)?;
            let payer = context
                .structural_memory()
                .expect("the real tracked packet payer");
            let budget =
                CustomPreparationBudget::packet_tracked(lease.status(), None, payer).unwrap();
            let before = lease.charged_memory_bytes();
            assert!(budget.try_plan_item(48));
            assert_eq!(lease.charged_memory_bytes(), before + 48);
            context.claim_scratch(32)?;
            assert_eq!(lease.charged_memory_bytes(), before + 48 + 32);
            // Settle the conservative temporary clone quote while retaining
            // the direct allocation and original custom structural claim.
            context.settle_temporary_scratch(960)?;
            assert_eq!(lease.charged_memory_bytes(), before + 48 + (1024 - 32));
            context.settle_temporary_scratch(0)?;
            assert_eq!(lease.charged_memory_bytes(), before + 48 + 32);
            *retained.lock().unwrap() = Some(budget.clone());
            drop(budget);
            assert_eq!(lease.charged_memory_bytes(), before + 48 + 32);
            Ok(())
        },
        |_: &()| 0,
    )
    .unwrap();
    assert!(
        lease.charged_memory_bytes() > 0,
        "the final custom holder retains its original packet ticket"
    );
    drop(retained.lock().unwrap().take());
    assert_eq!(lease.charged_memory_bytes(), 0);
}
