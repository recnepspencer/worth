use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitStrategyVersion {
    pub major: u16,
    pub minor: u16,
}

impl CommitStrategyVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyReadScopeClass {
    ExplicitTargetsOnly,
    KindBoundedScan,
    PartitionBoundedScan,
    BoundedNeighborhood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyReadLocalityClass {
    SinglePartition,
    PartitionBounded,
    CrossPartitionBounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyTraversalBasis {
    NoTraversal,
    AdjacencyBounded { max_depth: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyPacketContract {
    ProjectionOnly,
    PlannedPacketOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyReadCostClass {
    ORequestedSurface,
    OPartitionBoundedSurface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyReadContract {
    pub scope_class: StrategyReadScopeClass,
    pub locality_class: StrategyReadLocalityClass,
    pub traversal_basis: StrategyTraversalBasis,
    pub packet_contract: StrategyPacketContract,
    pub cost_class: StrategyReadCostClass,
}
