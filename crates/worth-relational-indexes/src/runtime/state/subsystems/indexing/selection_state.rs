//! Persistent index payload selection, independent of the observing root stamp.
use std::sync::Arc;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;
use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexId;
#[derive(Clone, Debug, Default)]
pub(crate) struct IndexSelectionState {
    pub(super) entries: im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>,
    pub(super) global: im::OrdMap<DerivedIndexId, Arc<DerivedIndexGeneration>>,
    pub(super) global_empty: im::OrdSet<DerivedIndexId>,
    pub(super) empty: im::OrdSet<DerivedIndexId>,
}
