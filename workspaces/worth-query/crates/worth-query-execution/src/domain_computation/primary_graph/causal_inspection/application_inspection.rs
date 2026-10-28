//! Fresh application inspection owns admission, original target and its terminal.
mod admission;
mod identity;
mod report;
mod request;
mod result;
mod stop;
pub use request::{
    WorthQueryApplicationInspectionRequest, WorthQueryObservedInspectionRequest,
    WorthQueryReferenceOnlyInspectionRequest,
};
pub use result::WorthQueryObservedApplicationInspection;
pub use stop::{
    WorthQueryApplicationInspectionCanonicalRefusal, WorthQueryApplicationInspectionDenial,
    WorthQueryApplicationInspectionResourceRefusal, WorthQueryApplicationInspectionStopKind,
};
#[cfg(test)]
#[path = "application_inspection/tests/scope_replacement.rs"]
mod scope_replacement;
mod target;
#[cfg(test)]
mod tests;
pub(in crate::domain_computation) use target::WorthQueryApplicationInspectionTarget;
