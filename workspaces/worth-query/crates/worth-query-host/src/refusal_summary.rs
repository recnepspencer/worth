//! End-user summaries for every primary facade refusal, denial and stop.
//!
//! Each refusal keeps its exact typed topology for code and `{:?}` logs;
//! [`WorthQueryUserFacingRefusal`] adds a closed category, a retry hint and
//! one jargon-free sentence suitable for a UI banner.
//!
//! ```
//! use worth_query_host::facade::{
//!     WorthQueryHomeUndoDenial, WorthQueryRefusalCategory, WorthQueryUserFacingRefusal,
//! };
//! let denial = WorthQueryHomeUndoDenial::Conflict;
//! assert_eq!(denial.refusal_category(), WorthQueryRefusalCategory::Conflict);
//! assert!(!denial.user_message().contains("Conflict"));
//! ```
mod correction;
mod custody;
mod home;
mod preview;
mod query_read;
mod request;
mod session;
mod summary;
mod transition;
mod world_history;

pub use summary::{
    WorthQueryRefusalCategory, WorthQueryRefusalSummary, WorthQueryUserFacingRefusal,
};
