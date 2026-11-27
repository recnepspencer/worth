//! Fact-free "who may act?": ability and capability verdicts read from the
//! installed policy decisions admission uses. Nothing is retained, reserved
//! or committed; a verdict is not authority.
pub use crate::session::{
    WorthQueryEligibilityDenial, WorthQueryEligibilityRoster, WorthQueryEligibilityVerdict,
};
pub use worth_query_execution::facade::eligibility::*;
