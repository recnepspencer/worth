use serde::{Deserialize, Serialize};

use crate::schema::data::{DescriptorSemanticsVersion, SchemaTransitionSummary};
use crate::validation::data::groups::InvariantGroupSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitStructuralSummary {
    pub invariant_groups: InvariantGroupSet,
    pub commit_topology: crate::transactions::data::mutation_planning::CommitTopology,
    pub touched_partitions: Vec<crate::identity::data::PartitionId>,
    pub bulk_entity_slots_reserved: usize,
    pub bulk_relation_slots_reserved: usize,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSchemaSummary {
    pub transition: Option<SchemaTransitionSummary>,
    pub descriptor_semantics_version: DescriptorSemanticsVersion,
}
