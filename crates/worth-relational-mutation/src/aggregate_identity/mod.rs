//! One owner for committed aggregate naming and occurrence-qualified lineage.
pub mod candidate_read;
pub mod changes;
pub use candidate_read::AggregateCandidateRead;
pub mod cohorts;
pub mod consequence_plan;
pub mod derived_source;
pub mod final_laws;
pub mod funding;
pub use final_laws::validate_final_derived_laws;
pub use worth_relational_contracts::internal_port::aggregate_identity::group_lineage::AggregateGroupLineageEvent;
pub mod materialization;
pub mod membership;
pub mod observations;
pub mod pinned;
pub mod preparation;
pub mod projected_storage;
pub mod projection;
pub mod proposal;
pub mod read;
pub mod restoration;
pub mod retention_extent;
pub use restoration::RestoredMembership;
pub mod selection;
pub mod stored_lineage;
pub mod stored_read;
pub mod stored_record;
pub mod transition;
pub use observations::AggregateReadObservations;
pub use pinned::{read_aggregate_lineage, read_aggregate_scope};
pub use pinned::{
    PinnedAggregateLineageRead, PinnedAggregateLineageScope, PinnedAggregateRead,
    PinnedAggregateReadDenial, PinnedAggregateReadDenialKind, PinnedAggregateReadScope,
};
pub use preparation::AggregateAdmission;
pub use proposal::{AggregateInvocation, AggregateProposal, AggregateProposalBorrow};
pub use read::{
    AggregateCensusRead, AggregateCensusReadIdentity, AggregateRead, AggregateReadCollection,
    AggregateReadDenial, AggregateReadLineageEvent, AggregateReadPort, PreparedAggregateCensus,
};
pub mod guard;
pub mod record;

pub use transition::group::{
    AggregateContinuityRule, AggregateGroupAssignment, AggregateGroupClaimant,
    AggregateGroupMember, AggregateGroupMembers, AggregateGroupNaming,
    AggregateGroupNamingAuthority, AggregateGroupNamingDenial, AggregateGroupOrigin,
    AggregateGroupScheme, AggregateGroupSchemeId, AggregateGroupTransition, AggregateMemberRole,
    AggregatePriorGroup,
};
pub(crate) use transition::{
    transition, AggregateAssignment, AggregateLineageEvent, AggregateLineageKind,
    AggregateTransitionDenial, PriorAggregate,
};
pub use transition::{GROUP_AGGREGATE_IDENTITY_DOMAIN_TAG, GROUP_AGGREGATE_TRANSITION_DOMAIN_TAG};
