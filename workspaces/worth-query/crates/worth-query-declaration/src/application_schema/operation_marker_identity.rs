//! Declaration-owned identity for an application operation marker.

use super::ApplicationStructuredValueBinding;

/// Exact schema membership and stable input meaning declared for an operation.
pub trait ApplicationOperationMarkerIdentity<Schema> {
    type InputBinding: ApplicationStructuredValueBinding;

    const IDENTIFIER: &'static str;
}
