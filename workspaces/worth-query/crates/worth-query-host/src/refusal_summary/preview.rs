//! "What-if" preview fork, adoption and discard refusals.
use crate::application_home::{
    WorthQueryPreviewAdoptDenial, WorthQueryPreviewDiscardDenial, WorthQueryPreviewForkDenial,
};

use super::request::BUSY;
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const PREVIEW_UNSETTLED: Summary = Summary::with_message(
    Category::Unavailable,
    "The preview couldn't be created yet. Try again in a moment.",
);

impl WorthQueryUserFacingRefusal for WorthQueryPreviewForkDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Advancement(cause) | Self::ContinuationAdvancement { denial: cause, .. } => {
                cause.user_summary()
            }
            Self::Selection(_) => BUSY,
            Self::Create(_)
            | Self::Recovery { .. }
            | Self::NoEffect(_)
            | Self::ReleasedOriginal(_) => PREVIEW_UNSETTLED,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryPreviewAdoptDenial {
    fn user_summary(&self) -> Summary {
        if self.is_stale_head() {
            return Summary::with_message(
                Category::Conflict,
                "Your data changed after this preview was made. Make a new preview to apply it.",
            );
        }
        match self {
            Self::Advancement(cause) => cause.user_summary(),
            Self::Adoption(_) => BUSY,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryPreviewDiscardDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ActiveBranchProtected => Summary::with_message(
                Category::Validation,
                "You can't discard the version you're currently using.",
            ),
            Self::Advancement(cause) => cause.user_summary(),
            Self::Close(_) => BUSY,
        }
    }
}
