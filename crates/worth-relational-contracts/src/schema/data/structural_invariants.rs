use serde::{Deserialize, Serialize};

use crate::identity::data::KindId;

use super::{ContractId, RelationIntegrityPlanRevision};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DirectedTraversalKind {
    SourceToTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AllowedCycleClass {
    NoCycles,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcyclicityContractDeclaration {
    pub contract_id: ContractId,
    pub traversal_direction: DirectedTraversalKind,
    pub allowed_cycle_class: AllowedCycleClass,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredAcyclicityContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub traversal_direction: DirectedTraversalKind,
    pub allowed_cycle_class: AllowedCycleClass,
    pub plan_revision: RelationIntegrityPlanRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PartitionIsolationMode {
    SamePartitionEndpoints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionIsolationContractDeclaration {
    pub contract_id: ContractId,
    pub isolation_mode: PartitionIsolationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredPartitionIsolationContract {
    pub contract_id: ContractId,
    pub relation_kind_id: KindId,
    pub isolation_mode: PartitionIsolationMode,
    pub plan_revision: RelationIntegrityPlanRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConnectivityMinimumEnforcement {
    SnapshotPublication,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectivityMinimumContractDeclaration {
    pub contract_id: ContractId,
    pub source_kind_ids: Vec<KindId>,
    pub target_kind_ids: Vec<KindId>,
    pub minimum_reachable_targets: u32,
    pub enforcement_boundary: ConnectivityMinimumEnforcement,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LoweredConnectivityMinimumContract {
    pub contract_id: ContractId,
    pub source_kind_ids: Vec<KindId>,
    pub relation_kind_id: KindId,
    pub target_kind_ids: Vec<KindId>,
    pub minimum_reachable_targets: u32,
    pub enforcement_boundary: ConnectivityMinimumEnforcement,
    pub plan_revision: RelationIntegrityPlanRevision,
}
