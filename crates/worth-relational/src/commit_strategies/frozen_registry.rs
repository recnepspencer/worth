use std::collections::BTreeMap;
use std::sync::Arc;

use crate::commit_strategies::data::{
    commit_strategy_registry_digest, CommitStrategyFamilyName, CommitStrategyRegistration,
    CommitStrategySemanticName, CommitStrategyVersion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateCommitStrategyRegistration {
    Id {
        id: crate::commit_strategies::data::CommitStrategyId,
    },
    SemanticName {
        name: CommitStrategySemanticName,
    },
    FamilyAndVersion {
        family_name: CommitStrategyFamilyName,
        version: CommitStrategyVersion,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrozenCommitStrategyRegistry {
    registrations: Arc<[CommitStrategyRegistration]>,
    index_by_id: Arc<BTreeMap<crate::commit_strategies::data::CommitStrategyId, usize>>,
    index_by_name: Arc<BTreeMap<CommitStrategySemanticName, usize>>,
    registry_digest: String,
}

impl FrozenCommitStrategyRegistry {
    pub(crate) fn from_registrations(
        registrations: Vec<CommitStrategyRegistration>,
    ) -> Result<Self, DuplicateCommitStrategyRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut family_versions = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());

        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateCommitStrategyRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateCommitStrategyRegistration::SemanticName {
                    name: descriptor.semantic_name().clone(),
                });
            }
            if family_versions
                .insert((descriptor.family_name().clone(), descriptor.version()), ())
                .is_some()
            {
                return Err(DuplicateCommitStrategyRegistration::FamilyAndVersion {
                    family_name: descriptor.family_name().clone(),
                    version: descriptor.version(),
                });
            }
            digest_basis.push(descriptor.clone());
        }

        Ok(Self {
            registrations: registrations.into(),
            index_by_id: Arc::new(ids),
            index_by_name: Arc::new(names),
            registry_digest: commit_strategy_registry_digest(&digest_basis),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommitStrategyRegistration> {
        self.registrations.iter()
    }

    pub fn get_by_id(
        &self,
        strategy_id: crate::commit_strategies::data::CommitStrategyId,
    ) -> Option<&CommitStrategyRegistration> {
        self.index_by_id
            .get(&strategy_id)
            .and_then(|index| self.registrations.get(*index))
    }

    pub fn get_by_name(
        &self,
        semantic_name: &crate::commit_strategies::data::CommitStrategySemanticName,
    ) -> Option<&CommitStrategyRegistration> {
        self.index_by_name
            .get(semantic_name)
            .and_then(|index| self.registrations.get(*index))
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

#[cfg(test)]
mod tests {
    use super::{DuplicateCommitStrategyRegistration, FrozenCommitStrategyRegistry};
    use crate::commit_strategies::data::{
        CommitStrategyDescriptor, CommitStrategyFamilyName, CommitStrategyId,
        CommitStrategyRegistration, CommitStrategySemanticName, CommitStrategyVersion,
        PersistentArtifactName, StrategyInputSchemaName, StrategyInputSchemaVersion,
        StrategyIntentName, StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract,
        StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
        StrategyTraversalBasis,
    };

    fn registration(
        id: u32,
        semantic_name: &str,
        family_name: &str,
        version: CommitStrategyVersion,
    ) -> CommitStrategyRegistration {
        CommitStrategyRegistration::new(CommitStrategyDescriptor::new(
            CommitStrategyId(id),
            CommitStrategySemanticName::new(semantic_name),
            CommitStrategyFamilyName::new(family_name),
            version,
            StrategyIntentName::new("replica.converge"),
            StrategyInputSchemaName::new("replica.input.v1"),
            StrategyInputSchemaVersion(1),
            StrategyOutputSchemaName::new("replica.output.v1"),
            StrategyReadContract {
                scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
                locality_class: StrategyReadLocalityClass::SinglePartition,
                traversal_basis: StrategyTraversalBasis::NoTraversal,
                packet_contract: StrategyPacketContract::ProjectionOnly,
                cost_class: StrategyReadCostClass::ORequestedSurface,
            },
            PersistentArtifactName::new("replica.convergence"),
        ))
        .expect("valid registration")
    }

    #[test]
    fn frozen_registry_rejects_duplicate_id() {
        let left = registration(
            7,
            "strategy.left",
            "strategy.family",
            CommitStrategyVersion::new(1, 0),
        );
        let right = registration(
            7,
            "strategy.right",
            "strategy.family.2",
            CommitStrategyVersion::new(1, 0),
        );

        let error =
            FrozenCommitStrategyRegistry::from_registrations(vec![left, right]).unwrap_err();
        assert_eq!(
            error,
            DuplicateCommitStrategyRegistration::Id {
                id: CommitStrategyId(7)
            }
        );
    }

    #[test]
    fn frozen_registry_rejects_duplicate_semantic_name() {
        let left = registration(
            7,
            "strategy.same",
            "strategy.family",
            CommitStrategyVersion::new(1, 0),
        );
        let right = registration(
            8,
            "strategy.same",
            "strategy.family.2",
            CommitStrategyVersion::new(1, 0),
        );

        let error =
            FrozenCommitStrategyRegistry::from_registrations(vec![left, right]).unwrap_err();
        assert_eq!(
            error,
            DuplicateCommitStrategyRegistration::SemanticName {
                name: CommitStrategySemanticName::new("strategy.same")
            }
        );
    }

    #[test]
    fn frozen_registry_rejects_duplicate_family_and_version() {
        let left = registration(
            7,
            "strategy.left",
            "strategy.family",
            CommitStrategyVersion::new(1, 0),
        );
        let right = registration(
            8,
            "strategy.right",
            "strategy.family",
            CommitStrategyVersion::new(1, 0),
        );

        let error =
            FrozenCommitStrategyRegistry::from_registrations(vec![left, right]).unwrap_err();
        assert_eq!(
            error,
            DuplicateCommitStrategyRegistration::FamilyAndVersion {
                family_name: CommitStrategyFamilyName::new("strategy.family"),
                version: CommitStrategyVersion::new(1, 0),
            }
        );
    }

    #[test]
    fn frozen_registry_exposes_canonical_digest_and_iteration() {
        let registry = FrozenCommitStrategyRegistry::from_registrations(vec![
            registration(
                7,
                "strategy.left",
                "strategy.family",
                CommitStrategyVersion::new(1, 0),
            ),
            registration(
                8,
                "strategy.right",
                "strategy.family",
                CommitStrategyVersion::new(1, 1),
            ),
        ])
        .expect("valid registry");

        assert_eq!(registry.len(), 2);
        assert!(!registry.registry_digest().is_empty());
        assert_eq!(registry.iter().count(), 2);
    }

    #[test]
    fn frozen_registry_digest_is_order_independent() {
        let left_first = FrozenCommitStrategyRegistry::from_registrations(vec![
            registration(
                7,
                "strategy.left",
                "strategy.family",
                CommitStrategyVersion::new(1, 0),
            ),
            registration(
                8,
                "strategy.right",
                "strategy.family",
                CommitStrategyVersion::new(1, 1),
            ),
        ])
        .expect("valid registry");
        let right_first = FrozenCommitStrategyRegistry::from_registrations(vec![
            registration(
                8,
                "strategy.right",
                "strategy.family",
                CommitStrategyVersion::new(1, 1),
            ),
            registration(
                7,
                "strategy.left",
                "strategy.family",
                CommitStrategyVersion::new(1, 0),
            ),
        ])
        .expect("valid registry");

        assert_eq!(left_first.registry_digest(), right_first.registry_digest());
    }
}
