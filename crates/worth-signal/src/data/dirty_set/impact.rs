/// Dirty impact for one domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainImpact<I: Copy + Ord> {
    global: bool,
    scoped: im::OrdSet<I>,
}

impl<I: Copy + Ord> DomainImpact<I> {
    /// Empty, clean impact.
    pub fn empty() -> Self {
        Self {
            global: false,
            scoped: im::OrdSet::new(),
        }
    }

    /// Whether this domain is fully dirty.
    pub fn is_global(&self) -> bool {
        self.global
    }

    /// Whether no dirty signal is recorded.
    pub fn is_empty(&self) -> bool {
        !self.global && self.scoped.is_empty()
    }

    /// Mark this domain globally dirty and drop scoped keys.
    pub fn mark_global(&mut self) {
        self.global = true;
        self.scoped.clear();
    }

    /// Add one scoped impact key.
    pub fn add_scoped(&mut self, impact: I) {
        if !self.global {
            self.scoped.insert(impact);
        }
    }

    /// Add multiple scoped impact keys.
    pub fn add_scoped_many<T>(&mut self, impacts: T)
    where
        T: IntoIterator<Item = I>,
    {
        if self.global {
            return;
        }
        self.scoped.extend(impacts);
    }

    /// Iterate scoped impact keys in deterministic order.
    pub fn scoped(&self) -> impl Iterator<Item = I> + '_ {
        self.scoped.iter().copied()
    }
}

impl<I: Copy + Ord> Default for DomainImpact<I> {
    fn default() -> Self {
        Self::empty()
    }
}
