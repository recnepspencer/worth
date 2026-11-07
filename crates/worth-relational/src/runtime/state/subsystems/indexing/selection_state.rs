//! Persistent index payload selection, independent of the observing root stamp.
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexId};
use std::sync::Arc;
#[derive(Clone, Debug, Default)]
pub(crate) struct IndexSelectionState {
    pub(super) entries: im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>,
    pub(super) empty: im::OrdSet<DerivedIndexId>,
}
