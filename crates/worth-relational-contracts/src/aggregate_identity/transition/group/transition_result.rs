use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use super::model::AggregateGroupAssignment;
use crate::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;

/// The complete naming outcome of one group transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateGroupTransition {
    /// Next-state groups in identity order.
    pub assignments: Vec<AggregateGroupAssignment>,
    pub retired: BTreeSet<AggregateIdentity>,
    pub lineage: BTreeSet<Event>,
}
