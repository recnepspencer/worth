//! Declaration-owned identity for an application effect marker.

use super::ApplicationStructuredValueBinding;

/// Exact schema membership and stable payload meaning declared for an effect.
pub trait ApplicationEffectMarkerIdentity<Schema> {
    type PayloadBinding: ApplicationStructuredValueBinding;

    const IDENTIFIER: &'static str;
}
