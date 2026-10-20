use super::*;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialKind, ExecutionResourceLease,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateResourceCeiling,
};

#[test]
fn candidate_funding_preserves_native_first_interruption_and_final_drop() {
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1_000_000),
                Some(1_000_000),
            ),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let requirements = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(1, 0, 0, 0, 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(128),
    );
    let baseline = lease.charged_memory_bytes();
    let mut funding = CandidateFunding::admit(
        requirements,
        None,
        ExecutionAllocationPolicy::Execution(&lease),
    )
    .unwrap();
    let entries = super::super::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::entry_storage_bound(1).unwrap();
    let before = lease.charged_memory_bytes();
    let blocked = lease
        .reserve_memory(
            lease
                .available_memory_bytes()
                .expect("this fixture supplies a finite memory ceiling")
                - entries
                + 1,
        )
        .unwrap();
    let refused = funding.grow_correspondence(entries).unwrap_err();
    assert!(matches!(
        refused.allocation_denial().unwrap().kind(),
        ExecutionAllocationDenialKind::Lease(_)
    ));
    drop(blocked);
    assert_eq!(lease.charged_memory_bytes(), before);
    funding.grow_correspondence(entries).unwrap();
    let funding = std::sync::Arc::new(funding);
    let clone = funding.clone();
    let held = lease
        .reserve_memory(
            lease
                .available_memory_bytes()
                .expect("this fixture supplies a finite memory ceiling"),
        )
        .unwrap();
    let cancelled = CancellationSource::new();
    let stopped = lease.controlled_child(
        cancelled.token(),
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
    );
    let deadline = CandidateFunding::admit(
        requirements,
        None,
        ExecutionAllocationPolicy::Execution(&stopped),
    )
    .unwrap_err();
    assert_eq!(
        deadline.allocation_denial().unwrap().kind(),
        ExecutionAllocationDenialKind::DeadlineElapsed
    );
    cancelled.cancel();
    let cancellation = CandidateFunding::admit(
        requirements,
        None,
        ExecutionAllocationPolicy::Execution(&stopped),
    )
    .unwrap_err();
    assert_eq!(
        cancellation.allocation_denial().unwrap().kind(),
        ExecutionAllocationDenialKind::Cancelled
    );
    drop(held);
    drop(funding);
    assert!(lease.charged_memory_bytes() > baseline);
    drop(clone);
    assert_eq!(lease.charged_memory_bytes(), baseline);
    for deadline_case in [false, true] {
        let cancellation = CancellationSource::new();
        let deadline =
            deadline_case.then(|| std::time::Instant::now() + std::time::Duration::from_secs(1));
        let original = lease.controlled_child(cancellation.token(), deadline);
        let mut admitted = CandidateFunding::admit(
            requirements,
            None,
            ExecutionAllocationPolicy::Execution(&original),
        )
        .expect("original allocation control is live at admission");
        let prior = admitted._memory.as_ref().unwrap().bytes();
        let prior_charge = lease.charged_memory_bytes();
        let competing = lease
            .reserve_memory(
                lease
                    .available_memory_bytes()
                    .expect("this fixture supplies a finite memory ceiling"),
            )
            .unwrap();
        let exhausted_charge = lease.charged_memory_bytes();
        assert_eq!(
            lease
                .available_memory_bytes()
                .expect("this fixture supplies a finite memory ceiling"),
            0
        );
        let expected = if let Some(deadline) = deadline {
            std::thread::sleep(deadline.saturating_duration_since(std::time::Instant::now()));
            assert!(original.deadline_elapsed());
            ExecutionAllocationDenialKind::DeadlineElapsed
        } else {
            cancellation.cancel();
            assert!(original.is_cancelled());
            ExecutionAllocationDenialKind::Cancelled
        };
        assert!(!lease.is_cancelled() && !lease.deadline_elapsed());
        // The same native grant retains the stopped allocation node; no
        // borrowed lease or live parent can substitute for its controls.
        drop(original);
        let refused = admitted.grow_correspondence(entries).unwrap_err();
        let cause = refused.allocation_denial().unwrap();
        assert_eq!(cause.kind(), expected);
        assert_eq!(cause.requested_payload_bytes(), Some(prior + entries));
        assert_eq!(admitted._memory.as_ref().unwrap().bytes(), prior);
        assert_eq!(lease.charged_memory_bytes(), exhausted_charge);
        drop(competing);
        assert_eq!(lease.charged_memory_bytes(), prior_charge);
        let admitted = std::sync::Arc::new(admitted);
        let clone = admitted.clone();
        drop(admitted);
        assert_eq!(lease.charged_memory_bytes(), prior_charge);
        drop(clone);
        assert_eq!(lease.charged_memory_bytes(), baseline);
    }
    drop(
        CandidateFunding::admit(
            requirements,
            None,
            ExecutionAllocationPolicy::Execution(&lease),
        )
        .unwrap(),
    );
    assert_eq!(lease.charged_memory_bytes(), baseline);
}
