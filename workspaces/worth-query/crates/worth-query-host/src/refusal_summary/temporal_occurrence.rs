//! Refused temporal occurrences: a recurring fire or other timed operation
//! that catch-up could not commit.
use crate::temporal_contribution::{
    WorthQueryTemporalOccurrenceRefusal, WorthQueryTemporalOccurrenceRefusalKind as Kind,
};

use super::request::RULE_REFUSED;
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const DUPLICATE: Summary = Summary::with_message(
    Category::Conflict,
    "A scheduled item wasn't created because another item already uses the same identifier.",
);
const MISSING_CONTEXT: Summary = Summary::with_message(
    Category::NotFound,
    "A scheduled item wasn't created because the place it belongs to no longer exists.",
);
const EFFECT_REFUSED: Summary = Summary::with_message(
    Category::Validation,
    "A scheduled item wasn't created because the app refused the change it describes.",
);

impl WorthQueryUserFacingRefusal for WorthQueryTemporalOccurrenceRefusal {
    fn user_summary(&self) -> Summary {
        match self.kind() {
            Kind::UniqueValueTaken => DUPLICATE,
            Kind::ContextUnresolved => MISSING_CONTEXT,
            Kind::EffectRefused => EFFECT_REFUSED,
            Kind::CommitRefused => RULE_REFUSED,
        }
    }
}
