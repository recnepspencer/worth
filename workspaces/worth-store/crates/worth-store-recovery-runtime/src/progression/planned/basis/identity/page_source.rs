//! Causal identity of the page source each redo decision observed. Every
//! source kind hashes a distinct tag, its frame coordinate and its identity.

use sha2::{Digest, Sha256};
use worth_store_recovery_physics::RecoveryPageSource;

pub(super) fn hash_page_source(digest: &mut Sha256, source: RecoveryPageSource) {
    let (tag, coordinate, identity) = match source {
        RecoveryPageSource::Materialized {
            coordinate,
            routing_identity,
        } => (1, coordinate, routing_identity),
        RecoveryPageSource::AbsentTarget {
            coordinate,
            root_membership_identity,
        } => (2, coordinate, root_membership_identity),
        RecoveryPageSource::PlannedResult {
            coordinate,
            causal_identity,
        } => (3, coordinate, causal_identity),
        RecoveryPageSource::HistoricalReleasedDrop {
            coordinate,
            selected_root_identity,
            descriptor_operation,
            old_operation,
            wal_target_digest,
        } => {
            digest.update(descriptor_operation);
            digest.update(old_operation);
            digest.update(wal_target_digest);
            (4, coordinate, selected_root_identity)
        }
        RecoveryPageSource::HistoricalRetiredTarget {
            coordinate,
            selected_root_identity,
            retiring_operation,
            old_operation,
            wal_target_digest,
        } => {
            digest.update(retiring_operation);
            digest.update(old_operation);
            digest.update(wal_target_digest);
            (5, coordinate, selected_root_identity)
        }
    };
    digest.update([tag]);
    digest.update(coordinate.artifact().canonical_file_name().as_bytes());
    digest.update(coordinate.offset().to_le_bytes());
    digest.update(coordinate.length().to_le_bytes());
    digest.update(identity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

    #[test]
    fn sealed_source_coordinate_and_routing_identity_are_plan_identity_causal() {
        let first = source_digest(1, 0, [1; 32]);
        assert_ne!(first, source_digest(2, 0, [1; 32]));
        assert_ne!(first, source_digest(1, 4096, [1; 32]));
        assert_ne!(first, source_digest(1, 0, [2; 32]));
    }

    fn source_digest(generation: u64, offset: u64, routing: [u8; 32]) -> [u8; 32] {
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 1,
                generation,
            },
            offset,
            4096,
        )
        .unwrap();
        let mut digest = Sha256::new();
        hash_page_source(
            &mut digest,
            RecoveryPageSource::Materialized {
                coordinate,
                routing_identity: routing,
            },
        );
        digest.finalize().into()
    }
}
