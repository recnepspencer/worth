//! Expression conditions under concurrent change: authority revoked or a
//! source replaced after the advance is prepared, an absent document, and
//! the same request key reused.

pub use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial;
pub use worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
pub use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenialKind, WorthQueryEntityResolutionDenialKind,
    WorthQueryOperationAuthorizationDenialKind,
};

pub use super::*;
