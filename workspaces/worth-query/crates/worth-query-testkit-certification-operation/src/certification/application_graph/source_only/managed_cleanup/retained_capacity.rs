//! Retained delivery loses real finite World observation capacity before effect.
pub use super::*;
pub use crate::facade::certification::source_only::{
    host,
    retained::{operation, RetainedProgram},
};
pub use std::time::Duration;
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryProductBranchAdmissionDenial,
    WorthQueryProductBranchAdmissionRefusalKind,
};
pub use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;

pub fn authentication_stop(
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    let stop = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    stop.cancel();
    worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
        None,
        stop.token(),
    )
}
