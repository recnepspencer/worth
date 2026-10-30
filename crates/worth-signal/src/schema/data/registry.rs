mod retained_charge;

use std::collections::BTreeMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{SignalSchemaDescriptor, SignalSchemaId, SignalSchemaName, SignalSchemaRegistration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateSignalSchemaRegistration {
    Id {
        id: SignalSchemaId,
    },
    SemanticName {
        name: SignalSchemaName,
    },
    SemanticNameAndVersion {
        name: SignalSchemaName,
        version: super::SignalSchemaVersion,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignalSchemaRegistry {
    registrations: Arc<[SignalSchemaRegistration]>,
    index_by_id: Arc<BTreeMap<SignalSchemaId, usize>>,
    index_by_name: Arc<BTreeMap<SignalSchemaName, usize>>,
    registry_digest: String,
}

impl SignalSchemaRegistry {
    pub fn from_registrations(
        registrations: Vec<SignalSchemaRegistration>,
    ) -> Result<Self, DuplicateSignalSchemaRegistration> {
        let mut ids = BTreeMap::new();
        let mut names = BTreeMap::new();
        let mut semantic_versions = BTreeMap::new();
        let mut digest_basis = Vec::with_capacity(registrations.len());

        for (index, registration) in registrations.iter().enumerate() {
            let descriptor = registration.descriptor();
            if ids.insert(descriptor.id(), index).is_some() {
                return Err(DuplicateSignalSchemaRegistration::Id {
                    id: descriptor.id(),
                });
            }
            if names
                .insert(descriptor.semantic_name().clone(), index)
                .is_some()
            {
                return Err(DuplicateSignalSchemaRegistration::SemanticName {
                    name: descriptor.semantic_name().clone(),
                });
            }
            if semantic_versions
                .insert(
                    (descriptor.semantic_name().clone(), descriptor.version()),
                    (),
                )
                .is_some()
            {
                return Err(DuplicateSignalSchemaRegistration::SemanticNameAndVersion {
                    name: descriptor.semantic_name().clone(),
                    version: descriptor.version(),
                });
            }
            digest_basis.push(descriptor.clone());
        }

        Ok(Self {
            registrations: registrations.into(),
            index_by_id: Arc::new(ids),
            index_by_name: Arc::new(names),
            registry_digest: registry_digest(&digest_basis),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SignalSchemaRegistration> {
        self.registrations.iter()
    }

    pub fn get_by_id(&self, schema_id: SignalSchemaId) -> Option<&SignalSchemaRegistration> {
        self.index_by_id
            .get(&schema_id)
            .and_then(|index| self.registrations.get(*index))
    }

    pub fn get_by_name(
        &self,
        semantic_name: &SignalSchemaName,
    ) -> Option<&SignalSchemaRegistration> {
        self.index_by_name
            .get(semantic_name)
            .and_then(|index| self.registrations.get(*index))
    }

    pub fn resolve_by_id(&self, schema_id: SignalSchemaId) -> Option<&SignalSchemaDescriptor> {
        self.get_by_id(schema_id)
            .map(SignalSchemaRegistration::descriptor)
    }

    pub fn resolve_by_name(
        &self,
        semantic_name: &SignalSchemaName,
    ) -> Option<&SignalSchemaDescriptor> {
        self.get_by_name(semantic_name)
            .map(SignalSchemaRegistration::descriptor)
    }

    pub fn registry_digest(&self) -> &str {
        &self.registry_digest
    }
}

fn registry_digest(descriptors: &[SignalSchemaDescriptor]) -> String {
    let mut canonical_descriptors = descriptors.to_vec();
    canonical_descriptors.sort_by(|left, right| {
        left.semantic_name()
            .cmp(right.semantic_name())
            .then_with(|| left.version().cmp(&right.version()))
            .then_with(|| left.id().cmp(&right.id()))
    });
    let bytes =
        serde_json::to_vec(&canonical_descriptors).expect("signal schema registry serialization");
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::{DuplicateSignalSchemaRegistration, SignalSchemaRegistry};
    use crate::data::node::NodeContract;
    use crate::schema::data::{
        SignalSchemaDescriptor, SignalSchemaId, SignalSchemaName, SignalSchemaRegistration,
        SignalSchemaVersion,
    };

    fn registration(
        id: u32,
        semantic_name: &str,
        version: SignalSchemaVersion,
    ) -> SignalSchemaRegistration {
        SignalSchemaRegistration::new(SignalSchemaDescriptor::new(
            SignalSchemaId(id),
            SignalSchemaName::new(semantic_name),
            version,
            NodeContract::wildcard(),
        ))
        .expect("valid schema registration")
    }

    #[test]
    fn frozen_registry_rejects_duplicate_id() {
        let left = registration(7, "schema.left", SignalSchemaVersion::new(1, 0));
        let right = registration(7, "schema.right", SignalSchemaVersion::new(1, 0));

        let error = SignalSchemaRegistry::from_registrations(vec![left, right]).unwrap_err();
        assert_eq!(
            error,
            DuplicateSignalSchemaRegistration::Id {
                id: SignalSchemaId(7)
            }
        );
    }

    #[test]
    fn frozen_registry_rejects_duplicate_semantic_name() {
        let left = registration(7, "schema.same", SignalSchemaVersion::new(1, 0));
        let right = registration(8, "schema.same", SignalSchemaVersion::new(1, 1));

        let error = SignalSchemaRegistry::from_registrations(vec![left, right]).unwrap_err();
        assert_eq!(
            error,
            DuplicateSignalSchemaRegistration::SemanticName {
                name: SignalSchemaName::new("schema.same")
            }
        );
    }

    #[test]
    fn frozen_registry_exposes_stable_digest_and_iteration() {
        let registry = SignalSchemaRegistry::from_registrations(vec![
            registration(7, "schema.left", SignalSchemaVersion::new(1, 0)),
            registration(8, "schema.right", SignalSchemaVersion::new(1, 0)),
        ])
        .expect("valid registry");

        assert_eq!(registry.len(), 2);
        assert!(!registry.registry_digest().is_empty());
        assert_eq!(registry.iter().count(), 2);
    }

    #[test]
    fn frozen_registry_digest_is_order_independent() {
        let left_first = SignalSchemaRegistry::from_registrations(vec![
            registration(7, "schema.left", SignalSchemaVersion::new(1, 0)),
            registration(8, "schema.right", SignalSchemaVersion::new(1, 1)),
        ])
        .expect("valid registry");
        let right_first = SignalSchemaRegistry::from_registrations(vec![
            registration(8, "schema.right", SignalSchemaVersion::new(1, 1)),
            registration(7, "schema.left", SignalSchemaVersion::new(1, 0)),
        ])
        .expect("valid registry");

        assert_eq!(left_first.registry_digest(), right_first.registry_digest());
    }
}
