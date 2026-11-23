//! Persistent index payload selection, independent of the observing root stamp.
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexId};
use std::sync::Arc;
#[derive(Clone, Debug, Default)]
pub(crate) struct IndexSelectionState {
    pub(super) entries: im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>,
    pub(super) global: im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>,
    pub(super) global_empty: im::OrdSet<DerivedIndexId>,
    pub(super) empty: im::OrdSet<DerivedIndexId>,
}

pub(crate) type SharedIndexSelectionState = Arc<dyn IndexSelectionBacking>;

/// Immutable selection shares carry the exact Native allocation and prove its
/// thread safety at the owner. A selected edit detaches only when shared.
pub(crate) trait IndexSelectionBacking: std::fmt::Debug + Send + Sync {
    fn state(&self) -> &IndexSelectionState;
    fn state_mut(&mut self) -> &mut IndexSelectionState;
}

impl IndexSelectionBacking for IndexSelectionState {
    fn state(&self) -> &Self {
        self
    }

    fn state_mut(&mut self) -> &mut Self {
        self
    }
}

impl std::ops::Deref for dyn IndexSelectionBacking {
    type Target = IndexSelectionState;

    fn deref(&self) -> &Self::Target {
        self.state()
    }
}

impl IndexSelectionState {
    pub(super) fn make_mut(state: &mut SharedIndexSelectionState) -> &mut Self {
        // These selections have no weak references. An exclusive reference
        // to the sole share is sufficient, and a detach copies only map roots.
        if Arc::strong_count(state) != 1 {
            *state = Arc::new((***state).clone());
        }
        Arc::get_mut(state)
            .expect("exclusive retained selection")
            .state_mut()
    }
}
