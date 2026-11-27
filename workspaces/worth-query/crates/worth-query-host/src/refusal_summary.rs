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
//!
//! An application rule names itself with a [`WorthQueryDomainRefusal`]: a
//! stable `code` to branch on and an operator `message` shown verbatim. A
//! hand-written handler returns `HandlerResult::domain_refusal(code, message)`;
//! an installed invariant declares `WorthQueryDomainRefusalBinding` as its
//! `RefusalBinding`; `denial.domain_refusal()` on a session run denial reads
//! either one back, and its summary keeps both.
//!
//! ```
//! use worth_query_host::facade::primary_graph::{HandlerResult, WorthQueryOperationBodyDenial};
//! use worth_query_host::facade::{
//!     WorthQueryDomainRefusal, WorthQueryDomainRefusalCarrier, WorthQueryDomainRefusalDenial,
//!     WorthQueryRefusalCategory, WorthQueryUserFacingRefusal,
//! };
//! let expired = WorthQueryDomainRefusal::new("lot_expired", "Lot 7 expired on 3 May.").unwrap();
//! let summary = WorthQueryOperationBodyDenial::from(expired.clone()).user_summary();
//! assert_eq!(summary.category(), WorthQueryRefusalCategory::Validation);
//! assert_eq!(summary.user_message(), "Lot 7 expired on 3 May.");
//! assert_eq!(summary.domain_refusal().map(|r| r.code()), Some("lot_expired"));
//!
//! // In `OperationHandler::decide`, refuse with one call:
//! let refused = HandlerResult::<(), WorthQueryOperationBodyDenial>::domain_refusal(
//!     "lot_expired",
//!     "Lot 7 expired on 3 May.",
//! );
//! let HandlerResult::DomainDenied(denial) = refused else { unreachable!() };
//! assert_eq!(denial.domain_refusal(), Some(&expired));
//!
//! // Oversized or malformed reasons are refused, never truncated.
//! let long = "x".repeat(257);
//! assert!(matches!(
//!     WorthQueryDomainRefusal::new("lot_expired", &long),
//!     Err(WorthQueryDomainRefusalDenial::MessageTooLong { maximum: 256, observed: 257 })
//! ));
//! ```
mod audit_inspection;
mod correction;
mod custody;
mod eligibility;
mod history_read;
mod home;
mod live_open;
mod local_setup;
mod managed_index;
mod operation_close;
mod preview;
mod query_read;
mod request;
mod session;
mod stops;
mod summary;
mod temporal_occurrence;
mod transition;
mod validation;
mod world_history;

pub use summary::{
    WorthQueryRefusalCategory, WorthQueryRefusalSummary, WorthQueryUserFacingRefusal,
};
pub use worth_query_execution::facade::domain_refusal::{
    WorthQueryDomainRefusal, WorthQueryDomainRefusalCarrier, WorthQueryDomainRefusalDenial,
    WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES,
    WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES,
};
