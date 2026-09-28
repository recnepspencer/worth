use std::collections::BTreeMap;

/// Structural work whose exact value is part of a certification claim.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryCertificationCounter {
    BoundaryChecks,
    AuthorityMints,
    ProviderContacts,
    GraphLookups,
    DependencyVisits,
    CollectionRowsVisited,
    ReplayAdmissions,
    ReversalAdmissions,
}

/// Canonical sparse counter ledger. Missing counters mean exact zero.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryCertificationCounters {
    values: BTreeMap<WorthQueryCertificationCounter, u64>,
}

/// Why `WorthQueryCertificationCounters::exact` refused a counter set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCertificationCounterSetDenial {
    /// The same counter was supplied more than once.
    DuplicateCounter(WorthQueryCertificationCounter),
}

impl WorthQueryCertificationCounters {
    pub fn exact(
        values: impl IntoIterator<Item = (WorthQueryCertificationCounter, u64)>,
    ) -> Result<Self, WorthQueryCertificationCounterSetDenial> {
        let mut exact = BTreeMap::new();
        for (counter, value) in values {
            if exact.insert(counter, value).is_some() {
                return Err(WorthQueryCertificationCounterSetDenial::DuplicateCounter(
                    counter,
                ));
            }
        }
        exact.retain(|_, value| *value != 0);
        Ok(Self { values: exact })
    }

    pub fn value(&self, counter: WorthQueryCertificationCounter) -> u64 {
        self.values.get(&counter).copied().unwrap_or(0)
    }

    pub fn values(&self) -> &BTreeMap<WorthQueryCertificationCounter, u64> {
        &self.values
    }
}
