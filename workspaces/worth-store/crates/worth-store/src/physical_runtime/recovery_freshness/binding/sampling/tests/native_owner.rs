use super::super::super::{
    manifest_cleanup::Observation, operations::RecoveryBindingOperationsCapacity,
    StoreRecoveryOperationEvidence, StoreRecoveryRetirementObligation, StoreRecoveryWalMember,
};
use super::super::group_validation::RecoveryWalGroupBinding;
use super::*;
use crate::physical_runtime::{
    durability::{RetiredArtifact, RetirementRecord},
    PhysicalRecoveryRejoinResidentDenial,
};
use std::mem::size_of;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

fn interval(start: u64) -> WalLsnRange {
    WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(start + 1),
    )
    .unwrap()
}

fn capacity() -> SamplingCapacity {
    let operations = RecoveryBindingOperationsCapacity::for_records(2).unwrap();
    let requested = operations.requested_bytes()
        + (2 * size_of::<StoreRecoveryWalMember>()) as u64
        + (2 * size_of::<RecoveryWalGroupBinding>()) as u64
        + (2 * size_of::<usize>()) as u64
        + (2 * size_of::<(usize, RetirementRecord)>()) as u64
        + (2 * size_of::<StoreRecoveryRetirementObligation>()) as u64
        + (2 * size_of::<(WalLsnRange, Vec<u8>)>()) as u64
        + (2 * size_of::<Observation>()) as u64
        + 64
        + 32;
    SamplingCapacity {
        operations,
        members: 2,
        retirements: 2,
        copies: 2,
        cleanups: 2,
        requested,
    }
}

#[test]
fn native_sample_owner_funds_all_retained_capacities_and_releases_scratch_then_result() {
    let mut disposed_observer = None;
    with_owner(|coordination, media, evidence| {
        let observer = coordination.certification_residency_allocations();
        let dimension = Dimension::OperationScope(Scope::Recovery);
        with_empty_checkpoint(media.store_identity(), |assembly| {
            let shared = coordination.admit_shared_checkpoint(assembly).unwrap();
            let facts = shared.facts();
            let basis = {
                let mut window = coordination.begin_source_read_allocation().unwrap();
                window
                    .begin_checkpoint_binding_rebuild(&shared, 0)
                    .unwrap()
                    .finish()
                    .unwrap()
            };
            let baseline = observer.snapshot().for_dimension(dimension).active_units();
            let requested = capacity().requested;
            let held = coordination
                .certification_begin_recovery_allocation(
                    NonZeroU64::new(ORIGINAL - baseline - requested + 1).unwrap(),
                )
                .unwrap();
            let allocation = SamplingAllocation::from_coordination(coordination).unwrap();
            assert!(allocation.matches_checkpoint_basis(&basis));
            let before = observer.snapshot().for_dimension(dimension);
            // This actual reserve precedes every SamplingStorage Vec preparation.
            assert_eq!(
                allocation.reserve(requested).unwrap_err(),
                AllocationDenial::Backing {
                    requested,
                    cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                        required: ORIGINAL + 1,
                        admitted: ORIGINAL,
                    },
                }
            );
            let denied = observer.snapshot().for_dimension(dimension);
            assert_eq!(denied.admissions(), before.admissions());
            assert_eq!(denied.admitted_units(), before.admitted_units());
            assert_eq!(denied.denials(), before.denials() + 1);
            drop(held);

            let backing = allocation.reserve(requested).unwrap();
            let mut storage = SamplingStorage::prepare(capacity(), 2, ORIGINAL, &backing).unwrap();
            storage.operations.merge(evidence).unwrap();
            let mut redo = super::super::storage::copy_bytes(&[9; 64]).unwrap();
            redo.truncate(7);
            let redo_pointer = redo.as_ptr();
            storage.redo_bytes = redo.len() as u64;
            storage.members.push(StoreRecoveryWalMember {
                lsn_range: interval(3),
                operation_identity: [1; 32],
                group_identity: [2; 32],
                group_member_identity: [3; 32],
                group_member_ordinal: 1,
                group_member_count: 1,
                group_membership_digest: [4; 32],
                canonical_redo: redo,
            });
            storage.retirement_records.push((
                0,
                RetirementRecord {
                    artifact: RetiredArtifact::Segment {
                        segment: 2,
                        generation: 1,
                    },
                    completion: false,
                    source_root: 1,
                    bytes: 128,
                    release: None,
                },
            ));
            let mut copy = super::super::storage::copy_bytes(&[8; 32]).unwrap();
            copy.truncate(9);
            let copy_pointer = copy.as_ptr();
            storage.copies.push((interval(4), copy));
            let cleanup = BlobManifestResidueCleanupV1::intent(
                media.store_identity().bytes(),
                [2; 16],
                PersistedRecordIdentity::new([3; 16], 4).unwrap(),
                [4; 32],
                [5; 32],
                [6; 32],
                [7; 32],
                1,
                2,
                [8; 32],
                16,
                1,
            )
            .unwrap()
            .into();
            storage
                .cleanups
                .observe(interval(5), cleanup, media.store_identity().bytes(), 2)
                .unwrap();
            let known_scratch = (storage.groups.capacity() * size_of::<RecoveryWalGroupBinding>()
                + storage.group_scratch.capacity() * size_of::<usize>()
                + storage.retirement_records.capacity() * size_of::<(usize, RetirementRecord)>())
                as u64;
            assert!(known_scratch > 0);
            let source = SamplingSource {
                store: media.store_identity(),
                generation: facts.compaction_cutover().product_generation(),
                cutoff: facts.compaction_cutover().wal_cutoff_lsn_exclusive(),
                context: PhysicalBindingDecodingContext::new(
                    media.store_identity(),
                    PhysicalDurabilityPolicyIdentity::from_recovery_binding([7; 32]),
                    PhysicalIdempotencyPolicy::from_recovery_binding(NonZeroU64::new(1).unwrap()),
                ),
                sealed_basis_digest: facts.source().security_binding().unwrap().digest(),
                policy_identity: [7; 32],
                basis: &basis,
                evidence: &[],
            };
            let before_finish = observer.snapshot().for_dimension(dimension);
            let sample = super::super::finish_sample(backing, storage, &source).unwrap();
            // Independent oracle: actual owned Vec capacities, including inline
            // roster allocation and nested payload spare capacity; no production
            // owned_heap_bytes/roster_heap_bytes sizing is used for this expectation.
            let expected = (sample.operations.capacity()
                * size_of::<StoreRecoveryOperationEvidence>()
                + sample.wal_members.capacity() * size_of::<StoreRecoveryWalMember>()
                + sample.retirements.capacity() * size_of::<StoreRecoveryRetirementObligation>()
                + sample.extent_copy_frames.capacity() * size_of::<(WalLsnRange, Vec<u8>)>()
                + sample.blob_manifest_residue_cleanups.capacity() * size_of::<Observation>()
                + sample
                    .wal_members
                    .iter()
                    .map(|member| member.canonical_redo.capacity())
                    .sum::<usize>()
                + sample
                    .extent_copy_frames
                    .iter()
                    .map(|(_, bytes)| bytes.capacity())
                    .sum::<usize>()) as u64;
            assert_eq!(sample.operations.len(), 1);
            assert_eq!(sample.wal_members.len(), 1);
            assert_eq!(sample.retirements.len(), 1);
            assert_eq!(sample.extent_copy_frames.len(), 1);
            assert_eq!(sample.blob_manifest_residue_cleanups.len(), 1);
            for (len, capacity) in [
                (sample.operations.len(), sample.operations.capacity()),
                (sample.wal_members.len(), sample.wal_members.capacity()),
                (sample.retirements.len(), sample.retirements.capacity()),
                (
                    sample.extent_copy_frames.len(),
                    sample.extent_copy_frames.capacity(),
                ),
                (
                    sample.blob_manifest_residue_cleanups.len(),
                    sample.blob_manifest_residue_cleanups.capacity(),
                ),
            ] {
                assert!(capacity > len);
            }
            assert_eq!(sample.wal_members[0].canonical_redo.as_ptr(), redo_pointer);
            assert_eq!(sample.extent_copy_frames[0].1.as_ptr(), copy_pointer);
            assert!(
                sample.wal_members[0].canonical_redo.capacity()
                    > sample.wal_members[0].canonical_redo.len()
            );
            assert!(
                sample.extent_copy_frames[0].1.capacity() > sample.extent_copy_frames[0].1.len()
            );
            assert!(
                requested - expected > known_scratch,
                "lookup scratch also disappears"
            );
            assert_eq!(sample.charged_bytes(), expected);
            let retained = observer.snapshot().for_dimension(dimension);
            assert_eq!(retained.active_units(), baseline + expected);
            assert_eq!(
                retained.released_units() - before_finish.released_units(),
                requested - expected
            );
            drop(sample);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                baseline
            );
            drop(basis);
            drop(shared);
            assert_eq!(
                observer.snapshot().for_dimension(dimension).active_units(),
                0
            );
        });
        let final_bytes = observer.snapshot().for_dimension(dimension);
        assert_eq!(final_bytes.admitted_units(), final_bytes.released_units());
        disposed_observer = Some(observer);
    });
    let disposed = disposed_observer
        .unwrap()
        .snapshot()
        .for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}
