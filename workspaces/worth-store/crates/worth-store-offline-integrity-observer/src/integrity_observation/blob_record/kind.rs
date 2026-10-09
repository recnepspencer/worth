//! What this reader holds for an inner-frame kind before it reads a payload: the
//! family the frame is reported under, its frame ceiling, and whether it carries
//! an occurrence claim. The set of codes is the format's declaration, so a code
//! the format adds is not a frame here until it has a row.

use worth_foundational::PhysicalArtifactFamily as Family;
use worth_store_physical_format::integrity_declarations::families::blob::blob_record_kind_is_declared;

use super::super::OfflineArtifactFamily;
use super::fact::BlobFact;

const CHUNK_MAX: usize = 1 << 20;
const NODE_MAX: usize = 512 << 10;
const CONTROL_MAX: usize = 64 << 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrameKind {
    pub(super) code: u8,
    pub(crate) family: OfflineArtifactFamily,
    pub(super) maximum_bytes: usize,
    pub(super) occurrence_claim: bool,
}

impl FrameKind {
    /// `None` for a code the format does not declare.
    pub(crate) fn declared(code: u8) -> Option<Self> {
        if !blob_record_kind_is_declared(code) {
            return None;
        }
        let session = Family::BlobResumeSession.into();
        let (family, maximum_bytes, occurrence_claim) = match code {
            1 | 5 | 6 => (session, CONTROL_MAX, false),
            2 => (Family::BlobChunkFrame.into(), CHUNK_MAX, true),
            3 => (Family::BlobTreeNode.into(), NODE_MAX, true),
            4 => (Family::BlobGenerationPublication.into(), CONTROL_MAX, false),
            7 | 9 | 13 => (Family::BlobDropSetManifest.into(), CONTROL_MAX, false),
            8 | 14 | 16 => (Family::BlobReclaimDescriptor.into(), CONTROL_MAX, false),
            10 => (
                OfflineArtifactFamily::OriginalDropReservation,
                CONTROL_MAX,
                false,
            ),
            11 | 15 => (session, CONTROL_MAX, true),
            12 => (OfflineArtifactFamily::DedupeQuarantine, CONTROL_MAX, true),
            _ => unreachable!("every declared kind has a row"),
        };
        Some(Self {
            code,
            family,
            maximum_bytes,
            occurrence_claim,
        })
    }

    /// The kind a read fact came from.
    pub(crate) fn of(fact: &BlobFact) -> Self {
        Self::declared(fact.kind_code()).expect("a fact is read from a declared kind")
    }

    /// Whether a frame of this kind is a row that a check seeks as `role`.
    pub(crate) fn may_be(self, role: FrameRole) -> bool {
        match role {
            FrameRole::Declaration => self.code == 1,
            FrameRole::ChunkClaim => matches!(self.code, 2 | 11 | 15),
            FrameRole::DropReservation => self.code == 10,
            FrameRole::ReleaseManifest => self.code == 13,
        }
    }
}

/// A row that a check finds by what its frame says, not by its record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameRole {
    /// The declaration of a session.
    Declaration,
    /// A session's chunk at one ordinal: its chunk frame or its reuse claim.
    ChunkClaim,
    /// The reservation that names a drop-set manifest.
    DropReservation,
    /// The manifest of a release that names a publication.
    ReleaseManifest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_kind_code_has_its_row_and_no_other_code_has_one() {
        let session = OfflineArtifactFamily::Declared(Family::BlobResumeSession);
        let manifest = OfflineArtifactFamily::Declared(Family::BlobDropSetManifest);
        let descriptor = OfflineArtifactFamily::Declared(Family::BlobReclaimDescriptor);
        let control = |family| (family, 64 << 10, false);
        let claimed = |family| (family, 64 << 10, true);
        let rows = [
            (1, control(session)),
            (
                2,
                (
                    OfflineArtifactFamily::Declared(Family::BlobChunkFrame),
                    1 << 20,
                    true,
                ),
            ),
            (
                3,
                (
                    OfflineArtifactFamily::Declared(Family::BlobTreeNode),
                    512 << 10,
                    true,
                ),
            ),
            (
                4,
                control(OfflineArtifactFamily::Declared(
                    Family::BlobGenerationPublication,
                )),
            ),
            (5, control(session)),
            (6, control(session)),
            (7, control(manifest)),
            (8, control(descriptor)),
            (9, control(manifest)),
            (10, control(OfflineArtifactFamily::OriginalDropReservation)),
            (11, claimed(session)),
            (12, claimed(OfflineArtifactFamily::DedupeQuarantine)),
            (13, control(manifest)),
            (14, control(descriptor)),
            (15, claimed(session)),
            (16, control(descriptor)),
        ];
        for code in 0..=u8::MAX {
            let expected = rows
                .iter()
                .find(|(declared, _)| *declared == code)
                .map(|(_, row)| *row);
            let kind = FrameKind::declared(code);
            assert_eq!(
                kind.map(|kind| (kind.family, kind.maximum_bytes, kind.occurrence_claim)),
                expected,
                "kind {code}"
            );
            assert_eq!(kind.map(|kind| kind.code), expected.map(|_| code));
            let roles = [
                (FrameRole::Declaration, code == 1),
                (FrameRole::ChunkClaim, [2, 11, 15].contains(&code)),
                (FrameRole::DropReservation, code == 10),
                (FrameRole::ReleaseManifest, code == 13),
            ];
            for (role, is) in roles {
                let may_be = kind.is_some_and(|kind| kind.may_be(role));
                assert_eq!(may_be, is, "kind {code} as {role:?}");
            }
            assert_eq!(
                expected.is_some(),
                blob_record_kind_is_declared(code),
                "kind {code}"
            );
        }
    }
}
