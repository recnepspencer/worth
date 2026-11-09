//! Descriptive owner evidence is exhaustive and never substitutes for admission.
use super::{WorldOpenComponentIntent, WorldRelationalEvidence};
use crate::durability::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Medium};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_signal::facade::durability::{
    SignalDurableFrameDenial as SignalFrame, SignalRecordedSourcePoint,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldOccurrenceOwnerEvidence {
    RelationalOnly(WorldRelationalEvidence),
    SignalOnly(WorldSignalEvidence),
    RelationalAndSignal {
        relational: WorldRelationalEvidence,
        signal: WorldSignalEvidence,
    },
}
impl WorldOccurrenceOwnerEvidence {
    pub(crate) const fn component_intent(&self) -> WorldOpenComponentIntent {
        match self {
            Self::RelationalOnly(_) => WorldOpenComponentIntent::RelationalOnly,
            Self::SignalOnly(_) => WorldOpenComponentIntent::SignalOnly,
            Self::RelationalAndSignal { .. } => WorldOpenComponentIntent::RelationalAndSignal,
        }
    }
    pub(crate) fn relational(&self) -> Option<&WorldRelationalEvidence> {
        match self {
            Self::RelationalOnly(relational) | Self::RelationalAndSignal { relational, .. } => {
                Some(relational)
            }
            Self::SignalOnly(_) => None,
        }
    }
    pub(crate) fn signal(&self) -> Option<&WorldSignalEvidence> {
        match self {
            Self::SignalOnly(signal) | Self::RelationalAndSignal { signal, .. } => Some(signal),
            Self::RelationalOnly(_) => None,
        }
    }
}

/// Signal's exact owner envelope is retained alongside its decoded description.
/// Only Signal encodes or decodes that envelope; neither value is a live proof.
#[derive(Clone, Debug)]
pub(crate) struct WorldSignalEvidence {
    point: SignalRecordedSourcePoint,
    bytes: ExecutionImmutableBytes,
}
impl PartialEq for WorldSignalEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point
    }
}
impl Eq for WorldSignalEvidence {}
impl WorldSignalEvidence {
    pub(crate) fn from_point(
        point: SignalRecordedSourcePoint,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Medium> {
        let bytes = point.encode_bytes(request).map_err(signal_frame)?;
        if bytes.len() > SignalRecordedSourcePoint::MAXIMUM_ENCODED_BYTES {
            return Err(Frame::SizeLimit.into());
        }
        Ok(Self { point, bytes })
    }
    pub(crate) fn from_bytes(
        bytes: &[u8],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Medium> {
        let point =
            SignalRecordedSourcePoint::readmit_bytes(request, bytes).map_err(signal_frame)?;
        Self::from_point(point, request)
    }
    pub(crate) fn point(&self) -> &SignalRecordedSourcePoint {
        &self.point
    }
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}
fn signal_frame(cause: SignalFrame) -> Medium {
    match cause {
        SignalFrame::Request(cause) => Medium::Request(cause),
        SignalFrame::Backing(cause) => Medium::Backing(cause),
        SignalFrame::UnsupportedVersion(cause) => Medium::Frame(Frame::UnsupportedVersion(cause)),
        SignalFrame::SizeLimit => Medium::Frame(Frame::SizeLimit),
        SignalFrame::ChecksumMismatch => Medium::Frame(Frame::DigestMismatch),
        SignalFrame::ForeignOwner => Medium::Frame(Frame::IdentityMismatch),
        SignalFrame::ForeignBranch => Medium::Frame(Frame::IdentityMismatch),
        SignalFrame::Malformed | SignalFrame::SnapshotRequired | SignalFrame::Image(_) => {
            Medium::Frame(Frame::Malformed)
        }
    }
}
