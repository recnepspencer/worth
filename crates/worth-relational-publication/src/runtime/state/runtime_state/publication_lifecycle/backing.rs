//! Type-erased shared handle onto `RelationalRuntimePublicationLifecycle`.
//!
//! Uses unsized coercion on the existing `Arc<RelationalRuntimePublicationLifecycle>`
//! allocation so rustdoc auto-trait synthesis does not traverse live candidates
//! (`PreparedGenerationBatch` / `DerivedIndexEntries`) through publication
//! bindings carried by host runtimes.

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use worth_relational_mutation::internal_port::validation::candidate_source::registration_observation::ManagedSourceRegistration;

use super::RelationalRuntimePublicationLifecycle;

trait PublicationLifecycleBacking: Send + Sync {
    fn as_lifecycle(&self) -> &RelationalRuntimePublicationLifecycle;
    fn into_source_registration(self: Arc<Self>) -> Arc<dyn ManagedSourceRegistration>;
}

impl PublicationLifecycleBacking for RelationalRuntimePublicationLifecycle {
    fn as_lifecycle(&self) -> &RelationalRuntimePublicationLifecycle {
        self
    }

    fn into_source_registration(self: Arc<Self>) -> Arc<dyn ManagedSourceRegistration> {
        self
    }
}

#[derive(Clone)]
pub(crate) struct SharedPublicationLifecycle(Arc<dyn PublicationLifecycleBacking>);

impl SharedPublicationLifecycle {
    pub(super) fn new(lifecycle: RelationalRuntimePublicationLifecycle) -> Self {
        let backing: Arc<RelationalRuntimePublicationLifecycle> = Arc::new(lifecycle);
        Self(backing)
    }

    pub(super) fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(Arc::as_ptr(&self.0), Arc::as_ptr(&other.0))
    }

    pub(super) fn strong_count(&self) -> usize {
        Arc::strong_count(&self.0)
    }

    pub(super) fn weak_count(&self) -> usize {
        Arc::weak_count(&self.0)
    }

    pub(super) fn to_source_registration(&self) -> Arc<dyn ManagedSourceRegistration> {
        Arc::clone(&self.0).into_source_registration()
    }
}

impl Deref for SharedPublicationLifecycle {
    type Target = RelationalRuntimePublicationLifecycle;

    fn deref(&self) -> &Self::Target {
        PublicationLifecycleBacking::as_lifecycle(&*self.0)
    }
}

impl fmt::Debug for SharedPublicationLifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}
