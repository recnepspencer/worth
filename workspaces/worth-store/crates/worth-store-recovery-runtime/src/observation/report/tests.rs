use sha2::{Digest, Sha256};

use super::super::{
    protocol, RecoveryReportCounters, RecoveryReportDecodeDenial, RECOVERY_REPORT_PROTOCOL,
};
use super::model::{RecoveryReportBlockCause, RecoveryReportDenialCause};
use super::{RecoveryReportEnvelope, RecoveryReportOutcome};
use crate::{PhysicalRecoveryOutcome, PhysicalRecoveryRefusal, PhysicalRecoveryRefusalKind};
use worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture;

#[test]
fn report_round_trip_wrong_family_future_version_and_digest_are_distinct() {
    let outcome = PhysicalRecoveryOutcome::Refused(PhysicalRecoveryRefusal::new(
        PhysicalRecoveryRefusalKind::CoordinationUnavailable,
        0,
    ));
    let encoded = RecoveryReportEnvelope::from_outcome(&outcome).encode();
    assert_eq!(
        RecoveryReportEnvelope::decode(&encoded).unwrap().outcome(),
        RecoveryReportOutcome::Refused
    );

    let mut wrong_family = encoded.clone();
    wrong_family[8] = b'x';
    refresh_digest(&mut wrong_family);
    assert_eq!(
        RecoveryReportEnvelope::decode(&wrong_family),
        Err(RecoveryReportDecodeDenial::WrongProtocolFamily)
    );

    let mut future = encoded.clone();
    let version_offset = 8 + RECOVERY_REPORT_PROTOCOL.as_str().len();
    future[version_offset..version_offset + 4].copy_from_slice(&2_u32.to_le_bytes());
    refresh_digest(&mut future);
    let Err(RecoveryReportDecodeDenial::UnsupportedVersion(unsupported)) =
        RecoveryReportEnvelope::decode(&future)
    else {
        panic!("future version must be typed")
    };
    assert_eq!(
        unsupported.posture(),
        BoundaryProtocolUnsupportedVersionPosture::ExceedsWindow
    );

    assert_eq!(
        RecoveryReportEnvelope::decode(&encoded[..31]),
        Err(RecoveryReportDecodeDenial::Malformed)
    );

    let payload_end = encoded.len() - 32;
    let header_end = 8 + RECOVERY_REPORT_PROTOCOL.as_str().len() + 4;
    for payload_length in header_end..payload_end {
        let truncated = rehashed_prefix(&encoded, payload_length);
        assert_eq!(
            RecoveryReportEnvelope::decode(&truncated),
            Err(RecoveryReportDecodeDenial::Malformed),
            "field-boundary truncation at {payload_length}"
        );
    }

    let mut trailing = encoded[..payload_end].to_vec();
    trailing.push(0xaa);
    trailing.extend_from_slice(&Sha256::digest(&trailing));
    assert_eq!(
        RecoveryReportEnvelope::decode(&trailing),
        Err(RecoveryReportDecodeDenial::Malformed)
    );

    let mut malformed = encoded.clone();
    let outcome_offset = version_offset + 4;
    malformed[outcome_offset] = 0;
    refresh_digest(&mut malformed);
    assert_eq!(
        RecoveryReportEnvelope::decode(&malformed),
        Err(RecoveryReportDecodeDenial::Malformed)
    );

    for optional_offset in [outcome_offset + 1, outcome_offset + 2] {
        let mut malformed_optional = encoded.clone();
        malformed_optional[optional_offset] = 2;
        refresh_digest(&mut malformed_optional);
        assert_eq!(
            RecoveryReportEnvelope::decode(&malformed_optional),
            Err(RecoveryReportDecodeDenial::Malformed),
            "invalid optional flag at {optional_offset}"
        );
    }

    let mut damaged = encoded;
    damaged[20] ^= 1;
    assert_eq!(
        RecoveryReportEnvelope::decode(&damaged),
        Err(RecoveryReportDecodeDenial::DigestMismatch)
    );
}

#[test]
fn publication_indeterminate_is_a_distinct_terminal_report() {
    let report = RecoveryReportEnvelope {
        outcome: RecoveryReportOutcome::PublicationIndeterminate,
        store: Some([0x42; 16]),
        root_generation: None,
        counters: RecoveryReportCounters::new(17, 0, 0, 0),
        denial_cause: Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate),
    };
    let decoded = RecoveryReportEnvelope::decode(&report.encode())
        .expect("publication-indeterminate report must round-trip");
    assert_eq!(
        decoded.outcome(),
        RecoveryReportOutcome::PublicationIndeterminate
    );
    assert_eq!(decoded.store_identity(), Some([0x42; 16]));
    assert_eq!(decoded.root_generation(), None);
    assert_eq!(decoded.counters().recovery_effects(), 17);
    assert_eq!(
        decoded.denial_cause(),
        Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate)
    );
}

#[test]
fn selected_custody_block_cause_round_trips_without_reusing_an_older_code() {
    let report = RecoveryReportEnvelope {
        outcome: RecoveryReportOutcome::Blocked,
        store: Some([0x42; 16]),
        root_generation: None,
        counters: RecoveryReportCounters::new(0, 0, 0, 0),
        denial_cause: Some(RecoveryReportDenialCause::Blocked(
            RecoveryReportBlockCause::SelectedCustody,
        )),
    };
    let decoded = RecoveryReportEnvelope::decode(&report.encode()).unwrap();
    assert_eq!(decoded.denial_cause(), report.denial_cause);
}

#[test]
fn literal_v1_report_rejects_header_and_field_truncation_and_all_optional_flags() {
    let (encoded, optional_flags) = literal_v1_report();
    RecoveryReportEnvelope::decode(&encoded).expect("literal v1 recovery report");
    let payload_end = encoded.len() - 32;
    for length in 0..encoded.len() {
        if length == payload_end {
            continue;
        }
        assert_eq!(
            RecoveryReportEnvelope::decode(&rehashed_prefix(&encoded, length)),
            Err(RecoveryReportDecodeDenial::Malformed),
            "literal v1 truncation at {length}"
        );
    }
    for offset in optional_flags {
        let mut malformed = encoded.clone();
        malformed[offset] = 2;
        refresh_digest(&mut malformed);
        assert_eq!(
            RecoveryReportEnvelope::decode(&malformed),
            Err(RecoveryReportDecodeDenial::Malformed),
            "literal v1 optional flag at {offset}"
        );
    }
}

fn literal_v1_report() -> (Vec<u8>, [usize; 2]) {
    let mut bytes = Vec::new();
    protocol::encode_header(&mut bytes);
    bytes.push(2);
    let store_flag = bytes.len();
    bytes.push(0);
    let root_flag = bytes.len();
    bytes.push(0);
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.push(6);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&digest);
    (bytes, [store_flag, root_flag])
}

fn refresh_digest(bytes: &mut [u8]) {
    let split = bytes.len() - 32;
    let digest: [u8; 32] = Sha256::digest(&bytes[..split]).into();
    bytes[split..].copy_from_slice(&digest);
}

fn rehashed_prefix(encoded: &[u8], payload_length: usize) -> Vec<u8> {
    let mut truncated = encoded[..payload_length].to_vec();
    let digest: [u8; 32] = Sha256::digest(&truncated).into();
    truncated.extend_from_slice(&digest);
    truncated
}

/// A limit is reported as a limit, never as the damage of a phase; each
/// damaged phase keeps its own cause.
#[test]
fn a_block_cause_is_reported_as_a_limit_or_as_its_own_damage() {
    use super::model::block_cause;
    use crate::entry::{PhysicalRecoveryBlockCause as Cause, PhysicalRecoveryBlockKind as Kind};
    use crate::orchestration::recovery_limit_for_test;
    use RecoveryReportBlockCause as Report;
    let limit = recovery_limit_for_test(crate::PhysicalRecoveryLimitDimension::WalBytes, 9, 8);
    let limit = Cause::Limit {
        phase: Kind::WalInventory,
        limit: limit.into(),
    };
    assert_eq!(block_cause(limit), Report::Limit);
    for (kind, report) in [
        (Kind::MediaObservation, Report::MediaObservation),
        (Kind::SourceAllocation, Report::SourceAllocation),
        (Kind::RootProtocol, Report::RootProtocol),
        (Kind::Checkpoint, Report::Checkpoint),
        (Kind::WalInventory, Report::WalInventory),
        (Kind::SourceSelection, Report::SourceSelection),
        (Kind::BindingFreshness, Report::BindingFreshness),
        (Kind::PageAdmission, Report::PageAdmission),
        (
            Kind::OperationReconciliation,
            Report::OperationReconciliation,
        ),
        (Kind::RedoPlanning, Report::RedoPlanning),
        (Kind::SelectedCustody, Report::SelectedCustody),
        (Kind::Staging, Report::Staging),
        (Kind::Publication, Report::Publication),
    ] {
        assert_eq!(block_cause(Cause::Damage(kind)), report, "{kind:?}");
    }
}
