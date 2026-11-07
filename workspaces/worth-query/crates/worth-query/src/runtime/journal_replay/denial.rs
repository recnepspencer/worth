use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryJournalReplayDenialKind {
    InvalidSegmentBounds,
    UnknownSegmentIdentity,
    StaleBasisReplay,
    CrossSchemeReplay,
    JournalGap,
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
}

impl WorthQueryJournalReplayDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidSegmentBounds => "invalid-segment-bounds",
            Self::UnknownSegmentIdentity => "unknown-segment-identity",
            Self::StaleBasisReplay => "stale-basis-replay",
            Self::CrossSchemeReplay => "cross-scheme-replay",
            Self::JournalGap => "journal-gap",
            Self::EvidenceIdentityDerivationDenied(_) => "evidence-identity-derivation-denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalReplayDenial {
    kind: WorthQueryJournalReplayDenialKind,
    message: String,
}

impl WorthQueryJournalReplayDenial {
    pub(in crate::runtime) fn new(
        kind: WorthQueryJournalReplayDenialKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryJournalReplayDenialKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<IdentityDerivationDenial> for WorthQueryJournalReplayDenial {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::new(
            WorthQueryJournalReplayDenialKind::EvidenceIdentityDerivationDenied(cause),
            "journal replay evidence identity refused by canonical digest limits",
        )
    }
}
