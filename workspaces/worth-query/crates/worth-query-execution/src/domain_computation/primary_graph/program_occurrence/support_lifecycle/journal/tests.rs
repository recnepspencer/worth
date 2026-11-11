use super::*;
use std::num::NonZeroU64;
use worth_foundational::facade::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
fn original(home: HomeIdentity, ordinal: u64) -> DurableCompositePublicationAttemptIdentity {
    DurableCompositePublicationAttemptIdentity::from_recorded(
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(0)),
        ordinal,
    )
}
fn home(byte: u8) -> HomeIdentity {
    HomeIdentity::from_observed_bytes([byte; 16]).unwrap()
}
fn revision(byte: u8) -> CanonicalDigestId {
    CanonicalDigestId::new([byte; 32])
}
fn prepare(
    ledger: &QueryProgramSupportLedger,
    record: QueryProgramSupportRecord,
) -> Result<QueryProgramSupportFrame, QueryProgramSupportDenial> {
    let mut generations = QueryProgramSupportPublicationGenerations::from_journal(ledger);
    ledger.prepare(record, &mut generations)
}
fn append(ledger: &mut QueryProgramSupportLedger, record: QueryProgramSupportRecord) -> Vec<u8> {
    let frame = prepare(ledger, record).unwrap();
    let bytes = frame.bytes().to_vec();
    ledger
        .readmit(QueryProgramSupportFrame::readmit(&bytes).unwrap())
        .unwrap();
    bytes
}
#[test]
fn acknowledged_obligation_membership_derives_counts_and_requires_explicit_release() {
    let mut ledger = QueryProgramSupportLedger::empty(home(1));
    let source = revision(1);
    let target = revision(2);
    let mut prefix = vec![append(
        &mut ledger,
        QueryProgramSupportRecord::Install { revision: source },
    )];
    prefix.push(append(
        &mut ledger,
        QueryProgramSupportRecord::Install { revision: target },
    ));
    let identity =
        ProgramSupportObligationIdentity::from_recorded(home(1), NonZeroU64::new(3).unwrap());
    prefix.push(append(
        &mut ledger,
        QueryProgramSupportRecord::Acquire(QueryProgramSupportObligation {
            identity,
            kind: ProgramSupportObligationKind::BranchAdoption,
            source,
            target,
            original: original(home(1), 7),
        }),
    ));
    assert_eq!(ledger.counts(&source), (0, 1));
    assert_eq!(ledger.counts(&target), (0, 1));
    let mut cold = QueryProgramSupportLedger::empty(home(1));
    for bytes in prefix {
        cold.readmit(QueryProgramSupportFrame::readmit(&bytes).unwrap())
            .unwrap();
    }
    assert_eq!(cold.counts(&source), (0, 1));
    assert_eq!(cold.counts(&target), (0, 1));
    let duplicate = QueryProgramSupportRecord::Acquire(QueryProgramSupportObligation {
        identity: ProgramSupportObligationIdentity::from_recorded(
            home(1),
            NonZeroU64::new(4).unwrap(),
        ),
        kind: ProgramSupportObligationKind::BranchAdoption,
        source,
        target,
        original: original(home(1), 7),
    });
    assert_eq!(
        prepare(&ledger, duplicate),
        Err(QueryProgramSupportDenial::DuplicateObligation)
    );
    append(&mut ledger, QueryProgramSupportRecord::Release { identity });
    assert_eq!(ledger.counts(&source), (0, 0));
    assert_eq!(ledger.counts(&target), (0, 0));
    assert_eq!(
        prepare(&ledger, QueryProgramSupportRecord::Release { identity }),
        Err(QueryProgramSupportDenial::UnknownObligation)
    );
}
#[test]
fn same_program_obligations_count_once_and_retirement_requires_no_durable_users() {
    let mut ledger = QueryProgramSupportLedger::empty(home(1));
    let source = revision(1);
    append(
        &mut ledger,
        QueryProgramSupportRecord::Install { revision: source },
    );
    let identity =
        ProgramSupportObligationIdentity::from_recorded(home(1), NonZeroU64::new(2).unwrap());
    append(
        &mut ledger,
        QueryProgramSupportRecord::Acquire(QueryProgramSupportObligation {
            identity,
            kind: ProgramSupportObligationKind::OriginalInterpretation,
            source,
            target: source,
            original: original(home(1), 8),
        }),
    );
    assert_eq!(ledger.counts(&source), (1, 0));
    append(
        &mut ledger,
        QueryProgramSupportRecord::Status {
            revision: source,
            before: ProgramSupportStatus::Active,
            after: ProgramSupportStatus::Retiring,
        },
    );
    assert_eq!(
        prepare(
            &ledger,
            QueryProgramSupportRecord::Status {
                revision: source,
                before: ProgramSupportStatus::Retiring,
                after: ProgramSupportStatus::Retired
            }
        ),
        Err(QueryProgramSupportDenial::SupportUnavailable)
    );
    append(&mut ledger, QueryProgramSupportRecord::Release { identity });
    append(
        &mut ledger,
        QueryProgramSupportRecord::Status {
            revision: source,
            before: ProgramSupportStatus::Retiring,
            after: ProgramSupportStatus::Retired,
        },
    );
    assert_eq!(ledger.status(&source), Some(ProgramSupportStatus::Retired));
    assert_eq!(
        prepare(
            &ledger,
            QueryProgramSupportRecord::Status {
                revision: source,
                before: ProgramSupportStatus::Retired,
                after: ProgramSupportStatus::Active
            }
        ),
        Err(QueryProgramSupportDenial::IllegalStatusTransition)
    );
}
#[test]
fn cold_readmission_preserves_retiring_and_retired_without_reinstalling_active() {
    let mut ledger = QueryProgramSupportLedger::empty(home(1));
    let source = revision(1);
    let mut prefix = vec![append(
        &mut ledger,
        QueryProgramSupportRecord::Install { revision: source },
    )];
    prefix.push(append(
        &mut ledger,
        QueryProgramSupportRecord::Status {
            revision: source,
            before: ProgramSupportStatus::Active,
            after: ProgramSupportStatus::Retiring,
        },
    ));
    let mut cold = QueryProgramSupportLedger::empty(home(1));
    for bytes in &prefix {
        cold.readmit(QueryProgramSupportFrame::readmit(bytes).unwrap())
            .unwrap();
    }
    assert_eq!(cold.status(&source), Some(ProgramSupportStatus::Retiring));
    let bytes = append(
        &mut ledger,
        QueryProgramSupportRecord::Status {
            revision: source,
            before: ProgramSupportStatus::Retiring,
            after: ProgramSupportStatus::Retired,
        },
    );
    cold.readmit(QueryProgramSupportFrame::readmit(&bytes).unwrap())
        .unwrap();
    assert_eq!(cold.status(&source), Some(ProgramSupportStatus::Retired));
    assert_eq!(cold.digest(), ledger.digest());
    assert_eq!(
        prepare(
            &cold,
            QueryProgramSupportRecord::Install { revision: source }
        ),
        Err(QueryProgramSupportDenial::DuplicateProgram)
    );
}
#[test]
fn support_frames_refuse_damage_foreign_home_and_wrong_prefix_with_typed_causes() {
    let ledger = QueryProgramSupportLedger::empty(home(1));
    let frame = prepare(
        &ledger,
        QueryProgramSupportRecord::Install {
            revision: revision(1),
        },
    )
    .unwrap();
    let mut bytes = frame.bytes().to_vec();
    bytes[50] ^= 1;
    assert_eq!(
        QueryProgramSupportFrame::readmit(&bytes),
        Err(QueryProgramSupportDenial::Checksum)
    );
    let mut foreign = QueryProgramSupportLedger::empty(home(2));
    assert_eq!(
        foreign.readmit(frame.clone()),
        Err(QueryProgramSupportDenial::ForeignHome)
    );
    let mut cold = QueryProgramSupportLedger::empty(home(1));
    cold.readmit(frame.clone()).unwrap();
    assert_eq!(
        cold.readmit(frame),
        Err(QueryProgramSupportDenial::PrefixMismatch)
    );
}
#[test]
fn a_checksummed_unknown_version_or_trailing_field_never_enters_the_ledger() {
    use sha2::{Digest, Sha256};
    let ledger = QueryProgramSupportLedger::empty(home(1));
    let frame = prepare(
        &ledger,
        QueryProgramSupportRecord::Install {
            revision: revision(1),
        },
    )
    .unwrap();
    let mut bytes = frame.bytes().to_vec();
    bytes.truncate(bytes.len() - 32);
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&hash);
    assert_eq!(
        QueryProgramSupportFrame::readmit(&bytes),
        Err(QueryProgramSupportDenial::UnsupportedVersion(2))
    );
    let mut bytes = frame.bytes().to_vec();
    bytes.truncate(bytes.len() - 32);
    bytes.push(9);
    let hash: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend_from_slice(&hash);
    assert_eq!(
        QueryProgramSupportFrame::readmit(&bytes),
        Err(QueryProgramSupportDenial::MalformedFrame)
    );
}

#[test]
fn prepared_support_records_name_only_the_affected_revisions() {
    let mut ledger = QueryProgramSupportLedger::empty(home(1));
    let a = revision(1);
    let b = revision(2);
    let unrelated = revision(3);
    for revision in [a, b] {
        append(&mut ledger, QueryProgramSupportRecord::Install { revision });
    }
    let retirement = prepare(
        &ledger,
        QueryProgramSupportRecord::Status {
            revision: a,
            before: ProgramSupportStatus::Active,
            after: ProgramSupportStatus::Retiring,
        },
    )
    .unwrap();
    assert!(ledger.record_mentions_revision(retirement.record(), &a));
    assert!(!ledger.record_mentions_revision(retirement.record(), &b));
    let identity =
        ProgramSupportObligationIdentity::from_recorded(home(1), NonZeroU64::new(3).unwrap());
    let acquire = QueryProgramSupportRecord::Acquire(QueryProgramSupportObligation {
        identity,
        kind: ProgramSupportObligationKind::BranchAdoption,
        source: a,
        target: b,
        original: original(home(1), 0),
    });
    for revision in [a, b] {
        assert!(ledger.record_mentions_revision(&acquire, &revision));
    }
    assert!(!ledger.record_mentions_revision(&acquire, &unrelated));
    append(&mut ledger, acquire);
    let release = QueryProgramSupportRecord::Release { identity };
    for revision in [a, b] {
        assert!(ledger.record_mentions_revision(&release, &revision));
    }
    assert!(!ledger.record_mentions_revision(&release, &unrelated));
}
