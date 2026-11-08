//! Retained exact revisions and latest-by-key selection share one contract store.
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
};

#[derive(Debug)]
pub(super) struct RetainedContracts {
    contracts: Vec<AspectContract>,
    latest: Vec<usize>,
    exact: Vec<usize>,
}
impl RetainedContracts {
    pub(super) fn new(contracts: Vec<AspectContract>) -> Self {
        let mut latest: Vec<_> = (0..contracts.len()).collect();
        latest.sort_unstable_by(|&a, &b| {
            contracts[a]
                .key()
                .cmp(contracts[b].key())
                .then_with(|| b.cmp(&a))
        });
        latest.dedup_by(|a, b| contracts[*a].key() == contracts[*b].key());
        latest.shrink_to_fit();
        let mut exact: Vec<_> = (0..contracts.len()).collect();
        exact.sort_unstable_by(|&a, &b| {
            basis(&contracts[a])
                .cmp(&basis(&contracts[b]))
                .then_with(|| a.cmp(&b))
        });
        exact.dedup_by(|a, b| basis(&contracts[*a]) == basis(&contracts[*b]));
        exact.shrink_to_fit();
        Self {
            contracts,
            latest,
            exact,
        }
    }
    pub(super) fn all(&self) -> &[AspectContract] {
        &self.contracts
    }
    pub(super) fn get(&self, key: &AspectKey) -> Option<&AspectContract> {
        self.latest
            .binary_search_by(|&i| self.contracts[i].key().cmp(key))
            .ok()
            .map(|i| &self.contracts[self.latest[i]])
    }
    pub(super) fn exact(
        &self,
        key: &AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    ) -> Option<&AspectContract> {
        self.exact
            .binary_search_by(|&i| basis(&self.contracts[i]).cmp(&(key, identity, revision)))
            .ok()
            .map(|i| &self.contracts[self.exact[i]])
    }
    pub(super) fn allocation_bytes(&self) -> u64 {
        (size_of::<Self>()
            + (self.latest.capacity() + self.exact.capacity()) * size_of::<usize>()
            + self.contracts.capacity() * size_of::<AspectContract>()) as u64
            + self
                .contracts
                .iter()
                .map(|c| c.owned_allocation_capacity_bytes() as u64)
                .sum::<u64>()
    }
}
fn basis(contract: &AspectContract) -> (&AspectKey, AspectIdentity, AspectContractRevision) {
    (contract.key(), contract.identity(), contract.revision())
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::ScalarAspectType;

    #[test]
    fn keyed_selection_equals_retained_revision_order_including_missing_bases() {
        let mut contracts = Vec::new();
        for revision in (0..64).rev() {
            for name in ["left", "right"] {
                contracts.push(AspectContract::scalar(
                    AspectKey::new(name).unwrap(),
                    AspectIdentity(revision % 3),
                    AspectContractRevision(revision),
                    ScalarAspectType::String,
                ));
            }
        }
        let mut duplicate = contracts[0].clone();
        duplicate = AspectContract::scalar(
            duplicate.key().clone(),
            duplicate.identity(),
            duplicate.revision(),
            ScalarAspectType::UInt64,
        );
        contracts.push(duplicate);
        let directory = RetainedContracts::new(contracts.clone());
        for name in ["left", "right", "absent"] {
            let key = AspectKey::new(name).unwrap();
            assert_eq!(
                directory.get(&key),
                contracts.iter().rev().find(|c| c.key() == &key)
            );
            for identity in 0..4 {
                for revision in 0..65 {
                    let identity = AspectIdentity(identity);
                    let revision = AspectContractRevision(revision);
                    assert_eq!(
                        directory.exact(&key, identity, revision),
                        contracts.iter().find(|c| c.key() == &key
                            && c.identity() == identity
                            && c.revision() == revision)
                    );
                }
            }
        }
    }
}
