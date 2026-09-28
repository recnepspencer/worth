use super::WorthQueryCertificationCounters;
use std::collections::BTreeMap;

/// Provider-neutral semantic result used by the parity oracle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCertificationObservation {
    semantic_facts: BTreeMap<String, String>,
    counters: WorthQueryCertificationCounters,
}

/// Why `WorthQueryCertificationObservation::new` refused an observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryCertificationObservationDenial {
    /// The named semantic fact was supplied more than once.
    DuplicateSemanticFact(String),
}

impl WorthQueryCertificationObservation {
    pub fn new(
        semantic_facts: impl IntoIterator<Item = (String, String)>,
        counters: WorthQueryCertificationCounters,
    ) -> Result<Self, WorthQueryCertificationObservationDenial> {
        let mut facts = BTreeMap::new();
        for (name, value) in semantic_facts {
            if facts.insert(name.clone(), value).is_some() {
                return Err(WorthQueryCertificationObservationDenial::DuplicateSemanticFact(name));
            }
        }
        Ok(Self {
            semantic_facts: facts,
            counters,
        })
    }

    pub fn semantic_facts(&self) -> &BTreeMap<String, String> {
        &self.semantic_facts
    }

    pub fn counters(&self) -> &WorthQueryCertificationCounters {
        &self.counters
    }
}
