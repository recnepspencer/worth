use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use crate::physical_runtime::PhysicalWorkSignalFamily;

use super::{
    effect_contract::require_effect_contract, PhysicalWalAppendScope, PhysicalWalBarrierScope,
    PhysicalWalFrameWriteDisposition, PhysicalWorkDeclarationDenial,
    PhysicalWorkDurabilityRequirement, PhysicalWorkEffectClass, PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryDisposition, PhysicalWorkResourceDemand, PhysicalWorkScope,
};

#[test]
fn batch_scope_rejects_empty_and_duplicate_member_sets() {
    assert_eq!(
        PhysicalWorkScope::batch([]),
        Err(PhysicalWorkDeclarationDenial::EmptyScope)
    );
    let coordinate = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 8)
        .expect("fixture coordinate is non-empty");
    assert_eq!(
        PhysicalWorkScope::batch([coordinate]),
        Err(PhysicalWorkDeclarationDenial::BatchRequiresMultipleMembers)
    );
    assert_eq!(
        PhysicalWorkScope::batch([coordinate, coordinate]),
        Err(PhysicalWorkDeclarationDenial::DuplicateScopeMember)
    );
}

#[test]
fn batch_scope_stops_at_the_physical_member_bound() {
    let coordinates = (0..=256).map(|offset| {
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, offset * 8, 8)
            .expect("fixture coordinate is non-empty")
    });
    assert_eq!(
        PhysicalWorkScope::batch(coordinates),
        Err(PhysicalWorkDeclarationDenial::ScopeCapacityExceeded)
    );
}

#[test]
fn batch_scope_rejects_overlapping_ranges_but_accepts_touching_ranges() {
    let first = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 8).unwrap();
    let overlap = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 7, 8).unwrap();
    let touching = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap();
    assert_eq!(
        PhysicalWorkScope::batch([first, overlap]),
        Err(PhysicalWorkDeclarationDenial::OverlappingScopeMembers)
    );
    assert!(PhysicalWorkScope::batch([first, touching]).is_ok());
}

#[test]
fn batch_scope_canonicalizes_the_same_member_set_across_input_order() {
    let first = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 8).unwrap();
    let second = RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 16, 8).unwrap();
    let forward = PhysicalWorkScope::batch([first, second]).unwrap();
    let reversed = PhysicalWorkScope::batch([second, first]).unwrap();

    assert_eq!(forward, reversed);
    assert_eq!(reversed.coordinates(), &[first, second]);
}

#[test]
fn physical_demand_is_derived_from_the_exact_scope() {
    let scope = PhysicalWorkScope::one(
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 8).unwrap(),
    );
    let read = PhysicalWorkResourceDemand::derive(
        &scope,
        PhysicalWorkOperationFamily::ArtifactRangeRead,
        PhysicalWorkDurabilityRequirement::ReadOnly,
    )
    .queue_shape();
    assert_eq!(read.queue_slots(), 1);
    assert_eq!(read.worker_permits(), 1);
    assert_eq!(read.bandwidth_tokens(), 8);
    assert_eq!(read.write_back_windows(), 0);
    assert_eq!(read.flush_permits(), 0);
    assert_eq!(read.sync_debt(), 0);

    let buffered_writeback = PhysicalWorkResourceDemand::derive(
        &scope,
        PhysicalWorkOperationFamily::ArtifactRangeWrite,
        PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ),
    )
    .queue_shape();
    assert_eq!(buffered_writeback.write_back_windows(), 1);
    assert_eq!(buffered_writeback.flush_permits(), 0);
    assert_eq!(buffered_writeback.sync_debt(), 0);

    let synchronized_writeback = PhysicalWorkResourceDemand::derive(
        &scope,
        PhysicalWorkOperationFamily::ArtifactRangeWrite,
        PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
            ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
        ),
    )
    .queue_shape();
    assert_eq!(synchronized_writeback.write_back_windows(), 1);
    assert_eq!(synchronized_writeback.flush_permits(), 1);
    assert_eq!(synchronized_writeback.sync_debt(), 0);

    let publication = PhysicalWorkResourceDemand::derive(
        &scope,
        PhysicalWorkOperationFamily::ArtifactPublication,
        PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ),
    )
    .queue_shape();
    assert_eq!(publication.write_back_windows(), 0);
    assert_eq!(publication.flush_permits(), 0);
    assert_eq!(publication.sync_debt(), 1);
}

#[test]
fn wal_append_scope_and_demand_do_not_claim_a_durability_barrier() {
    let target = PhysicalWalAppendScope::new(
        3,
        7,
        128,
        64,
        PhysicalWalFrameWriteDisposition::AppendExistingSegment,
    )
    .expect("fixture WAL append interval is valid");
    let scope = PhysicalWorkScope::wal_append(target);

    assert_eq!(scope.wal_append_target(), Some(target));
    assert_eq!(scope.member_count(), 1);
    assert!(scope.coordinates().is_empty());
    assert_eq!(scope.artifact_target(), None);

    let append = PhysicalWorkResourceDemand::derive(
        &scope,
        PhysicalWorkOperationFamily::WalAppend,
        PhysicalWorkDurabilityRequirement::WalAppend,
    )
    .queue_shape();
    assert_eq!(append.queue_slots(), 1);
    assert_eq!(append.worker_permits(), 1);
    assert_eq!(append.bandwidth_tokens(), 64);
    assert_eq!(append.write_back_windows(), 0);
    assert_eq!(append.flush_permits(), 0);
    assert_eq!(append.sync_debt(), 0);

    let barrier = PhysicalWalBarrierScope::new([1; 32], [2; 32], 3, 3, 7, 128, 129, 64, 32)
        .expect("fixture WAL barrier interval is valid");
    let barrier = PhysicalWorkResourceDemand::derive(
        &PhysicalWorkScope::wal_barrier(barrier),
        PhysicalWorkOperationFamily::DurabilityBarrier,
        PhysicalWorkDurabilityRequirement::WalDurabilityBarrier,
    )
    .queue_shape();
    assert_eq!(barrier.queue_slots(), 1);
    assert_eq!(barrier.worker_permits(), 1);
    assert_eq!(barrier.bandwidth_tokens(), 1);
    assert_eq!(barrier.write_back_windows(), 0);
    assert_eq!(barrier.flush_permits(), 1);
    assert_eq!(barrier.sync_debt(), 1);
}

#[test]
fn wal_append_scope_rejects_create_for_a_nonempty_segment() {
    assert!(PhysicalWalAppendScope::new(
        1,
        1,
        128,
        64,
        PhysicalWalFrameWriteDisposition::CreateSegment,
    )
    .is_none());
    assert!(PhysicalWalAppendScope::new(
        1,
        1,
        128,
        64,
        PhysicalWalFrameWriteDisposition::AppendExistingSegment,
    )
    .is_some());
}

#[test]
fn effect_contracts_reject_read_write_category_substitution() {
    assert!(require_effect_contract(
        PhysicalWorkOperationFamily::ArtifactRangeRead,
        PhysicalWorkEffectClass::ReadOnly,
        PhysicalWorkDurabilityRequirement::ReadOnly,
        PhysicalWorkRecoveryDisposition::NoEffect,
    )
    .is_ok());
    assert_eq!(
        require_effect_contract(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkEffectClass::IdempotentExactWrite,
            PhysicalWorkDurabilityRequirement::ReadOnly,
            PhysicalWorkRecoveryDisposition::RetryExact,
        ),
        Err(PhysicalWorkDeclarationDenial::EffectfulContractMismatch)
    );
}

#[test]
fn operation_family_has_one_exact_signal_family() {
    assert_eq!(
        PhysicalWorkOperationFamily::ArtifactMetadataRead.required_signal_family(),
        PhysicalWorkSignalFamily::ReadFault
    );
    assert_eq!(
        PhysicalWorkOperationFamily::ArtifactRangeRead.required_signal_family(),
        PhysicalWorkSignalFamily::ReadFault
    );
    assert_eq!(
        PhysicalWorkOperationFamily::ArtifactRangeWrite.required_signal_family(),
        PhysicalWorkSignalFamily::ExactWriteback
    );
    assert_eq!(
        PhysicalWorkOperationFamily::ArtifactPublication.required_signal_family(),
        PhysicalWorkSignalFamily::Publication
    );
    assert_eq!(
        PhysicalWorkOperationFamily::CheckpointCapture.required_signal_family(),
        PhysicalWorkSignalFamily::CheckpointCapture
    );
    assert_eq!(
        PhysicalWorkOperationFamily::WalAppend.required_signal_family(),
        PhysicalWorkSignalFamily::WalAppend
    );
    assert_eq!(
        PhysicalWorkOperationFamily::DurabilityBarrier.required_signal_family(),
        PhysicalWorkSignalFamily::DurabilityBarrier
    );
    assert_eq!(
        PhysicalWorkOperationFamily::WalReclamation.required_signal_family(),
        PhysicalWorkSignalFamily::WalReclamation
    );
}

#[test]
fn effect_contract_matrix_has_no_implicit_success_cells() {
    let operations = [
        PhysicalWorkOperationFamily::ArtifactMetadataRead,
        PhysicalWorkOperationFamily::ArtifactRangeRead,
        PhysicalWorkOperationFamily::ArtifactRangeWrite,
        PhysicalWorkOperationFamily::ArtifactPublication,
        PhysicalWorkOperationFamily::CheckpointCapture,
        PhysicalWorkOperationFamily::WalAppend,
        PhysicalWorkOperationFamily::DurabilityBarrier,
        PhysicalWorkOperationFamily::WalReclamation,
    ];
    let effects = [
        PhysicalWorkEffectClass::ReadOnly,
        PhysicalWorkEffectClass::ReversibleBeforePublication,
        PhysicalWorkEffectClass::IdempotentExactWrite,
        PhysicalWorkEffectClass::PublicationBoundary,
    ];
    let durabilities = [
        PhysicalWorkDurabilityRequirement::ReadOnly,
        PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        ),
        PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
            ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
        ),
        PhysicalWorkDurabilityRequirement::WalAppend,
        PhysicalWorkDurabilityRequirement::WalDurabilityBarrier,
        PhysicalWorkDurabilityRequirement::CheckpointCapture,
        PhysicalWorkDurabilityRequirement::WalReclamation,
    ];
    let recoveries = [
        PhysicalWorkRecoveryDisposition::NoEffect,
        PhysicalWorkRecoveryDisposition::RetryExact,
        PhysicalWorkRecoveryDisposition::ContinueSettlement,
        PhysicalWorkRecoveryDisposition::InspectionRequired,
    ];
    for operation in operations {
        for effect in effects {
            for durability in durabilities {
                for recovery in recoveries {
                    let expected = expected_effect_cell(operation, effect, durability, recovery);
                    assert_eq!(
                        require_effect_contract(operation, effect, durability, recovery).is_ok(),
                        expected,
                        "unexpected effect-contract cell: {operation:?}/{effect:?}/{durability:?}/{recovery:?}"
                    );
                }
            }
        }
    }
}

fn expected_effect_cell(
    operation: PhysicalWorkOperationFamily,
    effect: PhysicalWorkEffectClass,
    durability: PhysicalWorkDurabilityRequirement,
    recovery: PhysicalWorkRecoveryDisposition,
) -> bool {
    let writes = matches!(
        durability,
        PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(_)
    );
    match operation {
        PhysicalWorkOperationFamily::ArtifactMetadataRead
        | PhysicalWorkOperationFamily::ArtifactRangeRead => {
            effect == PhysicalWorkEffectClass::ReadOnly
                && durability == PhysicalWorkDurabilityRequirement::ReadOnly
                && recovery == PhysicalWorkRecoveryDisposition::NoEffect
        }
        PhysicalWorkOperationFamily::ArtifactRangeWrite => {
            writes
                && ((effect == PhysicalWorkEffectClass::ReversibleBeforePublication
                    && recovery == PhysicalWorkRecoveryDisposition::InspectionRequired)
                    || (effect == PhysicalWorkEffectClass::IdempotentExactWrite
                        && matches!(
                            recovery,
                            PhysicalWorkRecoveryDisposition::RetryExact
                                | PhysicalWorkRecoveryDisposition::InspectionRequired
                        )))
        }
        PhysicalWorkOperationFamily::ArtifactPublication => {
            writes
                && effect == PhysicalWorkEffectClass::PublicationBoundary
                && matches!(
                    recovery,
                    PhysicalWorkRecoveryDisposition::ContinueSettlement
                        | PhysicalWorkRecoveryDisposition::InspectionRequired
                )
        }
        PhysicalWorkOperationFamily::CheckpointCapture => {
            effect == PhysicalWorkEffectClass::PublicationBoundary
                && durability == PhysicalWorkDurabilityRequirement::CheckpointCapture
                && recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
        }
        PhysicalWorkOperationFamily::WalAppend => {
            effect == PhysicalWorkEffectClass::ReversibleBeforePublication
                && durability == PhysicalWorkDurabilityRequirement::WalAppend
                && recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
        }
        PhysicalWorkOperationFamily::DurabilityBarrier => {
            effect == PhysicalWorkEffectClass::PublicationBoundary
                && durability == PhysicalWorkDurabilityRequirement::WalDurabilityBarrier
                && recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
        }
        PhysicalWorkOperationFamily::WalReclamation => {
            effect == PhysicalWorkEffectClass::PublicationBoundary
                && durability == PhysicalWorkDurabilityRequirement::WalReclamation
                && recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
        }
        PhysicalWorkOperationFamily::RootPublication => {
            effect == PhysicalWorkEffectClass::PublicationBoundary
                && durability == PhysicalWorkDurabilityRequirement::RootPublication
                && recovery == PhysicalWorkRecoveryDisposition::InspectionRequired
        }
    }
}
