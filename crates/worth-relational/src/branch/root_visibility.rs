use crate::history::data::CanonicalCommitEnvelope;

use super::RelationalRootCorrectnessIndex;

/// Typed commitment to the complete state tuple selected by a branch root.
///
/// This is descriptive integrity, never branch currentness or mutation
/// authority. It binds the truth, schema, correctness-index posture, and
/// canonical commit that readers must observe as one unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RelationalBranchVisibilityCommitment([u8; 32]);

impl RelationalBranchVisibilityCommitment {
    pub(super) fn for_root(
        envelope: &CanonicalCommitEnvelope,
        storage_root: [u8; 32],
        schema_root: [u8; 32],
        correctness_index: RelationalRootCorrectnessIndex,
    ) -> Self {
        use sha2::{Digest, Sha256};

        let mut digest = Sha256::new();
        digest.update(b"worth.relational.branch-visibility.v1\0");
        digest.update(storage_root);
        digest.update(schema_root);
        digest.update(match correctness_index {
            RelationalRootCorrectnessIndex::AuthoritativeFallback => [0],
        });
        digest.update(
            envelope
                .encode_authoritative_payload()
                .expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "canonical branch envelope must encode for visibility commitment"),
        );
        Self(digest.finalize().into())
    }

    pub(crate) const fn digest(self) -> [u8; 32] {
        self.0
    }
}
