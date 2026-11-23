//! Immutable author declarations for which producer owns each source scope.
use crate::identity::data::KindId;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, Default)]
pub(crate) struct InstalledSourceScopeOwnership(Arc<BTreeMap<String, KindId>>);

impl InstalledSourceScopeOwnership {
    pub(crate) fn new(producers: impl IntoIterator<Item = (String, KindId)>) -> Self {
        Self(Arc::new(producers.into_iter().collect()))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub(crate) fn lookup_work(&self, producer: &str) -> usize {
        if self.0.is_empty() {
            return 0;
        }
        let height = usize::BITS as usize - self.0.len().max(1).leading_zeros() as usize;
        self.0
            .len()
            .min(11)
            .saturating_mul(height)
            .saturating_add(1)
            .saturating_mul(producer.len().saturating_add(1))
    }
    pub(crate) fn scope_kind(&self, producer: &str) -> Option<KindId> {
        self.0.get(producer).copied()
    }
}
