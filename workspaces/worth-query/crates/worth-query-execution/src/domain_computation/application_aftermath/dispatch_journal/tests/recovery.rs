//! Recorded claims have one generation, preserved deadline and a closed terminal grammar.
use super::*;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableWorldOwnerIdentity,
    RecoveryClaimDisposition as Terminal, WorldOwnerSlotIdentity,
};
fn claim() -> QueryRecoveryClaim {
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home(1), WorldOwnerSlotIdentity::from_recorded(2));
    QueryRecoveryClaim {
        original: DurableCompositeCommitIdentity::from_recorded(owner, 7),
        attempt: DurableCompositePublicationAttemptIdentity::from_recorded(owner, 8),
        expires_at_unix_ms: 987_654,
        material_pin: CorrectionMaterialPinIdentity::from_recorded(
            home(1),
            std::num::NonZeroU64::new(9).unwrap(),
        ),
    }
}
#[test]
fn claim_frames_keep_original_owner_deadline_and_pin_without_process_identity() {
    let mut ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let original = claim();
    let frame = ledger
        .prepare(
            QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Claim(original)),
            &mut generations,
        )
        .unwrap();
    assert_eq!(
        frame.bytes().len() as u64,
        QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES
    );
    let decoded =
        QueryDispatchFrame::readmit(frame.bytes(), QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES).unwrap();
    assert_eq!(decoded, frame);
    ledger.readmit(decoded).unwrap();
    let recorded = ledger.recovery.claim(&original.original).unwrap();
    assert_eq!(recorded.claim, original);
    assert_eq!(recorded.identity.position().get(), 1);
    assert!(recorded.terminal.is_none());
    assert_eq!(
        ledger.prepare(
            QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Claim(original)),
            &mut generations
        ),
        Err(Denial::MalformedFrame)
    );
}
#[test]
fn every_claim_terminal_roundtrips_and_only_relinquishment_permits_another_generation() {
    for terminal in [
        Terminal::Consumed,
        Terminal::Completed,
        Terminal::Expired,
        Terminal::Disposed,
        Terminal::ForceTerminated,
        Terminal::Relinquished,
    ] {
        let mut ledger = QueryDispatchLedger::empty(home(1));
        let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
        let original = claim();
        let frame = ledger
            .prepare(
                QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Claim(original)),
                &mut generations,
            )
            .unwrap();
        ledger.readmit(frame).unwrap();
        let old = ledger.recovery.claim(&original.original).unwrap().identity;
        let record = QueryRecoveryRecord::Disposition {
            claim: old,
            terminal,
        };
        let frame = ledger
            .prepare(
                QueryDispatchRecordIntent::Recovery(record),
                &mut generations,
            )
            .unwrap();
        ledger
            .readmit(
                QueryDispatchFrame::readmit(frame.bytes(), QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            ledger.recovery.claim(&original.original).unwrap().terminal,
            Some(terminal)
        );
        assert_eq!(
            ledger.prepare(
                QueryDispatchRecordIntent::Recovery(record),
                &mut generations
            ),
            Err(Denial::MalformedFrame)
        );
        let next = ledger.prepare(
            QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Claim(original)),
            &mut generations,
        );
        if terminal == Terminal::Relinquished {
            ledger.readmit(next.unwrap()).unwrap();
            let current = ledger.recovery.claim(&original.original).unwrap();
            assert_ne!(current.identity, old);
            assert!(current.terminal.is_none());
            assert_eq!(
                ledger.prepare(
                    QueryDispatchRecordIntent::Recovery(record),
                    &mut generations
                ),
                Err(Denial::MalformedFrame)
            );
        } else {
            assert_eq!(next, Err(Denial::MalformedFrame));
        }
    }
}

#[test]
fn zero_based_world_ordinals_roundtrip_without_becoming_dispatch_positions() {
    let mut original = claim();
    let owner = original.original.owner();
    original.original = DurableCompositeCommitIdentity::from_recorded(owner, 0);
    original.attempt = DurableCompositePublicationAttemptIdentity::from_recorded(owner, 0);
    let mut ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let frame = ledger
        .prepare(
            QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Claim(original)),
            &mut generations,
        )
        .unwrap();
    let decoded =
        QueryDispatchFrame::readmit(frame.bytes(), QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES).unwrap();
    ledger.readmit(decoded).unwrap();
    let recorded = ledger.recovery.claim(&original.original).unwrap();
    assert_eq!(recorded.claim, original);
    assert_eq!(recorded.identity.position().get(), 1);
}
