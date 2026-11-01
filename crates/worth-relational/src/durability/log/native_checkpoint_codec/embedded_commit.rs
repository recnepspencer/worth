//! Inline artifacts preserve the runtime-affine envelope projection required by
//! application checkpoint reconstruction. Its canonical payload digest remains
//! the journal artifact's digest; index caches are separate members.
use serde::Serialize;

use crate::history::data::{EmbeddedCheckpointEnvelopeRef, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Serialize)]
pub(super) struct NativeCommitRef<'a> {
    position: PatchStreamPosition,
    canonical: EmbeddedCheckpointEnvelopeRef<'a>,
}

impl<'a> NativeCommitRef<'a> {
    pub(super) fn new(commit: &'a PositionedCanonicalCommit) -> Self {
        Self {
            position: commit.position(),
            canonical: EmbeddedCheckpointEnvelopeRef::new(commit.envelope()),
        }
    }
}

#[derive(Serialize)]
pub(super) enum EmbeddedCommitRef<'a> {
    Native(NativeCommitRef<'a>),
    Journal(super::checkpoint_wire::ByteString<'a>),
}
impl<'a> EmbeddedCommitRef<'a> {
    pub(super) fn new(commit: &'a PositionedCanonicalCommit) -> Self {
        Self::Native(NativeCommitRef::new(commit))
    }
}
