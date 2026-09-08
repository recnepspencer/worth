//! Literal coverage twins: present-but-unusable containers are never absence proof.
use crate::{
    phase_4_literal_vectors::{pending_operation, wal_range},
    support::{clean_store, request},
};
use worth_foundational::{PhysicalArtifactFamily as Family, PhysicalByteRange};
use worth_store_offline_integrity_observer::{
    observe_store, OfflineIndeterminatePhysicalReason as Indeterminate,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalDamageCause as Cause,
    OfflineUnknownPhysicalReason as Unknown,
};

#[test]
fn namespace_truncation_and_absence_preserve_declared_seventy_two_byte_scope() {
    let fixture = clean_store("namespace-expected-scope");
    let path = fixture.store.join("namespace/identity");
    let bytes = std::fs::read(&path).unwrap();
    for absent in [false, true] {
        if absent {
            std::fs::remove_file(&path).unwrap();
        } else {
            std::fs::write(&path, &bytes[..71]).unwrap();
        }
        let report = observe_store(&request(&fixture)).unwrap();
        let row = report
            .artifacts()
            .iter()
            .find(|row| row.family() == Family::NamespaceIdentity)
            .unwrap();
        assert_eq!(row.range(), Some(PhysicalByteRange::new(0, 72).unwrap()));
        assert!(matches!(row.outcome(), Outcome::Damaged(d)
            if d.cause() == if absent { Cause::MissingArtifact } else { Cause::Truncation }));
    }
}

#[test]
fn present_rejected_middle_wal_retains_unknown_coverage_and_its_own_failure() {
    for (label, mutate) in [
        ("unsupported", 0),
        ("bad-checksum", 1),
        ("not-readable-file", 2),
        ("byte-bound", 3),
    ] {
        let fixture = clean_store(label);
        let wal = fixture.store.join("families/wal");
        std::fs::create_dir(&wal).unwrap();
        // Lexical acquisition is 1, 10, 2: both independently admitted neighbors
        // exist even when acquiring the last (logical middle) exhausts the bound.
        for (segment, start) in [(1, 3), (2, 4), (10, 5)] {
            std::fs::write(
                wal.join(format!("segment-{segment}-generation-2.wal")),
                wal_range(segment, start, start + 1),
            )
            .unwrap();
        }
        let clean = observe_store(&request(&fixture)).unwrap();
        let path = wal.join("segment-2-generation-2.wal");
        let mut bytes = wal_range(2, 4, 5);
        match mutate {
            0 => bytes[8..10].copy_from_slice(&2_u16.to_le_bytes()),
            1 => *bytes.last_mut().unwrap() ^= 1,
            2 => {
                std::fs::remove_file(&path).unwrap();
                std::fs::create_dir(&path).unwrap();
            }
            3 => bytes.resize(20 * 1024, 0),
            _ => unreachable!(),
        }
        if mutate != 2 {
            std::fs::write(&path, bytes).unwrap();
        }
        let report = observe_store(&request(&fixture)).unwrap();
        let gap = report
            .artifacts()
            .iter()
            .find(|row| row.identity().as_str() == "wal-required-lsn:4:5")
            .unwrap();
        assert!(
            matches!(
                gap.outcome(),
                Outcome::Unknown(_) | Outcome::Indeterminate(_)
            ),
            "{label}: {:?}",
            gap.outcome()
        );
        assert_eq!(
            report.counters().missing_artifacts(),
            clean.counters().missing_artifacts(),
            "rejected presence must not increase missing artifacts: {label}"
        );
        let middle = report
            .artifacts()
            .iter()
            .find(|row| row.relative_path().ends_with("segment-2-generation-2.wal"))
            .unwrap();
        match mutate {
            0 => {
                assert!(matches!(middle.outcome(), Outcome::Unsupported(_)));
                assert_eq!(
                    gap.outcome(),
                    &Outcome::Unknown(Unknown::WalCoverageUnavailable)
                );
            }
            1 => assert!(
                matches!(middle.outcome(), Outcome::Damaged(d) if d.cause() == Cause::ChecksumMismatch)
            ),
            2 => assert!(matches!(
                middle.outcome(),
                Outcome::Indeterminate(_) | Outcome::Unknown(_)
            )),
            3 => {
                assert_eq!(
                    middle.outcome(),
                    &Outcome::Indeterminate(Indeterminate::ByteBoundExceeded)
                );
                assert_eq!(
                    gap.outcome(),
                    &Outcome::Indeterminate(Indeterminate::ByteBoundExceeded)
                );
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn adjacent_present_wal_containers_have_scope_disagreement_not_a_missing_file() {
    let fixture = clean_store("present-adjacent-lsn-gap");
    let wal = fixture.store.join("families/wal");
    std::fs::create_dir(&wal).unwrap();
    for (segment, start) in [(1, 3), (2, 5)] {
        std::fs::write(
            wal.join(format!("segment-{segment}-generation-2.wal")),
            wal_range(segment, start, start + 1),
        )
        .unwrap();
    }
    let report = observe_store(&request(&fixture)).unwrap();
    let gap = report
        .artifacts()
        .iter()
        .find(|row| row.identity().as_str() == "wal-required-lsn:4:5")
        .unwrap();
    assert!(matches!(gap.outcome(), Outcome::Damaged(d) if d.cause() == Cause::ScopeMismatch));
}

#[test]
fn truncated_pending_obligation_preserves_the_declared_expected_scope() {
    let fixture = clean_store("truncated-pending-scope");
    let pending = fixture.store.join("families/physical-work");
    std::fs::create_dir(&pending).unwrap();
    let bytes = pending_operation();
    assert_eq!(bytes.len(), 160);
    std::fs::write(
        pending.join("effect-0000000000000001-0000000000000002-0000000000000003.pending"),
        &bytes[..159],
    )
    .unwrap();
    let report = observe_store(&request(&fixture)).unwrap();
    let row = report
        .artifacts()
        .iter()
        .find(|row| row.family() == Family::PhysicalWorkObligation)
        .unwrap();
    assert_eq!(row.range(), Some(PhysicalByteRange::new(0, 160).unwrap()));
    assert!(matches!(row.outcome(), Outcome::Damaged(d)
        if d.cause() == Cause::Truncation && d.damaged_range() == Some(PhysicalByteRange::new(159, 1).unwrap())));
}
