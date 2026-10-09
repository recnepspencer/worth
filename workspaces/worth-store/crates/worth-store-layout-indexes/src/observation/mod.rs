//! Read-only observations issued by ordinary layout operations.
//!
//! Observations describe what production owners executed. They do not satisfy
//! layout admission, planning, readiness, execution, or readmission APIs.

mod access;
mod integrity;
mod maintenance;
mod materialization;
mod owner_case;

pub use crate::access::shape::AccessShape;
pub use owner_case::{ObserveOwnerCase, OwnerCaseObservation};
