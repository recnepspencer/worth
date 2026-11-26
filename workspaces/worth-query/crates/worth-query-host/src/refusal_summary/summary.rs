//! The closed refusal vocabulary and the end-user summary it renders.
use std::borrow::Cow;
use std::fmt;

/// What kind of problem stopped an operation, from the end user's side.
///
/// Closed and machine-readable: branch on it for UI treatment (retry button,
/// sign-in prompt, refresh). The variant names never reach end-user text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WorthQueryRefusalCategory {
    /// The input or a rule declared by the application refused the request.
    Validation,
    /// Something changed since the user last saw it, or the step already happened.
    Conflict,
    /// The signed-in user may not perform or see this.
    PermissionDenied,
    /// The session's sign-in proof expired. It is not a missing permission:
    /// a renewed session (or a self-renewing one, on its next call) succeeds.
    SessionExpired,
    /// The item, or the history needed to act on it, no longer exists.
    NotFound,
    /// The data or request belongs to a different program version or setup.
    Unsupported,
    /// A work, memory or size limit stopped the request before it finished.
    Capacity,
    /// The request was cancelled or ran past its deadline.
    Interrupted,
    /// The data is busy or still settling; the same request can succeed later.
    Unavailable,
    /// Reading or saving durable data failed.
    Storage,
    /// An internal defect or inconsistency; report it rather than retry.
    Internal,
}

impl WorthQueryRefusalCategory {
    /// Whether repeating the same request unchanged can reasonably succeed.
    pub const fn retryable(self) -> bool {
        matches!(
            self,
            Self::Capacity | Self::Interrupted | Self::Unavailable | Self::SessionExpired
        )
    }

    /// A short, jargon-free sentence describing the category to an end user.
    pub const fn default_message(self) -> &'static str {
        match self {
            Self::Validation => "That change isn't allowed. Check the values and try again.",
            Self::Conflict => {
                "This was changed since you last saw it. Refresh to see the latest version."
            }
            Self::PermissionDenied => "You don't have permission to do that.",
            Self::SessionExpired => {
                "Your sign-in expired before this finished. Nothing was changed. Try again."
            }
            Self::NotFound => "That item, or the history needed for this, is no longer available.",
            Self::Unsupported => "This data was made by a different version of the app.",
            Self::Capacity => "That was too large to finish. Try again with less at once.",
            Self::Interrupted => "That took too long or was cancelled. Try again.",
            Self::Unavailable => "This is busy right now. Try again in a moment.",
            Self::Storage => "The app couldn't read or save its data on this device.",
            Self::Internal => "Something went wrong inside the app. Please report this problem.",
        }
    }
}

/// A refusal rendered for an end user: a category plus one safe sentence.
///
/// The message never contains Rust type names, `Debug` structure, internal
/// paths or identifiers; log the original refusal with `{:?}` for developers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRefusalSummary {
    category: WorthQueryRefusalCategory,
    message: Cow<'static, str>,
}

impl WorthQueryRefusalSummary {
    /// The category's own default sentence.
    pub const fn of(category: WorthQueryRefusalCategory) -> Self {
        Self {
            category,
            message: Cow::Borrowed(category.default_message()),
        }
    }

    /// A more specific fixed sentence for the same category.
    pub const fn with_message(category: WorthQueryRefusalCategory, message: &'static str) -> Self {
        Self {
            category,
            message: Cow::Borrowed(message),
        }
    }

    pub const fn category(&self) -> WorthQueryRefusalCategory {
        self.category
    }

    /// The sentence to show in a banner, toast or dialog.
    pub fn user_message(&self) -> &str {
        &self.message
    }

    pub const fn retryable(&self) -> bool {
        self.category.retryable()
    }
}

impl fmt::Display for WorthQueryRefusalSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// A refusal, denial or stop that can explain itself to an end user.
///
/// Every primary facade refusal implements it, so a UI shows
/// `refusal.user_message()` while logs keep `{refusal:?}`. Applications
/// implement it for their own domain denials to make
/// [`crate::facade::WorthQuerySessionRunDenial`] fully readable.
pub trait WorthQueryUserFacingRefusal {
    fn user_summary(&self) -> WorthQueryRefusalSummary;

    fn refusal_category(&self) -> WorthQueryRefusalCategory {
        self.user_summary().category()
    }

    fn user_message(&self) -> Cow<'static, str> {
        self.user_summary().message
    }

    fn retryable(&self) -> bool {
        self.refusal_category().retryable()
    }
}

impl<T: WorthQueryUserFacingRefusal + ?Sized> WorthQueryUserFacingRefusal for Box<T> {
    fn user_summary(&self) -> WorthQueryRefusalSummary {
        (**self).user_summary()
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryRefusalSummary {
    fn user_summary(&self) -> WorthQueryRefusalSummary {
        self.clone()
    }
}
