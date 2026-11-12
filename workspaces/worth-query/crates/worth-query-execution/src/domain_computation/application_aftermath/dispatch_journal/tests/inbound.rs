//! Original message meaning survives framing; expiry cannot discard pending work.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceClaims;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub(super) fn accepted() -> QueryInboundAcceptedMessage {
    QueryInboundAcceptedMessage {
        operation: "transfer".to_owned(),
        envelope_bytes: 192,
        claims: WorthQueryInboundOccurrenceClaims {
            audience: "original-audience".to_owned(),
            source_identity: "rail".to_owned(),
            key_epoch: 7,
            message_identity: [21; 32],
            signed_meaning_digest: [31; 32],
            issued_at_unix_seconds: 100,
            expires_at_unix_seconds: 150,
            protocol_identity: BoundaryProtocolIdentity::parse("bank.transfer".to_owned()).unwrap(),
            protocol_version: BoundaryProtocolVersion::new(1),
            correlation_family: "transfer".to_owned(),
            correlation_token: [41; 32],
            payload: vec![51; 256],
        },
        source: QueryInboundDispatchSource {
            world_slot: 1,
            world_commit: 8,
            world_attempt: 9,
            incarnation: 2,
            native_commit: 12,
            native_version: 13,
            native_branch: "main".to_owned(),
            entity: [1, 5, 1],
        },
    }
}

#[test]
fn accepted_message_retains_original_meaning_until_exact_terminal_acknowledgement_and_expiry() {
    let mut ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let original = accepted();
    let key = QueryInboundMessageKey::from_claims(&original.claims);
    let retained_bytes = QueryInboundAcceptedMessage::retained_bytes(
        &original.operation,
        &original.claims,
        original.envelope_bytes,
        &original.source.native_branch,
    )
    .unwrap();
    let frame = ledger
        .prepare(
            QueryDispatchRecordIntent::Inbound(QueryInboundRecord::Accepted(original.clone())),
            &mut generations,
        )
        .unwrap();
    assert_eq!(
        Some(retained_bytes),
        Some(frame.bytes.len() as u64 + original.envelope_bytes)
    );
    let decoded = super::super::codec::decode(frame.bytes(), retained_bytes).unwrap();
    assert_eq!(decoded, frame);
    ledger.readmit(decoded).unwrap();
    let recorded = ledger.inbound.next_after(0).unwrap();
    assert_eq!(recorded.accepted, original);
    assert_eq!(recorded.acknowledgement, None);
    assert!(ledger
        .inbound
        .next_after(recorded.acceptance_position)
        .is_none());
    let expire = QueryDispatchRecordIntent::Inbound(QueryInboundRecord::Expired {
        message: key.clone(),
    });
    assert_eq!(
        ledger.prepare(expire.clone(), &mut generations),
        Err(Denial::MalformedFrame)
    );
    let mut changed = original.clone();
    changed.claims.payload.push(99);
    assert_eq!(
        ledger.prepare(
            QueryDispatchRecordIntent::Inbound(QueryInboundRecord::Accepted(changed)),
            &mut generations
        ),
        Err(Denial::MalformedFrame)
    );
    let acknowledge = QueryDispatchRecordIntent::Inbound(QueryInboundRecord::Acknowledged {
        message: key.clone(),
        completion_world: 17,
    });
    let frame = ledger
        .prepare(acknowledge.clone(), &mut generations)
        .unwrap();
    let decoded = super::super::codec::decode(frame.bytes(), retained_bytes).unwrap();
    ledger.readmit(decoded).unwrap();
    assert_eq!(
        ledger.inbound.messages().get(&key).unwrap().acknowledgement,
        Some(17)
    );
    assert_eq!(
        ledger.prepare(acknowledge, &mut generations),
        Err(Denial::MalformedFrame)
    );
    let frame = ledger.prepare(expire, &mut generations).unwrap();
    ledger
        .readmit(super::super::codec::decode(frame.bytes(), retained_bytes).unwrap())
        .unwrap();
    assert!(ledger.inbound.messages().is_empty());
    assert!(ledger.inbound.next_after(0).is_none());
}

#[test]
fn inbound_frame_refuses_forged_lengths_truncation_and_trailing_meaning() {
    let ledger = QueryDispatchLedger::empty(home(1));
    let mut generations = QueryDispatchPublicationGenerations::from_journal(&ledger);
    let frame = ledger
        .prepare(
            QueryDispatchRecordIntent::Inbound(QueryInboundRecord::Accepted(accepted())),
            &mut generations,
        )
        .unwrap();
    let bound = frame.bytes.len() as u64 + 1;
    for length in 0..frame.bytes.len() {
        assert!(super::super::codec::decode(&frame.bytes[..length], bound).is_err());
    }
    let payload = &frame.bytes[..frame.bytes.len() - 32];
    let mut hostile_length = payload.to_vec();
    hostile_length[77..85].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(
        super::super::codec::decode(&resealed(hostile_length), bound),
        Err(Denial::MalformedFrame)
    );
    let mut trailing = payload.to_vec();
    trailing.push(0);
    assert_eq!(
        super::super::codec::decode(&resealed(trailing), bound),
        Err(Denial::MalformedFrame)
    );
    assert_eq!(
        super::super::codec::decode(frame.bytes(), frame.bytes.len() as u64 - 1),
        Err(Denial::MalformedFrame)
    );
}
