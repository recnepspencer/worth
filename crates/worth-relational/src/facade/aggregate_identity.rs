//! Allocation-independent group names and their canonical lineage records.
pub use crate::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupAssignment, AggregateGroupChange,
    AggregateGroupClaimant, AggregateGroupLineageEvent, AggregateGroupMember,
    AggregateGroupMemberDelta, AggregateGroupMembers, AggregateGroupNaming,
    AggregateGroupNamingAuthority, AggregateGroupNamingDenial, AggregateGroupOrigin,
    AggregateGroupScheme, AggregateGroupSchemeId, AggregateGroupTransition, AggregateMemberRole,
    AggregatePriorGroup, GROUP_AGGREGATE_IDENTITY_DOMAIN_TAG,
    GROUP_AGGREGATE_TRANSITION_DOMAIN_TAG,
};
pub use worth_foundational::facade::AggregateIdentity;
