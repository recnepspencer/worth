//! A managed Source read a coupled decision took from its committed basis
//! because Native proved the candidate left it unchanged. The committed
//! observation cannot be compared at a candidate image, so this claim is its
//! candidate support: the coupled owner marks every later append that
//! changes the Source or a Parent read, and the decision's support is equal
//! between two images exactly when no marked append lies between them.
use std::sync::{Arc, Mutex, PoisonError};
use worth_relational::facade::{
    identity::EntityId, mvcc::ManagedIndexParentReadFootprint,
    runtime::RelationalAppliedCandidateReadIdentity as Image,
};

pub struct AppliedSourceAlignment {
    /// The installed managed Source the read probed.
    source: Arc<str>,
    scope: EntityId,
    footprint: ManagedIndexParentReadFootprint,
    /// The candidate image the read was proved unchanged at.
    observed: Image,
    /// The image right after each later append that changed the read.
    changed: Mutex<Vec<Image>>,
}

impl AppliedSourceAlignment {
    pub fn new(
        source: &str,
        scope: EntityId,
        footprint: ManagedIndexParentReadFootprint,
        observed: Image,
    ) -> Self {
        Self {
            source: Arc::from(source),
            scope,
            footprint,
            observed,
            changed: Mutex::new(Vec::new()),
        }
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn scope(&self) -> EntityId {
        self.scope
    }
    pub fn footprint(&self) -> &ManagedIndexParentReadFootprint {
        &self.footprint
    }
    /// The append that produced image `at` changed the Source or a Parent read.
    pub fn mark_changed(&self, at: &Image) {
        let mut changed = self.changed.lock().unwrap_or_else(PoisonError::into_inner);
        if !changed.contains(at) {
            changed.push(at.clone());
        }
    }
    /// Whether no marked append lies after `earlier` and at or before `image`.
    /// An image that does not extend the read's own candidate proves nothing.
    pub fn unchanged_between(&self, earlier: &Image, image: &Image) -> bool {
        image.includes_assembly_prefix(&self.observed)
            && !self
                .changed
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .any(|at| image.includes_assembly_prefix(at) && !earlier.includes_assembly_prefix(at))
    }
}
