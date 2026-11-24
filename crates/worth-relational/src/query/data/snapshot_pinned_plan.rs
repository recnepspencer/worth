use super::planned_packets::{
    PlannedQueryPacket, QueryParallelLegality, QueryParallelProfitability,
};
use crate::snapshots::data::SnapshotHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPinnedQueryPlan {
    pub packet: PlannedQueryPacket,
    pub snapshot: SnapshotHandle,
    pub legality: QueryParallelLegality,
    pub profitability: QueryParallelProfitability,
}
