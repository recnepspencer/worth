//! The front door: read drafts from source text or encoded bytes.

use super::denial::ExpressionResult;
use super::draft::ExpressionDraft;
use super::profile::ExpressionProfile;

/// The V1 expression language.
///
/// Reading is bounded by the reading profile, engineering by default.
/// Admission then re-checks the draft against the profile it admits under,
/// so the common path names one profile, at admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpressionLanguage {
    profile: ExpressionProfile,
}

/// The V1 expression language, reading under engineering ceilings.
pub fn expressions() -> ExpressionLanguage {
    ExpressionLanguage {
        profile: ExpressionProfile::engineering(),
    }
}

impl ExpressionLanguage {
    /// Reads under `profile` instead, such as an editor's interactive
    /// ceilings for untrusted keystrokes.
    pub fn reading_within(self, profile: ExpressionProfile) -> Self {
        Self { profile }
    }

    /// Parses V1 source text into a draft.
    pub fn parse(&self, source: &str) -> ExpressionResult<ExpressionDraft> {
        ExpressionDraft::parse(source, &self.profile)
    }

    /// Decodes an untrusted encoded draft. Unknown encoding and language
    /// versions deny before any structure is read.
    pub fn decode(&self, bytes: &[u8]) -> ExpressionResult<ExpressionDraft> {
        ExpressionDraft::decode(bytes, &self.profile)
    }
}
