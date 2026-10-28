//! A home-declared closed, versioned journal namespace.
use super::OrderedJournalName;
use std::collections::BTreeSet;
use worth_store_physical_format::RecordAnchorName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedJournalRegistry {
    version: u16,
    names: BTreeSet<RecordAnchorName>,
}

impl Default for OrderedJournalRegistry {
    fn default() -> Self {
        Self {
            version: 1,
            names: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedJournalRegistryDenial {
    InvalidVersion,
    MalformedName,
    DuplicateName,
    AnchoredRecordCollision,
}

impl OrderedJournalRegistry {
    /// Declares the complete journal set for this home format. Anchored names
    /// are checked here; all anchored-record doors also enforce disjointness.
    pub fn declare(
        version: u16,
        journals: &[&str],
        anchored: &[&str],
    ) -> Result<Self, OrderedJournalRegistryDenial> {
        use OrderedJournalRegistryDenial as Denial;
        if version == 0 {
            return Err(Denial::InvalidVersion);
        }
        let mut names = BTreeSet::new();
        for name in journals {
            let name = RecordAnchorName::new(name).ok_or(Denial::MalformedName)?;
            if !names.insert(name) {
                return Err(Denial::DuplicateName);
            }
        }
        let mut anchors = BTreeSet::new();
        for name in anchored {
            let name = RecordAnchorName::new(name).ok_or(Denial::MalformedName)?;
            if !anchors.insert(name) {
                return Err(Denial::DuplicateName);
            }
            if names.contains(&name) {
                return Err(Denial::AnchoredRecordCollision);
            }
        }
        Ok(Self { version, names })
    }
    pub const fn version(&self) -> u16 {
        self.version
    }
    pub fn contains(&self, name: OrderedJournalName) -> bool {
        self.reserves(name.anchor())
    }
    pub(in crate::physical_runtime) fn reserves(&self, name: RecordAnchorName) -> bool {
        self.names.contains(&name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn home_registry_refuses_malformed_duplicate_and_overlapping_names() {
        use OrderedJournalRegistryDenial::*;
        for (journals, anchors, expected) in [
            (vec!["bad Name"], vec![], MalformedName),
            (vec!["a", "a"], vec![], DuplicateName),
            (vec!["a"], vec!["a"], AnchoredRecordCollision),
            (vec![], vec!["a", "a"], DuplicateName),
        ] {
            assert_eq!(
                OrderedJournalRegistry::declare(1, &journals, &anchors),
                Err(expected)
            );
        }
        assert_eq!(
            OrderedJournalRegistry::declare(0, &[], &[]),
            Err(InvalidVersion)
        );
        let registry = OrderedJournalRegistry::declare(3, &["owner.journal"], &["home"]).unwrap();
        assert!(registry.contains(OrderedJournalName::new("owner.journal").unwrap()));
        assert!(!registry.contains(OrderedJournalName::new("other.journal").unwrap()));
    }
}
