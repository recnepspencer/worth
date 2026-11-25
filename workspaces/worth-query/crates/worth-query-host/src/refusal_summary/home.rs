//! Home open, close, retirement and checkpoint refusals.
//!
//! The physical Store causes of a refused open are classified where Host
//! composes Store Application, in `application_home::refusal::user_summary`.
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;

use crate::application_home::{
    WorthQueryApplicationHomeCheckpointDenial, WorthQueryApplicationHomeCloseRefused,
    WorthQueryDirectoryOpenDenial, WorthQueryHomeClosePolicyRefused, WorthQueryHomeRetirementStop,
};

use super::request::{CAPACITY, INTERNAL, STORAGE};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const STILL_CLOSING: Summary = Summary::with_message(
    Category::Unavailable,
    "The app is still saving and closing its data. Try closing again in a moment.",
);

impl<Schema: ApplicationSchemaComposition, Program> WorthQueryUserFacingRefusal
    for WorthQueryDirectoryOpenDenial<Schema, Program>
{
    fn user_summary(&self) -> Summary {
        match self {
            Self::SchemaDeclaration(_) | Self::ProgramValidation(_) | Self::Limits(_) => INTERNAL,
            Self::Advancement(cause) => cause.user_summary(),
            Self::Home(refused) => refused.user_summary(),
        }
    }
}

impl<Schema, Program> WorthQueryUserFacingRefusal
    for WorthQueryApplicationHomeCloseRefused<Schema, Program>
{
    fn user_summary(&self) -> Summary {
        STILL_CLOSING
    }
}

impl<Owner, CloseRefused: WorthQueryUserFacingRefusal> WorthQueryUserFacingRefusal
    for WorthQueryHomeClosePolicyRefused<Owner, CloseRefused>
{
    fn user_summary(&self) -> Summary {
        match self {
            Self::Admission { .. } => CAPACITY,
            Self::Close(refused) => refused.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryHomeRetirementStop {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Admission(_) => CAPACITY,
            Self::CloseIncomplete => STILL_CLOSING,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationHomeCheckpointDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::MemoryLifecycleRequired => Summary::with_message(
                Category::Unsupported,
                "Only data saved on this device can be checkpointed.",
            ),
            // Pending coupled output records stay in retryable owner custody;
            // the checkpoint can be attempted again once they flush.
            Self::CoupledOutputRetention(_) => Summary::with_message(
                Category::Unavailable,
                "The app is still saving recent changes. Try again in a moment.",
            ),
            Self::Directory(_) => STORAGE,
        }
    }
}
