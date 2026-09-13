#[path = "wal_observation/wal_evidence_finalization.rs"]
mod wal_evidence_finalization;
#[path = "wal_observation/wal_frame_decode.rs"]
mod wal_frame_decode;
#[path = "wal_observation/wal_prefix_progression.rs"]
mod wal_prefix_progression;

use super::observer_evidence_accumulation::RecoveryObserverArtifactEvidence;

pub(super) fn observe(bytes: &[u8]) -> RecoveryObserverArtifactEvidence {
    let mut prefix = wal_prefix_progression::WalPrefixProgression::new();
    loop {
        match wal_frame_decode::decode(
            bytes,
            prefix.offset(),
            prefix.expected_segment(),
            prefix.expected_generation(),
            prefix.previous_lsn_end(),
        ) {
            wal_frame_decode::FrameDecode::Valid(frame) => prefix.record(frame),
            wal_frame_decode::FrameDecode::Stop(topology) => {
                prefix.stop(topology);
                break;
            }
        }
    }
    wal_evidence_finalization::finish(bytes, prefix)
}
