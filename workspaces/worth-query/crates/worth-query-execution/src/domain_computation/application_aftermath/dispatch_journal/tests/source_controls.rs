//! Pure source-control frames retain monotone facts without any publication grant.
use super::*;
use InboundSourceControl::{Retired, Revoked};

#[test]
fn source_control_frames_preserve_each_source_and_refuse_revocation_downgrade() {
    let mut ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    for (source, posture) in [
        (effect(1), Retired),
        (effect(2), Revoked),
        (effect(1), Revoked),
    ] {
        let frame = ledger
            .prepare(
                QueryDispatchRecordIntent::SourceControl { source, posture },
                &mut generations,
            )
            .unwrap();
        assert!(frame.bytes().len() as u64 <= QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES);
        let decoded =
            QueryDispatchFrame::readmit(frame.bytes(), QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES)
                .unwrap();
        assert_eq!(decoded, frame);
        assert_eq!(
            QueryDispatchLedger::empty(home(2)).readmit(decoded.clone()),
            Err(Denial::ForeignHome)
        );
        ledger.readmit(decoded).unwrap();
        assert_eq!(ledger.source_control(&source), Some(posture));
    }
    assert_eq!(ledger.source_control(&effect(1)), Some(Revoked));
    assert_eq!(ledger.source_control(&effect(2)), Some(Revoked));
    assert_eq!(
        ledger
            .prepare(
                QueryDispatchRecordIntent::SourceControl {
                    source: effect(1),
                    posture: Retired
                },
                &mut generations
            )
            .map(drop),
        Err(Denial::MalformedFrame)
    );
    assert_eq!(ledger.first_unobserved(), None);
    assert_eq!(
        ledger.custody(&effect(1)),
        None,
        "source facts never create dispatch custody"
    );
}

#[test]
fn an_intact_unknown_source_control_never_reactivates_a_source() {
    let ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let frame = ledger
        .prepare(
            QueryDispatchRecordIntent::SourceControl {
                source: effect(1),
                posture: Retired,
            },
            &mut generations,
        )
        .unwrap();
    let mut payload = frame.bytes()[..frame.bytes().len() - 32].to_vec();
    *payload.last_mut().unwrap() = 0;
    assert_eq!(
        QueryDispatchFrame::readmit(&resealed(payload), QUERY_DISPATCH_FIXED_RECORD_MAX_BYTES),
        Err(Denial::MalformedFrame)
    );
}
