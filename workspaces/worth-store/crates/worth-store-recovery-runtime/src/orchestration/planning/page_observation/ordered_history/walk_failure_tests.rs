use super::*;
use crate::entry::PhysicalRecoveryLimitDimension;
use crate::entry::PhysicalRecoveryLimitDimension::{
    ManifestEntries, ObservationBytes, StagingBytes,
};
use crate::orchestration::reader_limit::refused_past;
use crate::orchestration::recovery_budget::{allowance_for_test, recovery_limit_for_test};
use worth_store::physical_runtime::{FilesystemObservationBound, RecoveryDiscoveryArtifact};
use worth_store_recovery_physics::{
    test_support::{
        head_replay_limit_for_test, physics_limit_for_test, root_history_limit_for_test,
    },
    HeadReplayBound, PhysicalRedoTargetIdentity, PhysicsBound, RootHistoryBound,
};

/// The walk's 100 bytes of staging.
const STAGING: RecoveryAllowance = allowance_for_test(StagingBytes, 100);

fn limit(dimension: PhysicalRecoveryLimitDimension, observed: u64, admitted: u64) -> WalkFailure {
    WalkFailure::recovery(recovery_limit_for_test(dimension, observed, admitted))
}

/// A reader handed 65,536 of recovery's 65,540 observation bytes needed
/// one more: 65,541 in all.
fn assert_reader_out_of_bytes(failure: WalkFailure) {
    let WalkFailure::Limit(past) = failure else {
        panic!("a reader out of observation bytes is a limit, not {failure:?}");
    };
    assert_eq!(
        past.in_recovery(
            &crate::entry::PhysicalRecoveryLimitDeclaration::observing_for_test(65_540)
        ),
        Some(recovery_limit_for_test(ObservationBytes, 65_541, 65_540).into()),
    );
}

#[test]
fn a_failed_read_is_a_limit_only_when_the_reader_ran_out_of_observation_bytes() {
    let oversized = |bound| refused_past(bound, 65_537, 65_536);
    assert_reader_out_of_bytes(WalkFailure::from(oversized(
        FilesystemObservationBound::ObservationBytes,
    )));
    for damage in [
        // The walk's readers count no reads: a refused read is damage.
        refused_past(FilesystemObservationBound::Reads, 1, 0),
        // A root or routing block larger than one page is damaged media.
        oversized(FilesystemObservationBound::RequestedBytes),
        RecoveryDiscoveryFailure::InvalidAddress {
            artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
        },
    ] {
        assert_eq!(WalkFailure::from(damage), WalkFailure::Unverified);
    }
}

#[test]
fn only_an_exhausted_limit_or_a_lost_count_stops_the_walk_short() {
    for stopped in [
        PageObservationFailure::Limit(PageLimit::Recovery(recovery_limit_for_test(
            StagingBytes,
            9,
            8,
        ))),
        PageObservationFailure::CountOverflow,
    ] {
        assert_eq!(WalkFailure::from(stopped.clone()).stopped(), Some(stopped));
    }
    let target = PhysicalRedoTargetIdentity::InlinePage {
        segment: 1,
        page: 2,
        generation: 3,
    };
    for damage in [
        PageObservationFailure::InvalidTarget(target),
        PageObservationFailure::InvalidPage(target),
    ] {
        assert_eq!(WalkFailure::from(damage), WalkFailure::Unverified);
    }
    assert_eq!(WalkFailure::Unverified.stopped(), None);
}

#[test]
fn a_refusal_proven_is_unverified() {
    assert_eq!(Some(7).proven(), Ok(7));
    assert_eq!(None::<u8>.proven(), Err(WalkFailure::Unverified));
    assert_eq!(Err::<u8, ()>(()).proven(), Err(WalkFailure::Unverified));
}

#[test]
fn the_walks_scratch_states_the_whole_need() {
    assert_eq!(WalkFailure::left(STAGING, 40), Ok(60));
    assert_eq!(
        WalkFailure::left(STAGING, 110),
        Err(limit(StagingBytes, 110, 100))
    );
    // 60 left of 100: the walk holds 40, and needed 70 more.
    assert_eq!(WalkFailure::take(STAGING, 60, 50), Ok(10));
    assert_eq!(
        WalkFailure::take(STAGING, 60, 70),
        Err(limit(StagingBytes, 110, 100))
    );
    assert_eq!(WalkFailure::hold(STAGING, 100), Ok(()));
    assert_eq!(
        WalkFailure::hold(STAGING, 101),
        Err(limit(StagingBytes, 101, 100))
    );
    assert_eq!(
        WalkFailure::past_scratch(u64::MAX, 60, STAGING),
        WalkFailure::CountOverflow
    );
}

#[test]
fn a_bound_physics_ran_past_is_that_limit_with_what_it_needed() {
    let mut budget = ManifestEntryBudget::new(10, 4);
    let past =
        |bound, observed, admitted| Some(root_history_limit_for_test(bound, observed, admitted));
    assert_eq!(
        WalkFailure::refused(None, &mut budget, STAGING),
        WalkFailure::Unverified
    );
    assert_eq!(budget.refused(), None);
    // Physics had 60 of the walk's 100 bytes and needed 70: 110 in all.
    let scratch = past(RootHistoryBound::ScratchBytes, 70, 60);
    assert_eq!(
        WalkFailure::refused(scratch, &mut budget, STAGING),
        limit(StagingBytes, 110, 100)
    );
    assert_eq!(budget.refused(), None);
    // One view, handed all 10 entries, held 11.
    let entries = past(RootHistoryBound::Entries, 11, 10);
    assert_eq!(
        WalkFailure::refused(entries, &mut budget, STAGING),
        limit(ManifestEntries, 11, 10)
    );
    assert_eq!(
        budget.refused(),
        Some(recovery_limit_for_test(ManifestEntries, 11, 10))
    );
}

#[test]
fn a_control_physics_would_not_retain_is_the_scratch_the_walk_needed() {
    use AddressedReleasedControlDenial as Control;
    let past = physics_limit_for_test(PhysicsBound::RetainedBytes, 70, 60);
    assert_eq!(
        WalkFailure::control_refused(Control::Bound(past), STAGING),
        limit(StagingBytes, 110, 100)
    );
    assert_eq!(
        WalkFailure::control_refused(Control::CountOverflow, STAGING),
        WalkFailure::CountOverflow
    );
    for damage in [
        Control::ResultRoot,
        Control::Route,
        Control::Frame,
        Control::Allocation,
    ] {
        assert_eq!(
            WalkFailure::control_refused(damage, STAGING),
            WalkFailure::Unverified
        );
    }
}

#[test]
fn a_replay_bound_is_the_scratch_the_walk_needed_in_all() {
    use SelectedReleaseHeadReplayDenial as Replay;
    // Handed 60 of 100 bytes, the replay needed 70: the walk held 40.
    for bound in [HeadReplayBound::EffectBytes, HeadReplayBound::HeapBytes] {
        let past = head_replay_limit_for_test(bound, 70, 60);
        assert_eq!(
            WalkFailure::replay_refused(Replay::BoundExceeded(past), STAGING),
            limit(StagingBytes, 110, 100)
        );
    }
    assert_eq!(
        WalkFailure::replay_refused(Replay::SizeOverflow, STAGING),
        WalkFailure::CountOverflow
    );
    for damage in [
        Replay::NotAdmittedUpsert,
        Replay::NotAdmittedTerminalHeadRetirement,
        Replay::SourceRoot,
        Replay::SourcePath,
        Replay::Read,
    ] {
        assert_eq!(
            WalkFailure::replay_refused(damage, STAGING),
            WalkFailure::Unverified
        );
    }
}

#[test]
fn a_control_record_the_walk_could_not_read_keeps_its_limit() {
    use PhysicalRecoverySelectedRecordReadDenial as Denial;
    let mut budget = ManifestEntryBudget::new(10, 4);
    // Handed 60 of the walk's 100 bytes, the ledger was asked for 70.
    let mut resident = ResidentAllowance::new(60);
    assert!(resident.bytes(70).is_err());
    let unread = |denial, budget: &ManifestEntryBudget| {
        WalkFailure::unread(denial, budget, &resident, STAGING)
    };
    // A budget that refused nothing cannot say the limit's counts.
    assert_eq!(
        unread(Denial::ManifestEntryLimit, &budget),
        WalkFailure::CountOverflow
    );
    assert!(budget.charge(7).is_err());
    assert_eq!(
        unread(Denial::ManifestEntryLimit, &budget),
        limit(ManifestEntries, 11, 10)
    );
    assert_eq!(
        unread(Denial::ResidentBoundExceeded, &budget),
        limit(StagingBytes, 110, 100)
    );
    let out_of_bytes = refused_past(FilesystemObservationBound::ObservationBytes, 65_537, 65_536);
    assert_reader_out_of_bytes(unread(Denial::ManifestRead(out_of_bytes), &budget));
    assert_eq!(
        unread(Denial::InvalidPayload, &budget),
        WalkFailure::Unverified
    );
}
