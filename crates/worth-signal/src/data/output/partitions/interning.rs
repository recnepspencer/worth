use super::{
    InternedPartitionSubscription, InternedScopePath, PartitionInterner, PartitionSubscription,
    PartitionTokenId, ScopePath,
};

impl PartitionInterner {
    pub fn intern_subscription(
        &mut self,
        subscription: &PartitionSubscription,
    ) -> InternedPartitionSubscription {
        let segments: Vec<_> = subscription
            .path()
            .segments()
            .iter()
            .map(|segment| self.intern_segment(segment))
            .collect();
        InternedPartitionSubscription::new(
            InternedScopePath::new(&segments).expect("public path was validated"),
            subscription.coverage(),
        )
    }
    pub(crate) fn resolve_subscription(
        &self,
        subscription: &PartitionSubscription,
    ) -> Option<InternedPartitionSubscription> {
        let segments: Option<Vec<_>> = subscription
            .path()
            .segments()
            .iter()
            .map(|segment| self.segment_lookup.get(segment).copied())
            .collect();
        let path = InternedScopePath::new(&segments?).ok()?;
        Some(InternedPartitionSubscription::new(
            path,
            subscription.coverage(),
        ))
    }
    pub(crate) fn resolve_known_prefix(&self, path: &ScopePath) -> Option<InternedScopePath> {
        let segments: Vec<_> = path
            .segments()
            .iter()
            .map_while(|segment| self.segment_lookup.get(segment).copied())
            .collect();
        InternedScopePath::new(&segments).ok()
    }
    pub fn token_count(&self) -> usize {
        self.segments.len()
    }
    pub(super) fn intern_segment(&mut self, segment: &str) -> PartitionTokenId {
        if let Some(id) = self.segment_lookup.get(segment).copied() {
            return id;
        }
        let id = PartitionTokenId(self.segments.len() as u32);
        self.segments.push_back(segment.to_owned());
        self.segment_lookup.insert(segment.to_owned(), id);
        id
    }
    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            segments: self.segments.operational_clone(),
            segment_lookup: self.segment_lookup.operational_clone(),
        }
    }
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            segments: self.segments.fork_persistent(),
            segment_lookup: self.segment_lookup.fork_persistent(),
        }
    }
    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            segments: self.segments.clone(),
            segment_lookup: self.segment_lookup.fork_storage_identity(),
        }
    }
    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.segments.shares_storage_with(&other.segments)
            && self.segment_lookup.ptr_eq(&other.segment_lookup)
    }
}
