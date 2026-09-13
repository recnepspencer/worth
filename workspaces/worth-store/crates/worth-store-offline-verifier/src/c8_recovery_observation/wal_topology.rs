use super::artifact_walk::ObservedRecoveryArtifact;
use super::observer_evidence_accumulation::RecoveryObserverWalTopologyObservation;
use super::RecoveryObserverWalTopologyDenial;

pub(super) fn validate(
    artifacts: &[ObservedRecoveryArtifact],
) -> Result<(), RecoveryObserverWalTopologyDenial> {
    let mut observations = Vec::new();
    for artifact in artifacts {
        let Some(observation) = artifact.evidence().wal_topology else {
            continue;
        };
        if let Some(denial) = observation.denial {
            return Err(denial);
        }
        observations.push(observation);
    }
    observations.sort_by_key(|observation| (observation.segment, observation.generation));
    validate_order(&observations)
}

fn validate_order(
    observations: &[RecoveryObserverWalTopologyObservation],
) -> Result<(), RecoveryObserverWalTopologyDenial> {
    let Some(first) = observations.first() else {
        return Ok(());
    };
    for pair in observations.windows(2) {
        let previous = pair[0];
        let current = pair[1];
        if current.segment == previous.segment {
            return Err(RecoveryObserverWalTopologyDenial::DuplicateSegment);
        }
        if current.generation != first.generation {
            return Err(RecoveryObserverWalTopologyDenial::GenerationMismatch);
        }
        if current.segment != previous.segment.saturating_add(1) {
            return Err(RecoveryObserverWalTopologyDenial::NonContiguousSegment);
        }
        if current.first_lsn > previous.last_lsn {
            return Err(RecoveryObserverWalTopologyDenial::LsnGap);
        }
        if current.first_lsn < previous.last_lsn {
            return Err(RecoveryObserverWalTopologyDenial::LsnOverlap);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::super::{
        observe_recovery_artifacts, RecoveryObserverLimits, RecoveryObserverObservationDenial,
        RecoveryObserverWalTopologyDenial,
    };

    #[test]
    fn a_gap_between_valid_wal_files_is_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "first.wal"),
            frame(1, 1, 10, 20, b"first"),
        )
        .expect("first WAL");
        std::fs::write(
            wal_path(root.path(), "second.wal"),
            frame(2, 1, 21, 30, b"second"),
        )
        .expect("second WAL");

        assert_denial(root.path(), RecoveryObserverWalTopologyDenial::LsnGap);
    }

    #[test]
    fn an_overlap_between_valid_wal_files_is_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "first.wal"),
            frame(1, 1, 10, 20, b"first"),
        )
        .expect("first WAL");
        std::fs::write(
            wal_path(root.path(), "second.wal"),
            frame(2, 1, 19, 30, b"second"),
        )
        .expect("second WAL");

        assert_denial(root.path(), RecoveryObserverWalTopologyDenial::LsnOverlap);
    }

    #[test]
    fn a_noncontiguous_segment_between_valid_wal_files_is_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "first.wal"),
            frame(1, 1, 10, 20, b"first"),
        )
        .expect("first WAL");
        std::fs::write(
            wal_path(root.path(), "third.wal"),
            frame(3, 1, 20, 30, b"third"),
        )
        .expect("third WAL");

        assert_denial(
            root.path(),
            RecoveryObserverWalTopologyDenial::NonContiguousSegment,
        );
    }

    #[test]
    fn duplicate_segments_between_valid_wal_files_are_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "first.wal"),
            frame(1, 1, 10, 20, b"first"),
        )
        .expect("first WAL");
        std::fs::write(
            wal_path(root.path(), "duplicate.wal"),
            frame(1, 1, 20, 30, b"duplicate"),
        )
        .expect("duplicate WAL");

        assert_denial(
            root.path(),
            RecoveryObserverWalTopologyDenial::DuplicateSegment,
        );
    }

    #[test]
    fn generation_mismatch_inside_one_wal_file_is_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "generation.wal"),
            joined([
                frame(1, 1, 10, 20, b"first"),
                frame(1, 2, 20, 30, b"second"),
            ]),
        )
        .expect("generation WAL");

        assert_denial(
            root.path(),
            RecoveryObserverWalTopologyDenial::GenerationMismatch,
        );
    }

    #[test]
    fn segment_identity_mismatch_inside_one_wal_file_is_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "segment.wal"),
            joined([
                frame(1, 1, 10, 20, b"first"),
                frame(2, 1, 20, 30, b"second"),
            ]),
        )
        .expect("segment WAL");

        assert_denial(
            root.path(),
            RecoveryObserverWalTopologyDenial::SegmentIdentityMismatch,
        );
    }

    #[test]
    fn noncontiguous_lsns_inside_one_wal_file_are_denied() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            wal_path(root.path(), "lsn.wal"),
            joined([
                frame(1, 1, 10, 20, b"first"),
                frame(1, 1, 22, 30, b"second"),
            ]),
        )
        .expect("LSN WAL");

        assert_denial(
            root.path(),
            RecoveryObserverWalTopologyDenial::NonContiguousLsn,
        );
    }

    #[test]
    fn malformed_wal_header_is_denied_as_a_topology_failure() {
        let root = tempfile::tempdir().expect("WAL topology root");
        let mut malformed = frame(1, 1, 10, 20, b"malformed");
        malformed[8..10].copy_from_slice(&2_u16.to_le_bytes());
        std::fs::write(wal_path(root.path(), "malformed.wal"), malformed).expect("malformed WAL");

        assert_denial(
            root.path(),
            RecoveryObserverWalTopologyDenial::MalformedFrame,
        );
    }

    #[test]
    fn plausible_wal_outside_the_selected_family_is_retained_as_residue() {
        let root = tempfile::tempdir().expect("WAL topology root");
        std::fs::write(
            root.path().join("unselected.wal"),
            frame(7, 9, 10, 20, b"residue"),
        )
        .expect("unselected WAL");
        let limits = RecoveryObserverLimits::new(16, 16, 16, 16 * 1024).expect("limits");
        let report = observe_recovery_artifacts(root.path(), limits)
            .expect("unselected WAL must not become selected topology");
        assert_eq!(report.wal_segment_count(), 0);
        assert_eq!(report.residue_artifact_count(), 1);
        assert_eq!(report.residue_bytes(), report.bytes_read());
    }

    fn assert_denial(root: &std::path::Path, expected: RecoveryObserverWalTopologyDenial) {
        let limits = RecoveryObserverLimits::new(16, 16, 16, 16 * 1024).expect("limits");
        let failure = observe_recovery_artifacts(root, limits).expect_err("invalid WAL topology");
        assert_eq!(
            failure.denial(),
            RecoveryObserverObservationDenial::WalTopology(expected)
        );
    }

    fn wal_path(root: &std::path::Path, name: &str) -> std::path::PathBuf {
        let directory = root.join("families/wal");
        std::fs::create_dir_all(&directory).expect("WAL directory");
        directory.join(name)
    }

    fn frame(segment: u64, generation: u64, start: u64, end: u64, payload: &[u8]) -> Vec<u8> {
        let mut header = vec![0; 116];
        header[..8].copy_from_slice(b"WORTHWAL");
        header[8..10].copy_from_slice(&1_u16.to_le_bytes());
        header[10..12].copy_from_slice(&116_u16.to_le_bytes());
        header[12..20].copy_from_slice(&segment.to_le_bytes());
        header[20..28].copy_from_slice(&generation.to_le_bytes());
        header[28..36].copy_from_slice(&start.to_le_bytes());
        header[36..44].copy_from_slice(&end.to_le_bytes());
        header[44..52].copy_from_slice(&(payload.len() as u64).to_le_bytes());
        let payload_digest: [u8; 32] = Sha256::digest(payload).into();
        header[84..116].copy_from_slice(&payload_digest);
        let mut frame = header;
        frame.extend_from_slice(payload);
        let frame_digest: [u8; 32] = Sha256::digest(&frame).into();
        frame.extend_from_slice(&frame_digest);
        frame
    }

    fn joined<const N: usize>(frames: [Vec<u8>; N]) -> Vec<u8> {
        frames.into_iter().flatten().collect()
    }
}
