mod retained_charge;

use serde::{Deserialize, Serialize};

use super::basis::ReuseStrategy;

/// Semantic boundaries that must remain equivalent for artifact reuse to be legal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArtifactSemanticBoundary {
    TopologyRegime,
    ToleranceRegime,
    SemanticRegionIdentity,
    AuthorityLane,
    SnapshotLineage,
    ArtifactFamilyBasis,
    StructuralDependencyBasis,
    PartitionRegionBasis,
    PersistentCorrespondence,
    CompositionRegionSet,
}

/// Declarative equivalence boundaries for artifact reuse legality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEquivalenceContract {
    #[serde(default = "ArtifactEquivalenceContract::default_boundaries")]
    pub required_boundaries: Vec<ArtifactSemanticBoundary>,
    #[serde(default = "ArtifactEquivalenceContract::default_supported_strategies")]
    pub supported_strategies: Vec<ReuseStrategy>,
    #[serde(default)]
    pub allows_snapshot_restore_reuse: bool,
    #[serde(default)]
    pub allows_authority_reconciliation_reuse: bool,
}

impl ArtifactEquivalenceContract {
    pub fn strict() -> Self {
        Self {
            required_boundaries: Self::default_boundaries(),
            supported_strategies: Self::default_supported_strategies(),
            allows_snapshot_restore_reuse: false,
            allows_authority_reconciliation_reuse: false,
        }
    }

    fn default_boundaries() -> Vec<ArtifactSemanticBoundary> {
        vec![
            ArtifactSemanticBoundary::TopologyRegime,
            ArtifactSemanticBoundary::ToleranceRegime,
            ArtifactSemanticBoundary::SemanticRegionIdentity,
            ArtifactSemanticBoundary::ArtifactFamilyBasis,
            ArtifactSemanticBoundary::StructuralDependencyBasis,
            ArtifactSemanticBoundary::PartitionRegionBasis,
        ]
    }

    fn default_supported_strategies() -> Vec<ReuseStrategy> {
        vec![
            ReuseStrategy::OutputSuppression,
            ReuseStrategy::MemoizedArtifactReuse,
            ReuseStrategy::SnapshotRestoreReuse,
            ReuseStrategy::ReconciliationAdoption,
            ReuseStrategy::CrossIdentityPersistentMatch,
            ReuseStrategy::PartialArtifactSplicing,
        ]
    }

    pub fn supports_strategy(&self, strategy: ReuseStrategy) -> bool {
        self.supported_strategies.contains(&strategy)
    }

    fn ensure_boundary(&mut self, boundary: ArtifactSemanticBoundary) {
        if !self.required_boundaries.contains(&boundary) {
            self.required_boundaries.push(boundary);
        }
    }

    fn ensure_strategy(&mut self, strategy: ReuseStrategy) {
        if !self.supported_strategies.contains(&strategy) {
            self.supported_strategies.push(strategy);
        }
    }

    pub fn with_cross_identity_persistent_matching(mut self) -> Self {
        self.ensure_strategy(ReuseStrategy::CrossIdentityPersistentMatch);
        self.ensure_boundary(ArtifactSemanticBoundary::PersistentCorrespondence);
        self
    }

    pub fn with_partial_artifact_splicing(mut self) -> Self {
        self.ensure_strategy(ReuseStrategy::PartialArtifactSplicing);
        self.ensure_boundary(ArtifactSemanticBoundary::CompositionRegionSet);
        self
    }
}

impl Default for ArtifactEquivalenceContract {
    fn default() -> Self {
        Self::strict()
    }
}

/// Node-level declarative posture for artifact reuse and certification retention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeReuseContract {
    #[serde(default)]
    pub equivalence: ArtifactEquivalenceContract,
    #[serde(default)]
    pub retain_certification: bool,
}

impl NodeReuseContract {
    pub fn strict() -> Self {
        Self {
            equivalence: ArtifactEquivalenceContract::strict(),
            retain_certification: true,
        }
    }
}

impl Default for NodeReuseContract {
    fn default() -> Self {
        Self::strict()
    }
}
