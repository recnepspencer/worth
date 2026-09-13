mod retained_charge;

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::data::aspect::AspectMask;
use crate::data::comparator::VersionComparatorPolicy;
use crate::data::output::{scopes_overlap, PartitionSubscription};
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::data::performance::{
    ArtifactPolicyClass, AuthorityPolicy, CompileTimePerformanceContract, EquivalenceContract,
    MaintenanceMode, PathClass,
};
use crate::data::reuse::{ArtifactEquivalenceContract, NodeReuseContract};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContextRequirement {
    #[default]
    None,
    DomainContext,
    RelationalSnapshot,
}

impl fmt::Display for ContextRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::DomainContext => write!(f, "DomainContext"),
            Self::RelationalSnapshot => write!(f, "RelationalSnapshot"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSemanticContract {
    pub reads: AspectMask,
    pub produces: AspectMask,
    #[serde(default)]
    pub partition_scope: Option<Vec<PartitionSubscription>>,
    #[serde(default)]
    pub required_context: ContextRequirement,
}

impl NodeSemanticContract {
    pub fn wildcard() -> Self {
        Self {
            reads: AspectMask::ALL,
            produces: AspectMask::ALL,
            partition_scope: None,
            required_context: ContextRequirement::None,
        }
    }
}

impl Default for NodeSemanticContract {
    fn default() -> Self {
        Self::wildcard()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeProjectionContract {
    pub consumes: AspectMask,
    #[serde(default)]
    pub consumes_partitions: Option<Vec<PartitionSubscription>>,
}

impl NodeProjectionContract {
    pub fn wildcard() -> Self {
        Self {
            consumes: AspectMask::ALL,
            consumes_partitions: None,
        }
    }
}

impl Default for NodeProjectionContract {
    fn default() -> Self {
        Self::wildcard()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeExecutionContract {
    #[serde(default)]
    pub equivalence: EquivalenceContract,
    #[serde(default)]
    pub path_class: PathClass,
    #[serde(default)]
    pub maintenance_mode: MaintenanceMode,
    #[serde(default)]
    pub artifact_policy: ArtifactPolicyClass,
}

impl NodeExecutionContract {
    pub fn operational() -> Self {
        Self {
            equivalence: EquivalenceContract::default(),
            path_class: PathClass::Operational,
            maintenance_mode: MaintenanceMode::DensityAdaptive,
            artifact_policy: ArtifactPolicyClass::OperationalMinimal,
        }
    }
}

impl Default for NodeExecutionContract {
    fn default() -> Self {
        Self::operational()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAuthorityContract {
    #[serde(default)]
    pub policy: AuthorityPolicy,
}

impl NodeAuthorityContract {
    pub fn default_speculative() -> Self {
        Self {
            policy: AuthorityPolicy::SpeculativeThenReconcile,
        }
    }
}

impl Default for NodeAuthorityContract {
    fn default() -> Self {
        Self::default_speculative()
    }
}

/// Declarative contract for one node's evaluation semantics and execution posture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeContract {
    #[serde(default)]
    pub semantics: NodeSemanticContract,
    #[serde(default)]
    pub projection: NodeProjectionContract,
    #[serde(default)]
    pub execution: NodeExecutionContract,
    #[serde(default)]
    pub authority: NodeAuthorityContract,
    #[serde(default)]
    pub reuse: NodeReuseContract,
}

impl NodeContract {
    pub fn wildcard() -> Self {
        Self {
            semantics: NodeSemanticContract::wildcard(),
            projection: NodeProjectionContract::wildcard(),
            execution: NodeExecutionContract::operational(),
            authority: NodeAuthorityContract::default_speculative(),
            reuse: NodeReuseContract::strict(),
        }
    }

    pub fn reads(reads: impl Into<AspectMask>) -> Self {
        Self::wildcard().with_reads(reads)
    }

    pub fn with_reads(mut self, reads: impl Into<AspectMask>) -> Self {
        let reads = reads.into();
        self.semantics.reads = reads;
        self.projection.consumes = reads;
        self
    }

    pub fn with_produces(mut self, produces: impl Into<AspectMask>) -> Self {
        self.semantics.produces = produces.into();
        self
    }

    pub fn with_partition_scope(
        mut self,
        partition_scope: impl Into<PartitionSubscription>,
    ) -> Self {
        let partition_scope = vec![partition_scope.into()];
        self.semantics.partition_scope = Some(partition_scope.clone());
        self.projection.consumes_partitions = Some(partition_scope);
        self
    }

    pub fn with_partition_scopes(
        mut self,
        partition_scopes: impl IntoIterator<Item = PartitionSubscription>,
    ) -> Self {
        let partition_scopes = partition_scopes.into_iter().collect::<Vec<_>>();
        self.semantics.partition_scope = Some(partition_scopes.clone());
        self.projection.consumes_partitions = Some(partition_scopes);
        self
    }

    pub fn with_required_context(mut self, required_context: ContextRequirement) -> Self {
        self.semantics.required_context = required_context;
        self
    }

    pub fn with_projection_contract(mut self, projection: NodeProjectionContract) -> Self {
        self.projection = projection;
        self
    }

    pub fn with_equivalence(mut self, equivalence: EquivalenceContract) -> Self {
        self.execution.equivalence = equivalence;
        self
    }

    pub fn with_path_class(mut self, path_class: PathClass) -> Self {
        self.execution.path_class = path_class;
        self
    }

    pub fn with_maintenance_mode(mut self, maintenance_mode: MaintenanceMode) -> Self {
        self.execution.maintenance_mode = maintenance_mode;
        self
    }

    pub fn with_artifact_policy(mut self, artifact_policy: ArtifactPolicyClass) -> Self {
        self.execution.artifact_policy = artifact_policy;
        self
    }

    pub fn with_authority_policy(mut self, authority_policy: AuthorityPolicy) -> Self {
        self.authority.policy = authority_policy;
        self
    }

    pub fn with_reuse_contract(mut self, reuse: NodeReuseContract) -> Self {
        self.reuse = reuse;
        self
    }

    pub fn with_artifact_equivalence_contract(
        mut self,
        equivalence: ArtifactEquivalenceContract,
    ) -> Self {
        self.reuse.equivalence = equivalence;
        self
    }

    pub fn with_cross_identity_persistent_matching(mut self) -> Self {
        self.reuse.equivalence = self
            .reuse
            .equivalence
            .with_cross_identity_persistent_matching();
        self
    }

    pub fn with_partial_artifact_splicing(mut self) -> Self {
        self.reuse.equivalence = self.reuse.equivalence.with_partial_artifact_splicing();
        self
    }

    pub fn with_reuse_certification_retention(mut self, retain_certification: bool) -> Self {
        self.reuse.retain_certification = retain_certification;
        self
    }

    pub fn with_comparator_override(mut self, comparator: &VersionComparatorPolicy) -> Self {
        self.execution.equivalence = EquivalenceContract::for_comparator_override(comparator);
        self
    }

    pub fn with_output_equivalence(mut self, policy: &OutputEquivalencePolicy) -> Self {
        self.execution.equivalence = EquivalenceContract::for_output_equivalence(policy);
        self
    }

    pub fn compile_time_performance_contract(&self) -> CompileTimePerformanceContract {
        CompileTimePerformanceContract {
            equivalence: self.execution.equivalence,
            path_class: self.execution.path_class,
            maintenance_mode: self.execution.maintenance_mode,
            artifact_policy: self.execution.artifact_policy,
            authority_policy: self.authority.policy,
        }
    }

    pub fn reads_dirty_aspects(&self, dirty_aspects: AspectMask) -> bool {
        self.projection.consumes.intersects(dirty_aspects)
    }

    pub fn cares_about_change(
        &self,
        changed_aspects: AspectMask,
        changed_scopes: &[PartitionSubscription],
    ) -> bool {
        if !self.projection.consumes.intersects(changed_aspects) {
            return false;
        }
        match &self.projection.consumes_partitions {
            None => true,
            Some(contract_scopes) if changed_scopes.is_empty() => true,
            Some(contract_scopes) => contract_scopes.iter().any(|contract_scope| {
                changed_scopes
                    .iter()
                    .any(|changed_scope| scopes_overlap(contract_scope, changed_scope))
            }),
        }
    }

    pub(crate) fn cares_about_correlated_change(
        &self,
        changed_aspects: AspectMask,
        changed_scoped_aspects: &[(crate::data::aspect::Aspect, PartitionSubscription)],
    ) -> bool {
        for index in 0..crate::data::aspect::MAX_ASPECTS {
            let aspect = crate::data::aspect::Aspect::new(index as u8);
            let aspect_mask = AspectMask::from_aspect(aspect);
            if !changed_aspects.intersects(aspect_mask)
                || !self.projection.consumes.intersects(aspect_mask)
            {
                continue;
            }
            let scopes = changed_scoped_aspects
                .iter()
                .filter(|(candidate, _)| *candidate == aspect)
                .map(|(_, scope)| scope)
                .collect::<Vec<_>>();
            if scopes.is_empty() || self.projection.consumes_partitions.is_none() {
                return true;
            }
            if self
                .projection
                .consumes_partitions
                .as_ref()
                .is_some_and(|contracts| {
                    contracts.iter().any(|contract| {
                        scopes
                            .iter()
                            .any(|changed| scopes_overlap(contract, *changed))
                    })
                })
            {
                return true;
            }
        }
        false
    }
}

impl Default for NodeContract {
    fn default() -> Self {
        Self::wildcard()
    }
}
