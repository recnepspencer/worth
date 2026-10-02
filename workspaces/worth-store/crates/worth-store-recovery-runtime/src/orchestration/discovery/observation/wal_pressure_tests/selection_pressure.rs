//! Genuine C9 inputs exercise native candidate and tail-storage boundaries.
//! The local frontier is a WAL sub-boundary, not a substitute Store seal.

use super::*;
use crate::orchestration::discovery::wal::{discover_wal_inventory, WalDiscoveryInventory};
use crate::orchestration::wal_selection::{
    CandidateClassificationDenial, RecordedWalDisposition, ResidentWalCandidates,
    WalTailSelectionDenial,
};
use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, RecoveryWalAllocationDenial as AllocationDenial,
};
use worth_store_recovery_physics::{
    AdmittedWalFrameRejectionKind, CheckpointCoveredWalArtifact, PhysicalWalFrameFacts,
    PhysicalWalSegmentCandidate,
};

pub(super) fn deny_then_retry(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    observed: &[ObservedWalArtifact],
    healthy: &mut WalDiscoveryInventory,
) {
    let reads_before = discovery.counters().wal_bytes_read;
    candidate_frame_pressure(coordination, healthy);
    tail_partition_pressure(discovery, coordination, limits, observed, healthy);
    assert_eq!(
        discovery.counters().wal_bytes_read,
        reads_before,
        "candidate/tail retry reuses actual C4 buffers without rereading payloads",
    );
}

fn candidate_frame_pressure(coordination: &RecoveryCoordination, healthy: &WalDiscoveryInventory) {
    let observer = coordination.owner().certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let baseline = observer.snapshot().for_dimension(dimension).active_units();
    let segment = healthy
        .admitted
        .cleanup_segments(
            healthy
                .candidates
                .iter()
                .map(PhysicalWalSegmentCandidate::identity),
        )
        .next()
        .expect("healthy discovery retains an actual C9-admitted segment");
    assert!(!segment.frames().is_empty());
    let inspection = segment.inspection();
    let frame_bytes =
        (segment.frames().len() * std::mem::size_of::<PhysicalWalFrameFacts>()) as u64;
    let mut candidates = ResidentWalCandidates::prepare(coordination.owner(), 1).unwrap();
    let slot_bytes =
        (candidates.capacity() * std::mem::size_of::<PhysicalWalSegmentCandidate>()) as u64;
    assert_eq!(candidates.charged_bytes(), slot_bytes);
    assert_eq!(candidates.len(), 0);
    let active = observer.snapshot().for_dimension(dimension).active_units();
    assert_eq!(active, baseline + slot_bytes);
    let held = coordination
        .owner()
        .certification_begin_recovery_allocation(
            NonZeroU64::new(ORIGINAL - active - frame_bytes + 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    // Policy-negative twins borrow the same genuine admitted prefix. A known
    // corrupt disposition must not attempt to materialize additional facts.
    for (terminal, rejection) in [
        (true, AdmittedWalFrameRejectionKind::Other),
        (false, AdmittedWalFrameRejectionKind::Truncated),
    ] {
        assert!(
            matches!(
                candidates.classify(
                    coordination.owner(),
                    inspection.identity(),
                    inspection.byte_count(),
                    terminal,
                    Some(rejection),
                    Some(&segment),
                ),
                Ok(RecordedWalDisposition::Corrupt)
            ),
            "known corrupt prefix must reject before fact backing admission"
        );
        assert_eq!(candidates.len(), 0);
        assert_eq!(candidates.charged_bytes(), slot_bytes);
        assert_eq!(segment.inspection(), inspection);
        let rejected = observer.snapshot().for_dimension(dimension);
        assert_eq!(rejected.admissions(), before.admissions());
        assert_eq!(rejected.admitted_units(), before.admitted_units());
        assert_eq!(rejected.released_units(), before.released_units());
        assert_eq!(rejected.active_units(), before.active_units());
        assert_eq!(rejected.denials(), before.denials());
    }
    let cause = match candidates.classify(
        coordination.owner(),
        inspection.identity(),
        inspection.byte_count(),
        true,
        None,
        Some(&segment),
    ) {
        Err(CandidateClassificationDenial::Allocation(cause)) => cause,
        _ => panic!("actual candidate facts must deny native backing before vector allocation"),
    };
    assert_eq!(
        cause,
        AllocationDenial::Backing {
            requested: slot_bytes + frame_bytes,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                required: ORIGINAL + 1,
                admitted: ORIGINAL,
            },
        }
    );
    assert_eq!(candidates.len(), 0);
    assert_eq!(candidates.charged_bytes(), slot_bytes);
    assert_eq!(segment.inspection(), inspection);
    let denied = observer.snapshot().for_dimension(dimension);
    assert_eq!(denied.admissions(), before.admissions());
    assert_eq!(denied.admitted_units(), before.admitted_units());
    assert_eq!(denied.released_units(), before.released_units());
    assert_eq!(denied.active_units(), before.active_units());
    assert_eq!(denied.denials(), before.denials() + 1);
    drop(held);
    match candidates.classify(
        coordination.owner(),
        inspection.identity(),
        inspection.byte_count(),
        true,
        None,
        Some(&segment),
    ) {
        Ok(RecordedWalDisposition::Candidate {
            inspection: actual,
            torn_bytes: 0,
        }) => assert_eq!(actual, inspection),
        _ => panic!("same actual C9 segment must classify after native pressure is released"),
    }
    let actual_frame_bytes: u64 = candidates
        .iter()
        .map(|candidate| {
            (candidate.frame_fact_capacity() * std::mem::size_of::<PhysicalWalFrameFacts>()) as u64
        })
        .sum();
    assert_eq!(actual_frame_bytes, frame_bytes);
    assert_eq!(candidates.charged_bytes(), slot_bytes + actual_frame_bytes);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        baseline + slot_bytes + actual_frame_bytes
    );
    drop(candidates);
    drop(segment);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        baseline
    );
}

fn tail_partition_pressure(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    observed: &[ObservedWalArtifact],
    healthy: &mut WalDiscoveryInventory,
) {
    let observer = coordination.owner().certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let candidates = std::mem::take(&mut healthy.candidates);
    let count = candidates.len();
    assert!(count > 0);
    let frontier = candidates
        .iter()
        .next()
        .unwrap()
        .inspection()
        .lsn_range()
        .start()
        .get();
    let slot_bytes =
        (candidates.capacity() * std::mem::size_of::<PhysicalWalSegmentCandidate>()) as u64;
    let frame_bytes: u64 = candidates
        .iter()
        .map(|candidate| {
            (candidate.frame_fact_capacity() * std::mem::size_of::<PhysicalWalFrameFacts>()) as u64
        })
        .sum();
    let input_bytes = slot_bytes + frame_bytes;
    assert_eq!(candidates.charged_bytes(), input_bytes);
    let covered_bytes = (count * std::mem::size_of::<CheckpointCoveredWalArtifact>()) as u64;
    let retained_bytes = (count * std::mem::size_of::<PhysicalWalSegmentCandidate>()) as u64;
    let baseline = observer.snapshot().for_dimension(dimension).active_units();
    let held_bytes = ORIGINAL - baseline - covered_bytes - retained_bytes + 1;
    let held = coordination
        .owner()
        .certification_begin_recovery_allocation(NonZeroU64::new(held_bytes).unwrap())
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let cause = match candidates.select_tail(coordination.owner(), frontier, None) {
        Err(WalTailSelectionDenial::Allocation(cause)) => cause,
        _ => panic!("second prepared tail workspace must deny, after the first actually admits"),
    };
    assert_eq!(
        cause,
        AllocationDenial::Backing {
            requested: input_bytes + covered_bytes + retained_bytes,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                required: ORIGINAL + 1,
                admitted: ORIGINAL,
            },
        }
    );
    let denied = observer.snapshot().for_dimension(dimension);
    assert_eq!(denied.admissions(), before.admissions() + 1);
    assert_eq!(
        denied.admitted_units() - before.admitted_units(),
        covered_bytes
    );
    assert_eq!(
        denied.released_units() - before.released_units(),
        input_bytes + covered_bytes
    );
    assert_eq!(denied.denials(), before.denials() + 1);
    assert_eq!(denied.active_units(), baseline - input_bytes + held_bytes);
    assert_eq!(healthy.candidates.len(), 0);
    drop(held);
    let remaining = baseline - input_bytes;
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        remaining
    );

    let mut retry = match discover_wal_inventory(
        coordination.owner(),
        observed,
        discovery.store_identity(),
        limits.declaration().wal_frames,
    ) {
        Ok(inventory) => inventory,
        Err(_) => panic!("same real raw inputs must rebuild after tail pressure disposal"),
    };
    let candidates = std::mem::take(&mut retry.candidates);
    let input_bytes = candidates.charged_bytes();
    let first = candidates.iter().next().unwrap();
    // When actual media supplies a second frame, exercise trimming while retaining
    // the complete original frame-vector capacity. Otherwise the pure Physics
    // prepared-storage test supplies that semantic case.
    let trim = first
        .frame_facts()
        .get(1)
        .map(|frame| frame.lsn_range().start().get());
    let frontier = trim.unwrap_or_else(|| first.inspection().lsn_range().start().get());
    let before_tail = observer.snapshot().for_dimension(dimension).active_units();
    let tail = match candidates.select_tail(coordination.owner(), frontier, None) {
        Ok(tail) => tail,
        Err(_) => panic!("healthy WAL sub-boundary must preserve its actual contiguous tail"),
    };
    let facts = tail.facts();
    let actual_frame_bytes: u64 = facts
        .segments()
        .iter()
        .map(|candidate| {
            (candidate.frame_fact_capacity() * std::mem::size_of::<PhysicalWalFrameFacts>()) as u64
        })
        .sum();
    let expected = actual_frame_bytes
        + (facts.segment_capacity() * std::mem::size_of::<PhysicalWalSegmentCandidate>()) as u64
        + (facts.checkpoint_covered_capacity()
            * std::mem::size_of::<CheckpointCoveredWalArtifact>()) as u64;
    assert_eq!(
        facts.frame_count(),
        retry.valid_frames - u64::from(trim.is_some())
    );
    assert_eq!(tail.charged_bytes(), expected);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        before_tail - input_bytes + expected
    );
    drop(tail);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        before_tail - input_bytes
    );
    drop(retry);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        remaining
    );
}
