//! Pure frame and ledger laws; the custody owner's court drives real requests.
use super::*;
use sha2::Digest;
use QueryDispatchJournalDenial as Denial;
use QueryDispatchRecordedObservation as Observation;

fn home(byte: u8) -> HomeIdentity {
    HomeIdentity::from_observed_bytes([byte; 16]).unwrap()
}
fn effect(byte: u8) -> CanonicalDigestId {
    CanonicalDigestId::new([byte; 32])
}

/// Every observation the journal can record. The exhaustive match stops
/// compiling when a variant is added, so a new variant cannot skip the codec.
fn every_observation() -> Vec<Observation> {
    use ExternalEffectNoEffectCause as NoEffect;
    use QueryDispatchUnresolvedCause as Cause;
    let listed = |observation: Observation| match observation {
        Observation::Completed
        | Observation::Acknowledged
        | Observation::ProvenNoEffect(NoEffect::PayloadRejected | NoEffect::ContactNotStarted)
        | Observation::Unresolved(
            Cause::Timeout
            | Cause::Disconnect
            | Cause::LostResponse
            | Cause::DuplicatedAcknowledgement
            | Cause::UnsupportedProtocolVersion
            | Cause::UnknownProviderOutcome
            | Cause::ObservationDerivationDenied,
        ) => observation,
    };
    [
        Observation::Completed,
        Observation::Acknowledged,
        Observation::ProvenNoEffect(NoEffect::PayloadRejected),
        Observation::ProvenNoEffect(NoEffect::ContactNotStarted),
        Observation::Unresolved(Cause::Timeout),
        Observation::Unresolved(Cause::Disconnect),
        Observation::Unresolved(Cause::LostResponse),
        Observation::Unresolved(Cause::DuplicatedAcknowledgement),
        Observation::Unresolved(Cause::UnsupportedProtocolVersion),
        Observation::Unresolved(Cause::UnknownProviderOutcome),
        Observation::Unresolved(Cause::ObservationDerivationDenied),
    ]
    .map(listed)
    .to_vec()
}

/// A ledger holding one recorded attempt for `effect(1)`, and that attempt.
fn attempted() -> (
    QueryDispatchLedger,
    QueryDispatchPublicationGenerations,
    DispatchAttemptIdentity,
) {
    let mut ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let intent = QueryDispatchRecordIntent::Attempt {
        correlation: effect(1),
    };
    let frame = ledger.prepare(intent, &mut generations).unwrap();
    let QueryDispatchRecord::Attempt(attempt) = *frame.record() else {
        panic!("an attempt intent encodes an attempt record");
    };
    ledger.readmit(frame).unwrap();
    (ledger, generations, attempt.identity)
}

/// Seal an arbitrary payload with its valid checksum.
fn resealed(mut payload: Vec<u8>) -> Vec<u8> {
    let checksum: [u8; 32] = sha2::Sha256::digest(&payload).into();
    payload.extend_from_slice(&checksum);
    payload
}

#[test]
fn every_observation_survives_its_frame_and_decides_the_next_attempt() {
    for observation in every_observation() {
        let (mut ledger, mut generations, attempt) = attempted();
        let intent = QueryDispatchRecordIntent::Observation {
            attempt,
            observation,
        };
        let frame = ledger.prepare(intent, &mut generations).unwrap();
        assert!(frame.bytes().len() as u64 <= QUERY_DISPATCH_FRAME_MAX_BYTES);
        let decoded = QueryDispatchFrame::readmit(frame.bytes()).unwrap();
        assert_eq!(decoded, frame, "{observation:?}");
        ledger.readmit(decoded).unwrap();
        let custody = ledger.custody(&effect(1)).unwrap();
        assert_eq!(custody.attempt(), attempt);
        assert_eq!(custody.observation(), Some(observation));
        assert_eq!(ledger.first_unobserved(), None);
        let again = QueryDispatchRecordIntent::Attempt {
            correlation: effect(1),
        };
        let next = ledger.prepare(again, &mut generations).map(drop);
        match observation {
            Observation::Completed => assert_eq!(next, Err(Denial::EffectAlreadyCompleted)),
            Observation::Acknowledged
            | Observation::ProvenNoEffect(_)
            | Observation::Unresolved(_) => assert_eq!(next, Ok(()), "{observation:?}"),
        }
    }
}

#[test]
fn an_attempt_awaits_exactly_one_observation() {
    let (mut ledger, mut generations, attempt) = attempted();
    assert_eq!(
        ledger.first_unobserved(),
        Some(Denial::UnobservedAttempt {
            attempt,
            effect: effect(1),
        })
    );
    let second_attempt = QueryDispatchRecordIntent::Attempt {
        correlation: effect(1),
    };
    assert_eq!(
        ledger.prepare(second_attempt, &mut generations).map(drop),
        Err(Denial::AttemptAwaitingObservation)
    );
    let observed = QueryDispatchRecordIntent::Observation {
        attempt,
        observation: Observation::Acknowledged,
    };
    let frame = ledger.prepare(observed, &mut generations).unwrap();
    ledger.readmit(frame).unwrap();
    assert_eq!(
        ledger.prepare(observed, &mut generations).map(drop),
        Err(Denial::AttemptAlreadyObserved)
    );
    let never_recorded = QueryDispatchRecordIntent::Observation {
        attempt: DispatchAttemptIdentity::from_recorded(
            home(1),
            std::num::NonZeroU64::new(99).unwrap(),
        ),
        observation: Observation::Completed,
    };
    assert_eq!(
        ledger.prepare(never_recorded, &mut generations).map(drop),
        Err(Denial::UnknownAttempt)
    );
}

#[test]
fn frames_refuse_corruption_foreign_homes_and_broken_prefixes() {
    let ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let attempt = QueryDispatchRecordIntent::Attempt {
        correlation: effect(1),
    };
    let frame = ledger.prepare(attempt, &mut generations).unwrap();
    assert_eq!(
        frame.bytes().len() as u64,
        QUERY_DISPATCH_FRAME_MAX_BYTES,
        "an attempt is the largest version-one frame"
    );
    let mut corrupt = frame.bytes().to_vec();
    corrupt[80] ^= 1;
    assert_eq!(QueryDispatchFrame::readmit(&corrupt), Err(Denial::Checksum));
    let decoded = QueryDispatchFrame::readmit(frame.bytes()).unwrap();
    assert_eq!(decoded, frame);
    assert_eq!(
        QueryDispatchLedger::empty(home(2)).readmit(decoded.clone()),
        Err(Denial::ForeignHome)
    );
    let mut advanced = QueryDispatchLedger::empty(home(1));
    advanced.readmit(decoded.clone()).unwrap();
    assert_eq!(advanced.readmit(decoded), Err(Denial::PrefixMismatch));
}

#[test]
fn intact_frames_of_another_shape_or_version_name_their_own_refusal() {
    let ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let attempt = QueryDispatchRecordIntent::Attempt {
        correlation: effect(1),
    };
    let frame = ledger.prepare(attempt, &mut generations).unwrap();
    let payload = &frame.bytes()[..frame.bytes().len() - 32];
    for length in 0..frame.bytes().len() {
        assert!(
            matches!(
                QueryDispatchFrame::readmit(&frame.bytes()[..length]),
                Err(Denial::MalformedFrame | Denial::Checksum)
            ),
            "a {length}-byte prefix is never a frame"
        );
    }
    let mut trailing = payload.to_vec();
    trailing.push(0);
    assert_eq!(
        QueryDispatchFrame::readmit(&resealed(trailing)),
        Err(Denial::MalformedFrame),
        "this version's size law refuses one byte more"
    );
    let mut foreign_magic = payload.to_vec();
    foreign_magic[0] ^= 1;
    assert_eq!(
        QueryDispatchFrame::readmit(&resealed(foreign_magic)),
        Err(Denial::MalformedFrame)
    );
    let mut later_version = payload.to_vec();
    later_version[8..12].copy_from_slice(&2u32.to_le_bytes());
    later_version.extend_from_slice(&[0; 64]);
    assert_eq!(
        QueryDispatchFrame::readmit(&resealed(later_version)),
        Err(Denial::UnsupportedVersion(2)),
        "an intact later version reports itself, whatever its size"
    );
    let mut unknown_record = payload[..76].to_vec();
    unknown_record.push(7);
    assert_eq!(
        QueryDispatchFrame::readmit(&resealed(unknown_record)),
        Err(Denial::MalformedFrame)
    );
    let mut unknown_observation = payload[..76].to_vec();
    unknown_observation.push(1);
    unknown_observation.extend_from_slice(&1u64.to_le_bytes());
    unknown_observation.extend_from_slice(&[9, 0]);
    assert_eq!(
        QueryDispatchFrame::readmit(&resealed(unknown_observation)),
        Err(Denial::MalformedFrame)
    );
}
