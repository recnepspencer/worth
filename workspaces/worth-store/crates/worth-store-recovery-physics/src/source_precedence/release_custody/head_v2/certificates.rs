//! The selected checkpoint carries exactly the base's Batch records and its
//! V2 accumulator, in order. Store rejoin and the Serving seal share this law.

use worth_store_physical_format::{
    decode_checkpoint_certificate, CheckpointCertificateKind, ReleaseCheckpointCertificateV1,
};

use super::{VerifiedCheckpointReleaseHeadRosterV2, VerifiedSelectedReleaseHeadCustodyV2};

impl VerifiedCheckpointReleaseHeadRosterV2 {
    /// Whether a checkpoint's certificate records carry exactly this roster's
    /// Batch certificates followed by its V2 accumulator.
    pub fn certificates_match<'a>(&self, records: impl IntoIterator<Item = &'a [u8]>) -> bool {
        let mut expected = self
            .batches()
            .iter()
            .copied()
            .map(ReleaseCheckpointCertificateV1::Batch)
            .chain(std::iter::once(
                ReleaseCheckpointCertificateV1::AccumulatorV2(self.accumulator_v2()),
            ));
        let mut observed = 0_usize;
        for frame in records {
            let Ok((kind, payload)) = decode_checkpoint_certificate(frame) else {
                return false;
            };
            if kind != CheckpointCertificateKind::ReleasedDrop {
                continue;
            }
            match ReleaseCheckpointCertificateV1::decode(payload) {
                Ok(value) if expected.next() == Some(value) => observed += 1,
                _ => return false,
            }
        }
        expected.next().is_none()
            && observed == usize::from(self.release_certificate_record_count())
    }
}

impl VerifiedSelectedReleaseHeadCustodyV2 {
    /// See [`VerifiedCheckpointReleaseHeadRosterV2::certificates_match`].
    pub fn certificates_match<'a>(&self, records: impl IntoIterator<Item = &'a [u8]>) -> bool {
        self.roster.certificates_match(records)
    }
}
