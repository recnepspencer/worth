use std::{collections::BTreeMap, fmt, sync::Arc};

use worth_foundational::EquivalenceContractId;

use super::ConstructionDenial;

/// A composition-root-installed comparison over complete canonical encodings.
///
/// The identity must change whenever the predicate's meaning changes. It is
/// available to the host for inclusion in its revision identity.
#[derive(Clone)]
pub struct EquivalencePredicate {
    id: EquivalenceContractId,
    identity: [u8; 32],
    compare: Arc<dyn Fn(&[u8], &[u8]) -> bool + Send + Sync>,
}

impl fmt::Debug for EquivalencePredicate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EquivalencePredicate")
            .field("id", &self.id)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl EquivalencePredicate {
    pub fn new<F>(id: EquivalenceContractId, identity: [u8; 32], compare: F) -> Self
    where
        F: Fn(&[u8], &[u8]) -> bool + Send + Sync + 'static,
    {
        Self {
            id,
            identity,
            compare: Arc::new(compare),
        }
    }

    pub const fn id(&self) -> EquivalenceContractId {
        self.id
    }

    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub(crate) fn equivalent(&self, left: &[u8], right: &[u8]) -> bool {
        (self.compare)(left, right)
    }
}

#[derive(Debug, Default)]
pub(super) struct EquivalenceRegistry {
    predicates: BTreeMap<EquivalenceContractId, EquivalencePredicate>,
}

impl EquivalenceRegistry {
    pub(super) fn new(
        predicates: impl IntoIterator<Item = EquivalencePredicate>,
    ) -> Result<Self, ConstructionDenial> {
        let mut registry = Self::default();
        for predicate in predicates {
            if registry.predicates.contains_key(&predicate.id) {
                return Err(ConstructionDenial::DuplicateEquivalenceContract(
                    predicate.id,
                ));
            }
            if registry
                .predicates
                .values()
                .any(|installed| installed.identity == predicate.identity)
            {
                return Err(ConstructionDenial::DuplicateEquivalenceIdentity(
                    predicate.identity,
                ));
            }
            registry.predicates.insert(predicate.id, predicate);
        }
        Ok(registry)
    }

    pub(super) fn get(&self, id: EquivalenceContractId) -> Option<&EquivalencePredicate> {
        self.predicates.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::{EquivalencePredicate, EquivalenceRegistry};
    use crate::authority::ConstructionDenial;
    use worth_foundational::EquivalenceContractId;

    fn predicate(id: u64, identity: u8) -> EquivalencePredicate {
        EquivalencePredicate::new(
            EquivalenceContractId::new(id),
            [identity; 32],
            |left, right| left == right,
        )
    }

    #[test]
    fn installation_rejects_duplicate_ids_even_with_distinct_identities() {
        let result = EquivalenceRegistry::new([predicate(1, 1), predicate(1, 2)]);
        assert!(matches!(
            result,
            Err(ConstructionDenial::DuplicateEquivalenceContract(id))
                if id == EquivalenceContractId::new(1)
        ));
    }

    #[test]
    fn installation_rejects_identity_reuse_across_ids() {
        let result = EquivalenceRegistry::new([predicate(1, 1), predicate(2, 1)]);
        assert_eq!(
            result.unwrap_err(),
            ConstructionDenial::DuplicateEquivalenceIdentity([1; 32])
        );
    }

    #[test]
    fn installed_predicate_is_found_only_under_its_declared_id() {
        let registry = EquivalenceRegistry::new([predicate(1, 1)]).unwrap();
        let installed = registry.get(EquivalenceContractId::new(1)).unwrap();
        assert_eq!(installed.identity(), [1; 32]);
        assert!(installed.equivalent(b"same", b"same"));
        assert!(!installed.equivalent(b"left", b"right"));
        assert!(registry.get(EquivalenceContractId::new(2)).is_none());
    }
}
