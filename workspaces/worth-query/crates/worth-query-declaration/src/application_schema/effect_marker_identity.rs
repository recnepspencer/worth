//! Declaration-owned identity for an application effect marker.

use super::ApplicationRetainedEffectBinding;

/// Exact schema membership and stable payload meaning declared for an effect.
pub trait ApplicationEffectMarkerIdentity<Schema> {
    type PayloadBinding: ApplicationRetainedEffectBinding;

    const IDENTIFIER: &'static str;
}
