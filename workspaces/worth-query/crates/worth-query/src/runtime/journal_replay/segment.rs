use crate::evidence_identity::WorthQueryEvidenceIdentityScheme;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryJournalPosition, WorthQueryJournalPositionAuthority};

use super::{WorthQueryJournalReplayDenial, WorthQueryJournalReplayDenialKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalSegmentIdentity {
    start_position: u64,
    end_position: u64,
    segment_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryJournalSegmentIdentity {
    pub fn between(
        start: &WorthQueryJournalPosition,
        end: &WorthQueryJournalPosition,
    ) -> Result<Self, WorthQueryJournalReplayDenial> {
        if start.authority() != WorthQueryJournalPositionAuthority::Committed
            || end.authority() != WorthQueryJournalPositionAuthority::Committed
        {
            return Err(WorthQueryJournalReplayDenial::new(
                WorthQueryJournalReplayDenialKind::InvalidSegmentBounds,
                "journal segment bounds must be committed journal positions",
            ));
        }
        let start_position = start.ordinal_for_reporting();
        let end_position = end.ordinal_for_reporting();
        if start_position > end_position {
            return Err(WorthQueryJournalReplayDenial::new(
                WorthQueryJournalReplayDenialKind::InvalidSegmentBounds,
                "journal segment start must not exceed end",
            ));
        }
        Ok(Self::from_committed_bounds(start_position, end_position))
    }

    pub(in crate::runtime) fn from_committed_bounds(
        start_position: u64,
        end_position: u64,
    ) -> Self {
        let segment_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::JournalSegmentIdentity)
                .field_value(
                    WorthQueryEvidenceTag::new("journal_segment_start"),
                    start_position.to_string(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("journal_segment_end"),
                    end_position.to_string(),
                )
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            start_position,
            end_position,
            segment_identity,
        }
    }

    pub fn start_position_for_reporting(&self) -> u64 {
        self.start_position
    }

    pub fn end_position_for_reporting(&self) -> u64 {
        self.end_position
    }

    pub fn identity_digest(&self) -> &str {
        self.segment_identity.as_str()
    }

    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.segment_identity
    }

    pub fn admit_versioned_committed_bounds_for_replay(
        start_position: u64,
        end_position: u64,
        scheme: WorthQueryEvidenceIdentityScheme,
    ) -> Result<Self, WorthQueryJournalReplayDenial> {
        if start_position > end_position {
            return Err(WorthQueryJournalReplayDenial::new(
                WorthQueryJournalReplayDenialKind::InvalidSegmentBounds,
                "journal segment start must not exceed end",
            ));
        }
        let segment_identity = WorthQueryEvidenceIdentity::compose_with_scheme(
            WorthQueryEvidenceScope::JournalSegmentIdentity,
            scheme,
        )
        .field_value(
            WorthQueryEvidenceTag::new("journal_segment_start"),
            start_position.to_string(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("journal_segment_end"),
            end_position.to_string(),
        )
        .seal();
        Ok(Self {
            start_position,
            end_position,
            segment_identity,
        })
    }
}
