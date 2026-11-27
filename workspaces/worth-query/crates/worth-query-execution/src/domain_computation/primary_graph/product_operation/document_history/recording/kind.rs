//! The three kinds of recorded step and the rules that differ between them.
use super::{refusal::CandidateDisposition, JournalNamespace};

/// An edit and a gesture are writers keyed in the writer namespace; an
/// envelope (with or without a correction) is keyed in its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum RecordingKind {
    Edit,
    Envelope,
    Group,
}

impl RecordingKind {
    pub(in super::super) const fn namespace(self) -> JournalNamespace {
        match self {
            Self::Edit | Self::Group => JournalNamespace::Writer,
            Self::Envelope => JournalNamespace::Envelope,
        }
    }

    /// An edit refuses a moved head before it looks for an earlier attempt.
    /// An envelope and a gesture replay an attempt they already recorded even
    /// after the head has moved (ruling 2).
    pub(in super::super) const fn head_before_lookup(self) -> bool {
        matches!(self, Self::Edit)
    }

    /// An envelope parks a candidate that did not close and releases its
    /// record. An edit and a gesture keep both for the caller's retry.
    pub(in super::super) const fn on_refusal(self) -> CandidateDisposition {
        match self {
            Self::Edit | Self::Group => CandidateDisposition::KeepForRetry,
            Self::Envelope => CandidateDisposition::Park,
        }
    }
}
