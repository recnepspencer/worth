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
    PhysicalRecoveryLimitDimension::{self, ManifestEntries, ObservationBytes, StagingBytes},
    PhysicalRecoveryReleaseHeadControlDenial as Control,
    PhysicalRecoverySelectedRecordReadDenial as RecordRead,
};
use crate::orchestration::planning::manifest_entry_budget::{
    manifest_entry_limit_for_test, EntryAdmission,
};
use crate::orchestration::planning::page_observation::{PageLimit, PageObservationFailure};
use crate::orchestration::planning::selected_source_inventory::{
    ResidentTraceDenial, RoutesFailure,
};
use crate::orchestration::reader_limit::{refused_past, ReaderBytes};
use crate::orchestration::recovery_budget::recovery_limit_for_test;

fn outgrown(bound: FilesystemObservationBound) -> RecoveryDiscoveryFailure {
    refused_past(bound, 4_001, 4_000)
}

fn limits() -> crate::entry::PhysicalRecoveryLimitDeclaration {
    crate::entry::PhysicalRecoveryLimitDeclaration {
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
    }
}

fn limit(
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
) -> Option<crate::entry::PhysicalRecoveryLimitFailure> {
    Some(recovery_limit_for_test(dimension, observed, admitted).into())
}

/// The limit a denial the phase met states through the allowance that refused.
fn stated(
    denial: &Denial,
    limits: &crate::entry::PhysicalRecoveryLimitDeclaration,
    budget: &ManifestEntryBudget,
) -> Option<crate::entry::PhysicalRecoveryLimitFailure> {
    unread(denial)?.limit(limits, budget)
}

/// A phase's budget of all 40 entries that refused a charge of 5 with 38
/// already charged: 43 of 40.
fn refused_budget() -> ManifestEntryBudget {
    let mut budget = ManifestEntryBudget::for_test(40, 38);
    assert!(budget.admit(5).is_err());
    budget
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
        Denial::Control(Control::ManifestEntryLimit),
        control_read(RecordRead::ManifestEntryLimit),
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
    let limits = limits();
    // A refused charge is the budget's own refusal, with its counts.
    assert_eq!(
        stated(&Denial::ManifestEntryLimit, &limits, &refused_budget()),
        limit(ManifestEntries, 43, 40)
    );
    let mut budget = refused_budget();
    let mut resident = ResidentAllowance::new(8);
    let mut refused = |observed, admitted, budget: &mut ManifestEntryBudget| {
        roster_refused(
            SelectedHeadRosterAdmissionDenial::HeadEntries { observed, admitted },
            &mut resident,
            budget,
        )
    };
    // Handed all 40 entries, the roster held 57. The denial names no count:
    // the budget holds both.
    let denial = refused(57, 40, &mut budget);
    assert_eq!(denial, Denial::RosterEntryLimit);
    assert_eq!(
        stated(&denial, &limits, &budget),
        limit(ManifestEntries, 57, 40)
    );
    // Handed 10 of the 40, the roster needed 12: recovery held 30 beside it.
    let narrower = refused(12, 10, &mut budget);
    assert_eq!(
        stated(&narrower, &limits, &budget),
        limit(ManifestEntries, 42, 40)
    );
    // A roster handed more than recovery admits is no part of its limit.
    let wider = refused(57, 41, &mut budget);
    assert_eq!(stated(&wider, &limits, &budget), None);
    assert_eq!(stated(&Denial::DuplicateClaim, &limits, &budget), None);
    // A tree of more heads than that roster counts is damage, not that limit.
    let overfull = roster_refused(
        SelectedHeadRosterAdmissionDenial::Walk(ReleaseCustodyHeadWalkDenial::Visit(())),
        &mut resident,
        &mut budget,
    );
    assert_eq!(overfull, Denial::HeadWalk(WalkDenial::EntryCountExceeded));
    assert_eq!(stated(&overfull, &limits, &budget), None);
    // Nor is a tree of more blocks than that roster's heads can fill: the
    // ceiling is the roster's, and recovery sets no limit on blocks to name.
    let sprawling = roster_refused(
        SelectedHeadRosterAdmissionDenial::Walk(ReleaseCustodyHeadWalkDenial::Limit(
            release_custody_head_walk_limit_for_test(ReleaseCustodyHeadWalkBound::Nodes, 9, 7),
        )),
        &mut resident,
        &mut budget,
    );
    assert_eq!(
        sprawling,
        Denial::HeadWalk(WalkDenial::RosterBlockCeiling {
            observed: 9,
            admitted: 7,
        })
    );
    assert_eq!(stated(&sprawling, &limits, &budget), None);
}

#[test]
fn a_reader_out_of_observation_bytes_is_that_limit_with_the_count_it_reached() {
    let (limits, budget) = (limits(), refused_budget());
    let observation = FilesystemObservationBound::ObservationBytes;
    // A reader out of bytes refused with its own counts, beside the denial:
    // the denial invents none.
    assert_eq!(unread(&Denial::ObservationByteLimit), None);
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
        // Handed 4,000 of 10,000 bytes, the reader reached 4,001.
        assert_eq!(
            stated(&denial, &limits, &budget),
            limit(ObservationBytes, 10_001, 10_000),
            "{denial:?}",
        );
    }
}

#[test]
fn a_head_that_failed_verification_names_no_limit() {
    // A count the reader keeps of its own is no limit of recovery's.
    let reads = FilesystemObservationBound::Reads;
    for denial in [
        Denial::Roster(SelectedCustodyDenial::CertificateRoster),
        Denial::HeadWalk(WalkDenial::BoundExceeded),
        Denial::HeadWalk(WalkDenial::DuplicateNode),
        // A tree of more heads than the verified roster counts.
        Denial::HeadWalk(WalkDenial::EntryCountExceeded),
        Denial::SourceRootFormatMismatch,
        Denial::SourceRootRead {
            generation: 5,
            failure: outgrown(reads),
        },
        control_read(RecordRead::ManifestRead(outgrown(reads))),
        control_read(RecordRead::InvalidPayload),
    ] {
        assert!(
            matches!(unread(&denial), None | Some(HistoricalFailure::Invalid)),
            "{denial:?}",
        );
    }
}

/// Physics refused the roster, or the walk under it, past this phase's
/// resident window: the allowance holds the whole need, and the denial names
/// no count. Its retained ceiling is the roster's own denial.
#[test]
fn a_roster_past_its_resident_window_is_the_resident_bound_with_both_counts() {
    use worth_store_recovery_physics::{test_support::physics_limit_for_test, PhysicsBound};
    let memory = crate::orchestration::recovery_budget::allowance_for_test(
        PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
        20,
    );
    let mut budget = refused_budget();
    let past = physics_limit_for_test(PhysicsBound::ResidentBytes, 11, 6);
    let walk =
        release_custody_head_walk_limit_for_test(ReleaseCustodyHeadWalkBound::ResidentBytes, 13, 6);
    for (denial, public, need) in [
        (
            SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::Limit(past)),
            Denial::ResidentBoundExceeded,
            11,
        ),
        (
            SelectedHeadRosterAdmissionDenial::Walk(ReleaseCustodyHeadWalkDenial::Limit(walk)),
            Denial::HeadWalk(WalkDenial::ResidentBoundExceeded),
            13,
        ),
    ] {
        // Handed 6 of 8, with 2 held: the window needed `need` more.
        let mut resident = ResidentAllowance::new(8);
        resident.bytes(2).unwrap();
        assert_eq!(roster_refused(denial, &mut resident, &mut budget), public);
        assert_eq!(unread(&public), None, "the resident allowance names it");
        let limit = resident.refused_in(memory).expect("a resident limit");
        assert_eq!(
            (limit.dimension(), limit.observed(), limit.admitted()),
            (
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
                need + 2 + 12,
                20
            )
        );
    }
    let retained = physics_limit_for_test(PhysicsBound::RetainedBytes, 11, 6);
    assert_eq!(
        roster_refused(
            SelectedHeadRosterAdmissionDenial::Custody(SelectedCustodyDenial::Limit(retained)),
            &mut ResidentAllowance::new(8),
            &mut budget,
        ),
        Denial::Roster(SelectedCustodyDenial::Limit(retained))
    );
}

#[test]
fn source_routes_this_phase_had_no_room_to_hold_are_its_resident_bound() {
    let mut resident = ResidentAllowance::new(8);
    assert!(resident.bytes(9).is_err());
    let held = routes_denial(RoutesFailure::Held(
        ResidentTraceDenial::ResidentBoundExceeded,
    ));
    assert_eq!(held, Refused::from(Denial::ResidentBoundExceeded));
    assert_eq!(resident.exceeded_requirement(), Some(9));
    assert_eq!(
        unread(&held.denial),
        None,
        "the resident allowance names it"
    );
    let cause = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();
    assert_eq!(
        routes_denial(RoutesFailure::Held(ResidentTraceDenial::Allocation {
            requested: 5,
            cause: cause.clone(),
        }),),
        Refused::from(Denial::HeadWalk(WalkDenial::Allocation {
            requested: 5,
            cause,
        }))
    );
}

/// A limit the routes met keeps its own counts beside the public denial
/// that names it; damage keeps none.
#[test]
fn source_routes_out_of_a_limit_keep_its_counts() {
    let routes = |limit| {
        routes_denial(RoutesFailure::Observation(PageObservationFailure::Limit(
            limit,
        )))
    };
    let entries = PageLimit::Entries(manifest_entry_limit_for_test(11, 10));
    let reader = PageLimit::Reader(
        ReaderBytes::of(&outgrown(FilesystemObservationBound::ObservationBytes)).unwrap(),
    );
    let staging = PageLimit::Recovery(recovery_limit_for_test(StagingBytes, 9, 8));
    for (limit, denial) in [
        (entries, Denial::ManifestEntryLimit),
        (reader, Denial::ObservationByteLimit),
        (staging, Denial::WalkLimits),
    ] {
        assert_eq!(
            routes(limit),
            Refused {
                denial,
                limit: Some(limit),
            }
        );
    }
    let target = worth_store_recovery_physics::PhysicalRedoTargetIdentity::InlinePage {
        segment: 1,
        page: 2,
        generation: 3,
    };
    assert_eq!(
        routes_denial(RoutesFailure::Observation(
            PageObservationFailure::InvalidPage(target)
        ),),
        Refused::from(Denial::SourceRoutes(
            crate::entry::PhysicalRecoveryPageAdmissionDenial::InvalidPage(target)
        ))
    );
}
