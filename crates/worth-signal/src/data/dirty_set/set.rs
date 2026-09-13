use super::impact::DomainImpact;

/// Batched dirty map from domain -> impact.
#[derive(Debug, Clone)]
pub struct BatchedDirtySet<D: Copy + Ord, I: Copy + Ord> {
    by_domain: crate::data::persistent_ord_map::PersistentOrdMap<D, DomainImpact<I>>,
}

impl<D: Copy + Ord, I: Copy + Ord> BatchedDirtySet<D, I> {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            by_domain: self.by_domain.fork_persistent(),
        }
    }

    /// Create an empty dirty set.
    pub fn new() -> Self {
        Self {
            by_domain: crate::data::persistent_ord_map::PersistentOrdMap::new(),
        }
    }

    /// Whether all domains are clean.
    pub fn is_empty(&self) -> bool {
        self.by_domain.values().all(DomainImpact::is_empty)
    }

    /// Mark one domain globally dirty.
    pub fn mark_domain_global(&mut self, domain: D) {
        self.by_domain.entry(domain).or_default().mark_global();
    }

    /// Mark one scoped impact on one domain.
    pub fn mark_domain_scoped(&mut self, domain: D, impact: I) {
        self.by_domain.entry(domain).or_default().add_scoped(impact);
    }

    /// Mark multiple scoped impacts on one domain.
    pub fn mark_domain_scoped_many<T>(&mut self, domain: D, impacts: T)
    where
        T: IntoIterator<Item = I>,
    {
        self.by_domain
            .entry(domain)
            .or_default()
            .add_scoped_many(impacts);
    }

    /// Merge a precomputed impact for one domain.
    pub fn merge_domain_impact(&mut self, domain: D, impact: DomainImpact<I>) {
        if impact.is_empty() {
            self.by_domain.remove(&domain);
            return;
        }
        let current = self.by_domain.entry(domain).or_default();
        if impact.is_global() {
            current.mark_global();
            return;
        }
        current.add_scoped_many(impact.scoped());
        if current.is_empty() {
            self.by_domain.remove(&domain);
        }
    }

    /// Whether a domain currently has dirty impact.
    pub fn is_domain_dirty(&self, domain: D) -> bool {
        self.by_domain
            .get(&domain)
            .map(|impact| !impact.is_empty())
            .unwrap_or(false)
    }

    /// Deterministic iterator of dirty domains.
    pub fn dirty_domains(&self) -> impl Iterator<Item = D> + '_ {
        self.by_domain
            .iter()
            .filter_map(|(domain, impact)| (!impact.is_empty()).then_some(*domain))
    }

    /// Return the first dirty domain in deterministic order.
    pub fn first_dirty_domain(&self) -> Option<D> {
        self.by_domain
            .iter()
            .find_map(|(domain, impact)| (!impact.is_empty()).then_some(*domain))
    }

    /// Read-only impact for a domain.
    pub fn impact_for(&self, domain: D) -> Option<&DomainImpact<I>> {
        self.by_domain.get(&domain)
    }

    /// Take and clear impact for one domain.
    pub fn take_domain_impact(&mut self, domain: D) -> Option<DomainImpact<I>> {
        self.by_domain.remove(&domain)
    }

    /// Clear all dirty impacts.
    pub fn clear(&mut self) {
        self.by_domain.clear();
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            by_domain: self.by_domain.fork_storage_identity(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.by_domain.ptr_eq(&other.by_domain)
    }
}

impl<D: Copy + Ord, I: Copy + Ord> Default for BatchedDirtySet<D, I> {
    fn default() -> Self {
        Self::new()
    }
}
