//! Drop admission at the exact Recovery boundary. The Store's retained
//! custody alone sits near the limit, with no competing hold and the
//! production 64 KiB progress headroom. Each drop is either denied before any
//! ledger effect, or it is admitted and its checkpoint runs on the standing
//! reservation alone, as does the checkpoint after it.
use super::super::super::{
    heads::{SelectedReleaseHeadRoster, SelectedReleaseHeadStep},
    PendingReleaseEvent, SelectedReleaseBatchBasis,
};
use super::*;
use worth_store_physical_format::{
    OriginalDropReservationRequestV1, PersistedRecordIdentity, ReleaseCustodyHeadBlockReferenceV1,
    ReleasedDropWalFateWitnessV1,
};

/// A committed roster large enough that it matters where it is charged.
const HEADS: u8 = 16;
const DROPPED: u8 = 100;
const FENCE: u64 = 64;
const CLOSURE: u64 = 128;
const HEADROOM: u64 = 64 << 10;

fn record(object: u8, ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([object; 16], ordinal).unwrap()
}

fn key(object: u8) -> ReleaseCustodyHeadKeyV1 {
    ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap()
}

fn entry(object: u8) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        key(object),
        record(object, 1),
        [1; 32],
        record(object, 2),
        [2; 32],
        record(object, 3),
        [3; 32],
        [4; 32],
        None,
        7,
        1,
        false,
    )
    .unwrap()
}

fn root(last: u8) -> ReleaseCustodyHeadBlockReferenceV1 {
    ReleaseCustodyHeadBlockReferenceV1::new(9, u64::from(last), 0, key(1), key(last), [last; 32])
        .unwrap()
}

fn basis() -> SelectedReleaseBatchBasis {
    SelectedReleaseBatchBasis {
        descriptor_record: record(DROPPED, 1),
        descriptor_frame_sha256: [1; 32],
        custody_digest: [5; 32],
        reservation_record: record(DROPPED, 3),
        reservation_frame_sha256: [3; 32],
        request: OriginalDropReservationRequestV1::new([1; 32], [2; 32], 4, 8).unwrap(),
        fate: ReleasedDropWalFateWitnessV1::new(1, 2, [6; 32], [7; 32]).unwrap(),
        candidate_root_generation: 9,
        candidate_root_sha256: [8; 32],
        predecessor: None,
        cumulative_dropped: 1,
        cumulative_digest: [1; 32],
        terminal: false,
    }
}

/// The committed checkpoint, its standing reservation and the attempt's
/// control backing: the retained custody alone, before the drop admission.
fn retained(fixture: &Fixture) -> Option<SelectedReleaseCustodyLedger> {
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    let roster =
        || SelectedReleaseHeadRoster::from_selected(Some(root(HEADS)), (1..=HEADS).map(entry));
    ledger.checkpoint_heads = roster().unwrap();
    ledger.effective_heads = roster().unwrap();
    ledger
        .reserve_capture_custody(&fixture.owner, fixture.ceiling, None)
        .ok()?;
    ledger
        .prepare_control_backing(&fixture.owner, fixture.ceiling, FENCE)
        .ok()?;
    Some(ledger)
}

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The retained state itself does not fit; no drop was attempted.
    RetainedDoesNotFit,
    Denied,
    /// Admitted, checkpointed, committed and checkpointed again.
    Checkpointed {
        retained: u64,
    },
}

/// Admits one drop and applies its ledger effect, or reports why not.
fn admit_drop(fixture: &Fixture, limit: u64) -> Result<SelectedReleaseCustodyLedger, Outcome> {
    let mut ledger = retained(fixture).ok_or(Outcome::RetainedDoesNotFit)?;
    let before = (ledger.pending_events.len(), ledger.effective_heads.len());
    let root_frame = match ledger.admit_drop_backing(
        &fixture.owner,
        fixture.ceiling,
        CLOSURE,
        FENCE,
        key(DROPPED),
    ) {
        Ok(root_frame) => root_frame,
        Err(Denial::Resident(_)) => {
            let after = (ledger.pending_events.len(), ledger.effective_heads.len());
            assert_eq!(after, before, "a denied drop leaves no ledger effect");
            assert_eq!(ledger.effective_heads.root(), Some(root(HEADS)));
            return Err(Outcome::Denied);
        }
        Err(other) => panic!("unexpected drop denial at {limit}: {other:?}"),
    };
    // The drop's effect performs no allocation after admission.
    let step = SelectedReleaseHeadStep::new(
        Some(root(HEADS)),
        Some(root(DROPPED)),
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next: entry(DROPPED),
        },
    );
    let transition = step.prepare(&ledger.effective_heads).unwrap();
    assert!(transition.has_backing(&ledger.effective_heads));
    assert!(ledger.pending_events.len() < ledger.pending_events.capacity());
    transition.apply(&mut ledger.effective_heads);
    ledger
        .pending_events
        .push(PendingReleaseEvent::for_drop(basis(), step).unwrap());
    drop(root_frame);
    Ok(ledger)
}

fn drop_then_checkpoint(limit: u64) -> Outcome {
    let fixture = fixture_with(limit, HEADROOM);
    let mut ledger = match admit_drop(&fixture, limit) {
        Ok(ledger) => ledger,
        Err(outcome) => return outcome,
    };
    let retained = active(&fixture);
    let capture = ledger
        .prepare_capture(&fixture.owner, fixture.ceiling, false)
        .unwrap_or_else(|denial| panic!("admitted drop must checkpoint at {limit}: {denial:?}"));
    assert_eq!(
        active(&fixture),
        retained,
        "the capture consumes the reservation"
    );
    // Commit: the fold roster replaces the committed one; the drained tail
    // leaves the same grant sized for the remaining state.
    let storage = Arc::new(SealedCheckpointStorage::from_prepared(capture));
    let mut fold = storage.lease_fold().unwrap().into_workspace();
    ledger
        .effective_heads
        .copy_into_preallocated(&mut fold.heads)
        .unwrap();
    ledger.checkpoint_heads =
        std::mem::replace(&mut fold.heads, SelectedReleaseHeadRoster::empty());
    ledger.pending_events.clear();
    drop(fold);
    drop(storage);
    assert!(
        ledger.capture_custody_requirement(None).unwrap()
            <= ledger.capture_custody_bytes().unwrap()
    );
    drop(
        ledger
            .prepare_capture(&fixture.owner, fixture.ceiling, true)
            .unwrap_or_else(|denial| panic!("the next checkpoint must fit at {limit}: {denial:?}")),
    );
    assert_eq!(active(&fixture), retained);
    Outcome::Checkpointed { retained }
}

#[test]
fn drop_at_the_boundary_is_denied_before_effects_or_checkpoints() {
    let ample = fixture_with(1 << 20, HEADROOM);
    let admitted = admit_drop(&ample, 1 << 20).expect("an ample pool admits the drop");
    let retained = active(&ample);
    drop(admitted);
    // The first limit whose usable bytes hold exactly the retained custody.
    let boundary = retained + HEADROOM;
    assert_eq!(drop_then_checkpoint(boundary - 1), Outcome::Denied);
    assert_eq!(
        drop_then_checkpoint(boundary),
        Outcome::Checkpointed { retained }
    );
    // Either way across a wider band, never an admitted drop that cannot
    // checkpoint (each admitted case asserts its own checkpoints).
    for limit in (boundary - 4096..boundary + 512).step_by(16) {
        let outcome = drop_then_checkpoint(limit);
        assert_eq!(
            matches!(outcome, Outcome::Checkpointed { .. }),
            limit >= boundary,
            "limit {limit}: {outcome:?}"
        );
    }
}
