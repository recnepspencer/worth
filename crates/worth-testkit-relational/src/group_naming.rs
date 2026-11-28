//! Canonical committed-group fixtures from a named full partition.
use worth_relational::facade::aggregate_identity::{
    AggregateGroupNamingDenial, AggregateGroupPartitionTransition, AggregateGroupScheme,
    AggregatePriorGroup, CommittedGroupTransition, GroupTransitionKind,
};

pub fn committed_partition(
    scheme: &AggregateGroupScheme,
    prior: &[AggregatePriorGroup],
    full: &AggregateGroupPartitionTransition,
) -> Result<CommittedGroupTransition, AggregateGroupNamingDenial> {
    Ok(CommittedGroupTransition {
        kind: GroupTransitionKind::Partition,
        scheme: scheme.clone(),
        transition: full.published_transition(prior)?,
    })
}
