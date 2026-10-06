//! Selection of new scope segment IDs before a graph epoch publishes topology.
use std::collections::BTreeMap;

use super::{
    InternedPartitionSubscription, InternedScopePath, PartitionInterner, PartitionSubscription,
    PartitionTokenId,
};
use crate::data::error::SignalError;

pub(crate) struct PreparedPartitionInternerExpansion {
    base_count: usize,
    new_segments: Vec<String>,
    selected: BTreeMap<String, PartitionTokenId>,
}

impl PartitionInterner {
    pub(crate) fn prepare_expansion_for_subscriptions<'a>(
        &self,
        scopes: impl IntoIterator<Item = &'a PartitionSubscription>,
    ) -> Result<PreparedPartitionInternerExpansion, SignalError> {
        let base_count = self.segments.len();
        let mut new_segments = Vec::new();
        let mut selected = BTreeMap::new();
        for scope in scopes {
            for segment in scope.path().segments() {
                if self.segment_lookup.contains_key(segment) || selected.contains_key(segment) {
                    continue;
                }
                let index = base_count.checked_add(new_segments.len()).ok_or_else(|| {
                    SignalError::invalid_input("scope interner capacity overflow")
                })?;
                let id = u32::try_from(index)
                    .map_err(|_| SignalError::invalid_input("scope interner ID overflow"))?;
                selected.insert(segment.clone(), PartitionTokenId(id));
                new_segments.push(segment.clone());
            }
        }
        Ok(PreparedPartitionInternerExpansion {
            base_count,
            new_segments,
            selected,
        })
    }

    pub(crate) fn publish_prepared_expansion(
        &mut self,
        prepared: PreparedPartitionInternerExpansion,
    ) {
        assert_eq!(
            self.segments.len(),
            prepared.base_count,
            "scope interner drifted"
        );
        for (index, segment) in prepared.new_segments.iter().enumerate() {
            let selected = self.intern_segment(segment);
            assert_eq!(
                selected,
                PartitionTokenId((prepared.base_count + index) as u32)
            );
        }
    }
}

impl PreparedPartitionInternerExpansion {
    pub(crate) fn interned(
        &self,
        interner: &PartitionInterner,
        scope: &PartitionSubscription,
    ) -> InternedPartitionSubscription {
        let ids = scope
            .path()
            .segments()
            .iter()
            .map(|segment| {
                interner
                    .segment_lookup
                    .get(segment)
                    .or_else(|| self.selected.get(segment))
                    .copied()
                    .expect("prepared expansion covers every scope segment")
            })
            .collect::<Vec<_>>();
        InternedPartitionSubscription::new(
            InternedScopePath::new(&ids).expect("validated scope path"),
            scope.coverage(),
        )
    }
}
