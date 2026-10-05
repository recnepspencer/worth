//! A denied head observation names a limit only where recovery ran out of
//! one: a refused charge, or a reader out of recovery's observation bytes.

use worth_store::physical_runtime::{FilesystemObservationBound, RecoveryDiscoveryFailure};
use worth_store_physical_format::PersistedRecordIdentity;
use worth_store_physical_integrity::{
    release_custody_head_walk_limit_for_test, ReleaseCustodyHeadWalkBound,
};
use worth_store_recovery_physics::SelectedCustodyDenial;

use super::*;
use crate::entry::{
    PhysicalRecoveryPageAdmissionDenial as Page,
    PhysicalRecoveryReleaseHeadControlDenial as Control,
    PhysicalRecoverySelectedRecordReadDenial as RecordRead,
};
use crate::orchestration::reader_limit::refused_past;

fn outgrown(bound: FilesystemObservationBound) -> RecoveryDiscoveryFailure {
    refused_past(bound, 4_001, 4_000)
}

fn head_block() -> worth_store_physical_format::ReleaseCustodyHeadBlockReferenceV1 {
    let key = worth_store_physical_format::ReleaseCustodyHeadKeyV1::new([7; 16], 1).unwrap();
    worth_store_physical_format::ReleaseCustodyHeadBlockReferenceV1::new(5, 2, 0, key, key, [9; 32])
        .expect("a leaf reference")
}

fn control_read(denial: RecordRead) -> Denial {
    Denial::Control(Control::ControlRead {
        record: PersistedRecordIdentity::new([7; 16], 3).expect("nonzero record identity"),
        denial,
    })
}

#[test]
fn a_refused_charge_is_the_manifest_entry_limit() {
    for denial in [
        Denial::ManifestEntryLimit,
        Denial::SourceRoutes(Page::ManifestEntryLimit),
        Denial::Control(Control::ManifestEntryLimit),
        control_read(RecordRead::ManifestEntryLimit),
        Denial::SourceRootRead {
            generation: 5,
            failure: refused_past(FilesystemObservationBound::Reads, 1, 0),
        },
    ] {
        assert_eq!(
            unread(&denial),
            Some(HistoricalFailure::ManifestEntries),
            "{denial:?}",
        );
    }
}

#[test]
fn a_roster_of_more_heads_than_recovery_admits_is_the_entry_limit_with_its_count() {
    use crate::entry::{PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure};
    let limits = crate::entry::PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 1 << 20,
        manifest_entries: 40,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 1 << 20,
        redo_targets: 4096,
        redo_bytes: 1 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 1 << 20,
        recovery_memory_bytes: 1 << 20,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 1 << 20,
        observation_bytes: 10_000,
    };
    // Physics names its bound and the roster's count; neither is relabeled.
    let mut resident = ResidentAllowance::new(8);
    let denial = roster_refused(
        SelectedHeadRosterAdmissionDenial::HeadEntries {
            observed: 57,
            admitted: 40,
        },
        &mut resident,
    );
    assert_eq!(
        denial,
        Denial::RosterEntryLimit {
            observed: 57,
            admitted: 40,
        }
    );
    assert_eq!(
        limit_of(&denial, &limits, 0),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ManifestEntries,
            observed: 57,
            admitted: 40,
        })
    );
    // Every other denial names its limit as before.
    assert_eq!(
        limit_of(&Denial::ManifestEntryLimit, &limits, 0),
        HistoricalFailure::ManifestEntries.limit(&limits, 0)
    );
    assert_eq!(limit_of(&Denial::DuplicateClaim, &limits, 0), None);
    // A tree of more heads than that roster counts is damage, not that limit.
    let overfull = roster_refused(
        SelectedHeadRosterAdmissionDenial::Walk(ReleaseCustodyHeadWalkDenial::Visit(())),
        &mut resident,
    );
    assert_eq!(overfull, Denial::HeadWalk(WalkDenial::EntryCountExceeded));
    assert_eq!(limit_of(&overfull, &limits, 0), None);
    // Nor is a tree of more blocks than that roster's heads can fill: the
    // ceiling is the roster's, and recovery sets no limit on blocks to name.
    let sprawling = roster_refused(
        SelectedHeadRosterAdmissionDenial::Walk(ReleaseCustodyHeadWalkDenial::Limit(
            release_custody_head_walk_limit_for_test(ReleaseCustodyHeadWalkBound::Nodes, 9, 7),
        )),
        &mut resident,
    );
    assert_eq!(
        sprawling,
        Denial::HeadWalk(WalkDenial::RosterBlockCeiling {
            observed: 9,
            admitted: 7,
        })
    );
    assert_eq!(limit_of(&sprawling, &limits, 0), None);
}

#[test]
fn a_reader_out_of_observation_bytes_is_that_limit_with_the_count_it_reached() {
    let observation = FilesystemObservationBound::ObservationBytes;
    for denial in [
        Denial::ObservationByteLimit,
        Denial::SourceRoutes(Page::ObservationByteLimit),
    ] {
        assert_eq!(
            unread(&denial),
            Some(HistoricalFailure::ObservationBytes(None)),
            "{denial:?}",
        );
    }
    for denial in [
        Denial::SourceRootRead {
            generation: 5,
            failure: outgrown(observation),
        },
        // Head blocks are bounded by the bytes they cost, not by a count.
        Denial::HeadWalk(WalkDenial::Read(ReadDenial::Media {
            reference: head_block(),
            failure: outgrown(observation),
        })),
        control_read(RecordRead::ManifestRead(outgrown(observation))),
        control_read(RecordRead::ChunkRead {
            ordinal: 2,
            failure: outgrown(observation),
        }),
    ] {
        assert_eq!(
            unread(&denial),
            Some(HistoricalFailure::ObservationBytes(Some(4_001))),
            "{denial:?}",
        );
    }
}

#[test]
fn a_head_that_failed_verification_names_no_limit() {
    let requested = FilesystemObservationBound::RequestedBytes;
    for denial in [
        Denial::Roster(SelectedCustodyDenial::CertificateRoster),
        Denial::HeadWalk(WalkDenial::BoundExceeded),
        Denial::HeadWalk(WalkDenial::DuplicateNode),
        // A tree of more heads than the verified roster counts.
        Denial::HeadWalk(WalkDenial::EntryCountExceeded),
        Denial::SourceRootFormatMismatch,
        Denial::SourceRootRead {
            generation: 5,
            failure: outgrown(requested),
        },
        control_read(RecordRead::ManifestRead(outgrown(requested))),
        control_read(RecordRead::InvalidPayload),
    ] {
        assert!(
            matches!(unread(&denial), None | Some(HistoricalFailure::Invalid)),
            "{denial:?}",
        );
    }
}

/// Physics refused the roster past this phase's resident window: that bound,
/// with both counts. Its retained ceiling is the roster's own denial.
#[test]
fn a_roster_past_its_resident_window_is_the_resident_bound_with_both_counts() {
    use worth_store_recovery_physics::{test_support::physics_limit_for_test, PhysicsBound};
    let mut resident = ResidentAllowance::new(8);
    let past = physics_limit_for_test(PhysicsBound::ResidentBytes, 11, 6);
    assert_eq!(
        roster_refused(
            SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::Limit(past)),
            &mut resident,
        ),
        Denial::ResidentBoundExceeded {
            required: 11,
            admitted: 6,
        }
    );
    let retained = physics_limit_for_test(PhysicsBound::RetainedBytes, 11, 6);
    assert_eq!(
        roster_refused(
            SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::Limit(retained)),
            &mut resident,
        ),
        Denial::Roster(SelectedCustodyDenial::Limit(retained))
    );
}

#[test]
fn source_routes_this_phase_had_no_room_to_hold_are_its_resident_bound() {
    use crate::orchestration::planning::page_observation::PageObservationFailure;
    let mut resident = ResidentAllowance::new(8);
    assert!(resident.bytes(9).is_err());
    let held = routes_denial(
        RoutesFailure::Held(ResidentTraceDenial::ResidentBoundExceeded),
        &resident,
    );
    assert_eq!(
        held,
        Denial::ResidentBoundExceeded {
            required: 9,
            admitted: 8,
        }
    );
    assert_eq!(unread(&held), None, "the resident allowance names it");
    assert_eq!(
        routes_denial(
            RoutesFailure::Observation(PageObservationFailure::ManifestEntryLimit),
            &resident,
        ),
        Denial::SourceRoutes(Page::ManifestEntryLimit)
    );
    let cause = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();
    assert_eq!(
        routes_denial(
            RoutesFailure::Held(ResidentTraceDenial::Allocation {
                requested: 5,
                cause: cause.clone(),
            }),
            &resident,
        ),
        Denial::HeadWalk(WalkDenial::Allocation {
            requested: 5,
            cause,
        })
    );
}
