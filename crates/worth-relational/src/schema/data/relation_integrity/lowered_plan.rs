use serde::{Deserialize, Serialize};

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::schema::data::{
    LoweredAcyclicityContract, LoweredConnectivityMinimumContract,
    LoweredPartitionIsolationContract,
};

use super::{
    ContractId, MinimumCardinalityEnforcement, PairMinimumSemantics, RelationIntegrityPlanRevision,
    SymmetryMode, UniquenessScope,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredRelationIntegrityPlan {
    pub aggregate_identity: Option<super::AggregateIdentityContractDeclaration>,
    pub kind_id: KindId,
    pub plan_revision: RelationIntegrityPlanRevision,
    pub endpoint_kind_contracts: Vec<LoweredEndpointKindContract>,
    pub cardinality_maximum_contracts: Vec<LoweredCardinalityMaximumContract>,
    pub cardinality_minimum_contracts: Vec<LoweredCardinalityMinimumContract>,
    pub uniqueness_contracts: Vec<LoweredUniquenessContract>,
    pub symmetry_contracts: Vec<LoweredSymmetryContract>,
    pub endpoint_deletion_integrity_contracts: Vec<LoweredEndpointDeletionIntegrityContract>,
    pub acyclicity_contracts: Vec<LoweredAcyclicityContract>,
    pub partition_isolation_contracts: Vec<LoweredPartitionIsolationContract>,
    pub connectivity_minimum_contracts: Vec<LoweredConnectivityMinimumContract>,
}

impl LoweredRelationIntegrityPlan {
    pub fn contract_count(&self) -> usize {
        usize::from(self.aggregate_identity.is_some())
            + self.endpoint_kind_contracts.len()
            + self.cardinality_maximum_contracts.len()
            + self.cardinality_minimum_contracts.len()
            + self.uniqueness_contracts.len()
            + self.symmetry_contracts.len()
            + self.endpoint_deletion_integrity_contracts.len()
            + self.acyclicity_contracts.len()
            + self.partition_isolation_contracts.len()
            + self.connectivity_minimum_contracts.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredEndpointKindContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub allowed_source_kinds: Vec<KindId>,
    pub allowed_target_kinds: Vec<KindId>,
    pub self_edges_allowed: bool,
    pub cross_context_policy: CrossContextPolicy,
    pub plan_revision: RelationIntegrityPlanRevision,
}

impl LoweredEndpointKindContract {
    pub fn allows_source_kind(&self, kind_id: KindId) -> bool {
        self.allowed_source_kinds.binary_search(&kind_id).is_ok()
    }

    pub fn allows_target_kind(&self, kind_id: KindId) -> bool {
        self.allowed_target_kinds.binary_search(&kind_id).is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredCardinalityMaximumContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub source_max: Option<u64>,
    pub target_max: Option<u64>,
    pub pair_max: Option<u64>,
    pub plan_revision: RelationIntegrityPlanRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredCardinalityMinimumContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub source_min: Option<u64>,
    pub target_min: Option<u64>,
    pub pair_min: Option<u64>,
    pub pair_min_semantics: PairMinimumSemantics,
    pub candidate_source_kinds: Vec<KindId>,
    pub candidate_target_kinds: Vec<KindId>,
    pub minimum_enforcement: MinimumCardinalityEnforcement,
    pub plan_revision: RelationIntegrityPlanRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredUniquenessContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub scope: UniquenessScope,
    pub plan_revision: RelationIntegrityPlanRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredSymmetryContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub mode: SymmetryMode,
    pub plan_revision: RelationIntegrityPlanRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredEndpointDeletionIntegrityContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub mode: super::EndpointDeletionIntegrityMode,
    pub cascade_delete_policy: CascadeDeletePolicy,
    pub plan_revision: RelationIntegrityPlanRevision,
}
