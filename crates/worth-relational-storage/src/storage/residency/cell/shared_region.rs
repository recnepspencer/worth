//! Type-erased shared handle onto `ResidentRegion`.
//!
//! Uses unsized coercion on the existing `Arc<ResidentRegion>` allocation so
//! rustdoc auto-trait synthesis does not traverse deep `SharedColumn` /
//! `VersionedEntityMetadata` / `AspectValue` trees through residency cells and
//! leases.

use std::any::Any;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use super::ResidentRegion;

trait ResidentRegionBacking: Send + Sync {
    fn as_region(&self) -> &ResidentRegion;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl ResidentRegionBacking for ResidentRegion {
    fn as_region(&self) -> &ResidentRegion {
        self
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[derive(Clone)]
pub(in crate::internal_port::storage::residency) struct SharedResidentRegion(
    Arc<dyn ResidentRegionBacking>,
);

impl SharedResidentRegion {
    pub(in crate::internal_port::storage::residency) fn from_arc(
        region: Arc<ResidentRegion>,
    ) -> Self {
        Self(region)
    }

    pub(in crate::internal_port::storage::residency) fn into_arc(self) -> Arc<ResidentRegion> {
        self.0
            .into_any()
            .downcast::<ResidentRegion>()
            .expect("SharedResidentRegion always wraps ResidentRegion")
    }

    pub(in crate::internal_port::storage::residency) fn to_arc(&self) -> Arc<ResidentRegion> {
        Arc::clone(&self.0)
            .into_any()
            .downcast::<ResidentRegion>()
            .expect("SharedResidentRegion always wraps ResidentRegion")
    }
}

impl From<Arc<ResidentRegion>> for SharedResidentRegion {
    fn from(region: Arc<ResidentRegion>) -> Self {
        Self::from_arc(region)
    }
}

impl Deref for SharedResidentRegion {
    type Target = ResidentRegion;

    fn deref(&self) -> &Self::Target {
        ResidentRegionBacking::as_region(&*self.0)
    }
}

impl fmt::Debug for SharedResidentRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}
