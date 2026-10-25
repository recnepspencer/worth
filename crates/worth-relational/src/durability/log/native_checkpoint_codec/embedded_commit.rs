//! Inline artifacts preserve the runtime-affine envelope projection required by
//! application checkpoint reconstruction. Its canonical payload digest remains
//! the journal artifact's digest; index caches are separate members.
use serde::{Deserialize, Serialize};

use crate::history::data::{
    CanonicalEnvelopeWire, EmbeddedCheckpointEnvelopeRef, PositionedCanonicalCommit,
};
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeCommitWire<'a> {
    position: PatchStreamPosition,
    #[serde(borrow)]
    canonical: CanonicalEnvelopeWire<'a>,
}

impl NativeCommitWire<'_> {
    pub(super) fn readmit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, crate::durability::data::RelationalRecoveryStop> {
        let envelope = self
            .canonical
            .readmit(execution)
            .map_err(crate::symbols::data::field_text_recovery_stop)?;
        crate::durability::migration::ReadmittedCanonicalCommit::readmit_current(
            self.position,
            envelope,
        )
        .map(|commit| commit.into_positioned())
        .map_err(|detail| super::manifest_capture::corrupt(detail.to_string()).into())
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

#[derive(Deserialize)]
pub(super) enum EmbeddedCommitWire<'a> {
    #[serde(borrow)]
    Native(NativeCommitWire<'a>),
    #[serde(borrow)]
    Journal(super::checkpoint_wire::ByteString<'a>),
}
impl EmbeddedCommitWire<'_> {
    pub(super) fn readmit(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, crate::durability::data::RelationalRecoveryStop> {
        match self {
            Self::Native(wire) => wire.readmit(execution),
            Self::Journal(bytes) => crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::readmit(bytes.0, execution)
                .map(|admitted| admitted.into_positioned())
                .map_err(super::cold_checkpoint::artifact_stop),
        }
    }
}
