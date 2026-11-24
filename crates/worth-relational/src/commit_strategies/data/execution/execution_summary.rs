use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StrategyExecutionSummary {
    pub projected_entity_record_reads: usize,
    pub projected_relation_record_reads: usize,
    pub projected_partition_reads: usize,
}
