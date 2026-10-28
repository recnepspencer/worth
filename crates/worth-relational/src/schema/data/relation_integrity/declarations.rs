use serde::{Deserialize, Serialize};

use crate::config::data::CrossContextPolicy;
use crate::schema::data::{
    AcyclicityContractDeclaration, ConnectivityMinimumContractDeclaration,
    PartitionIsolationContractDeclaration,
};

use super::{derive_relation_integrity_plan_revision, ContractId, RelationIntegrityPlanRevision};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RelationIntegrityDeclarations {
    pub plan_revision: RelationIntegrityPlanRevision,
    pub endpoint_kind_contracts: Vec<EndpointKindContractDeclaration>,
    pub cardinality_contracts: Vec<CardinalityContractDeclaration>,
    pub uniqueness_contracts: Vec<UniquenessContractDeclaration>,
    pub symmetry_contracts: Vec<SymmetryContractDeclaration>,
    pub endpoint_deletion_integrity_contracts: Vec<EndpointDeletionIntegrityDeclaration>,
    pub acyclicity_contracts: Vec<AcyclicityContractDeclaration>,
    pub partition_isolation_contracts: Vec<PartitionIsolationContractDeclaration>,
    pub connectivity_minimum_contracts: Vec<ConnectivityMinimumContractDeclaration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggregate_identity: Option<super::AggregateIdentityContractDeclaration>,
}

impl RelationIntegrityDeclarations {
    pub fn new(
        endpoint_kind_contracts: Vec<EndpointKindContractDeclaration>,
        cardinality_contracts: Vec<CardinalityContractDeclaration>,
        uniqueness_contracts: Vec<UniquenessContractDeclaration>,
        symmetry_contracts: Vec<SymmetryContractDeclaration>,
        endpoint_deletion_integrity_contracts: Vec<EndpointDeletionIntegrityDeclaration>,
    ) -> Self {
        Self {
            aggregate_identity: None,
            plan_revision: derive_relation_integrity_plan_revision(
                &endpoint_kind_contracts,
                &cardinality_contracts,
                &uniqueness_contracts,
                &symmetry_contracts,
                &endpoint_deletion_integrity_contracts,
                &[],
                &[],
                &[],
                None,
            ),
            endpoint_kind_contracts,
            cardinality_contracts,
            uniqueness_contracts,
            symmetry_contracts,
            endpoint_deletion_integrity_contracts,
            acyclicity_contracts: Vec::new(),
            partition_isolation_contracts: Vec::new(),
            connectivity_minimum_contracts: Vec::new(),
        }
    }

    pub fn contract_count(&self) -> usize {
        usize::from(self.aggregate_identity.is_some())
            + self.endpoint_kind_contracts.len()
            + self.cardinality_contracts.len()
            + self.uniqueness_contracts.len()
            + self.symmetry_contracts.len()
            + self.endpoint_deletion_integrity_contracts.len()
            + self.acyclicity_contracts.len()
            + self.partition_isolation_contracts.len()
            + self.connectivity_minimum_contracts.len()
    }

    pub fn with_acyclicity_contracts(
        mut self,
        acyclicity_contracts: Vec<AcyclicityContractDeclaration>,
    ) -> Self {
        self.acyclicity_contracts = acyclicity_contracts;
        self.recompute_plan_revision();
        self
    }

    pub fn with_partition_isolation_contracts(
        mut self,
        partition_isolation_contracts: Vec<PartitionIsolationContractDeclaration>,
    ) -> Self {
        self.partition_isolation_contracts = partition_isolation_contracts;
        self.recompute_plan_revision();
        self
    }

    pub fn with_connectivity_minimum_contracts(
        mut self,
        connectivity_minimum_contracts: Vec<ConnectivityMinimumContractDeclaration>,
    ) -> Self {
        self.connectivity_minimum_contracts = connectivity_minimum_contracts;
        self.recompute_plan_revision();
        self
    }

    fn recompute_plan_revision(&mut self) {
        self.plan_revision = derive_relation_integrity_plan_revision(
            &self.endpoint_kind_contracts,
            &self.cardinality_contracts,
            &self.uniqueness_contracts,
            &self.symmetry_contracts,
            &self.endpoint_deletion_integrity_contracts,
            &self.acyclicity_contracts,
            &self.partition_isolation_contracts,
            &self.connectivity_minimum_contracts,
            self.aggregate_identity.as_ref(),
        );
    }

    pub fn with_aggregate_identity(
        mut self,
        contract: super::AggregateIdentityContractDeclaration,
    ) -> Self {
        self.aggregate_identity = Some(contract);
        self.recompute_plan_revision();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointKindContractDeclaration {
    pub contract_id: ContractId,
    pub allowed_source_kinds: Vec<crate::identity::data::KindId>,
    pub allowed_target_kinds: Vec<crate::identity::data::KindId>,
    pub self_edges_allowed: bool,
    pub cross_context_policy: CrossContextPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardinalityContractDeclaration {
    pub contract_id: ContractId,
    pub source_max: Option<u64>,
    pub target_max: Option<u64>,
    pub pair_max: Option<u64>,
    pub source_min: Option<u64>,
    pub target_min: Option<u64>,
    pub pair_min: Option<u64>,
    pub pair_min_semantics: PairMinimumSemantics,
    pub minimum_enforcement: MinimumCardinalityEnforcement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PairMinimumSemantics {
    ObservedDirectedPairs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MinimumCardinalityEnforcement {
    CommitBoundary,
    CertificationBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UniquenessScope {
    DirectedSemanticEdge,
    NormalizedSymmetricEdge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UniquenessContractDeclaration {
    pub contract_id: ContractId,
    pub scope: UniquenessScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SymmetryMode {
    CanonicalUndirected,
    PairedInverseRequired,
    InverseProhibited,
    PairedTwinRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymmetryContractDeclaration {
    pub contract_id: ContractId,
    pub mode: SymmetryMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EndpointDeletionIntegrityMode {
    RejectDeleteWithLiveRelations,
    RequireRelationDeletionInSameCommit,
    RequireRelationRetirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointDeletionIntegrityDeclaration {
    pub contract_id: ContractId,
    pub mode: EndpointDeletionIntegrityMode,
}
