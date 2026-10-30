use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, MergeOntologyMappingIdentityTag};

use super::taxonomy::{
    BridgeMergeConsumptionClass, BridgeMergeOntologyLoweringKind, CanonicalRelationalMergeClass,
};

pub type BridgeMergeOntologyMappingSurfaceIdentity =
    BridgeIdentity<MergeOntologyMappingIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMergeOntologyMappingEntry {
    canonical_relational_class: CanonicalRelationalMergeClass,
    bridge_class: BridgeMergeConsumptionClass,
    lowering_kind: BridgeMergeOntologyLoweringKind,
    canonical_basis: Arc<str>,
}

impl BridgeMergeOntologyMappingEntry {
    pub fn direct_wrapper(
        canonical_relational_class: CanonicalRelationalMergeClass,
        bridge_class: BridgeMergeConsumptionClass,
    ) -> Self {
        Self {
            canonical_relational_class,
            bridge_class,
            lowering_kind: BridgeMergeOntologyLoweringKind::DirectWrapper,
            canonical_basis: Arc::from(format!(
                "merge-ontology-entry|canonical:{canonical_relational_class:?}|bridge:{bridge_class:?}|lowering:{:?}",
                BridgeMergeOntologyLoweringKind::DirectWrapper
            )),
        }
    }

    pub fn canonical_relational_class(&self) -> CanonicalRelationalMergeClass {
        self.canonical_relational_class
    }

    pub fn bridge_class(&self) -> BridgeMergeConsumptionClass {
        self.bridge_class
    }

    pub fn lowering_kind(&self) -> BridgeMergeOntologyLoweringKind {
        self.lowering_kind
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMergeOntologyMappingSurface {
    mapping_identity: BridgeMergeOntologyMappingSurfaceIdentity,
    ontology_version: Arc<str>,
    entries: Arc<[BridgeMergeOntologyMappingEntry]>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeMergeOntologyMappingSurface {
    pub fn new(
        ontology_version: impl Into<Arc<str>>,
        mut entries: Vec<BridgeMergeOntologyMappingEntry>,
    ) -> Self {
        entries.sort_by(|left, right| {
            left.canonical_relational_class()
                .cmp(&right.canonical_relational_class())
                .then_with(|| left.bridge_class().cmp(&right.bridge_class()))
                .then_with(|| left.lowering_kind().cmp(&right.lowering_kind()))
        });
        let ontology_version = ontology_version.into();
        let canonical_basis = Arc::<str>::from(format!(
            "merge-ontology-mapping-surface|version={}|entries={}",
            ontology_version.as_ref(),
            entries
                .iter()
                .map(BridgeMergeOntologyMappingEntry::canonical_basis)
                .collect::<Vec<_>>()
                .join(","),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            mapping_identity: BridgeMergeOntologyMappingSurfaceIdentity::admit_bridge_owned(
                format!("merge-ontology-mapping-surface:sha256:{digest:x}"),
            ),
            ontology_version,
            entries: Arc::from(entries),
            canonical_basis,
            digest: Arc::from(format!("merge-ontology-mapping-surface:sha256:{digest:x}")),
        }
    }

    pub fn direct_phase_m9_0(ontology_version: impl Into<Arc<str>>) -> Self {
        Self::new(
            ontology_version,
            vec![
                BridgeMergeOntologyMappingEntry::direct_wrapper(
                    CanonicalRelationalMergeClass::AspectReconciliation,
                    BridgeMergeConsumptionClass::AspectReconciliationMerge,
                ),
                BridgeMergeOntologyMappingEntry::direct_wrapper(
                    CanonicalRelationalMergeClass::Deletion,
                    BridgeMergeConsumptionClass::DeletionMerge,
                ),
                BridgeMergeOntologyMappingEntry::direct_wrapper(
                    CanonicalRelationalMergeClass::TopologyRewire,
                    BridgeMergeConsumptionClass::TopologyRewireMerge,
                ),
                BridgeMergeOntologyMappingEntry::direct_wrapper(
                    CanonicalRelationalMergeClass::PolicyResolvedConflict,
                    BridgeMergeConsumptionClass::PolicyResolvedConflictMerge,
                ),
            ],
        )
    }

    pub fn mapping_identity(&self) -> &BridgeMergeOntologyMappingSurfaceIdentity {
        &self.mapping_identity
    }

    pub fn ontology_version(&self) -> &str {
        self.ontology_version.as_ref()
    }

    pub fn entries(&self) -> &[BridgeMergeOntologyMappingEntry] {
        &self.entries
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::BridgeMergeOntologyMappingSurface;

    #[test]
    fn merge_ontology_mapping_surface_is_canonical_for_same_inputs() {
        let left = BridgeMergeOntologyMappingSurface::direct_phase_m9_0("rel-merge-v1");
        let right = BridgeMergeOntologyMappingSurface::direct_phase_m9_0("rel-merge-v1");

        assert_eq!(left, right);
        assert_eq!(left.entries().len(), 4);
    }
}
