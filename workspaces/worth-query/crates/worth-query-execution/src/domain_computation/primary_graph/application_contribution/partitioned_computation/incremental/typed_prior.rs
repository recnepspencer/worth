//! A typed borrow of a prior always co-holds its state custody.
use super::retained::RetainedPartitions;
use crate::domain_computation::primary_graph::output_lineage::CustodiedComputation;
use std::sync::Arc;

pub(super) struct TypedPrior<Key, Item, Reduced> {
    typed: Arc<RetainedPartitions<Key, Item, Reduced>>,
    // Release the typed Arc before the custody that charges it.
    state: Arc<CustodiedComputation>,
}

impl<Key, Item, Reduced> Clone for TypedPrior<Key, Item, Reduced> {
    fn clone(&self) -> Self {
        Self {
            typed: Arc::clone(&self.typed),
            state: Arc::clone(&self.state),
        }
    }
}

impl<Key, Item, Reduced> TypedPrior<Key, Item, Reduced> {
    pub(super) fn typed(&self) -> &RetainedPartitions<Key, Item, Reduced> {
        &self.typed
    }

    pub(super) fn into_state(self) -> Arc<CustodiedComputation> {
        let Self { typed, state } = self;
        drop(typed);
        state
    }
}

impl<Key: Send + Sync + 'static, Item: Send + Sync + 'static, Reduced: Send + Sync + 'static>
    TypedPrior<Key, Item, Reduced>
{
    pub(super) fn from_state(state: Arc<CustodiedComputation>) -> Self {
        let typed = Arc::clone(&state.typed)
            .downcast::<RetainedPartitions<Key, Item, Reduced>>()
            .expect("the installation fixes the retained owner, item, key and result types");
        Self { typed, state }
    }
}
