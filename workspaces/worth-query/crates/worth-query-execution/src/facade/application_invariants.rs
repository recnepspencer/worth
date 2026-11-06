//! Installed domain rules and bounded proposed-state access for application hosts.

pub use crate::domain_computation::primary_graph::application_invariant::{
    CustomInvariantInterruption, WorthQueryAggregateIdentity,
    WorthQueryApplicationInvariantComponent, WorthQueryApplicationInvariantComponentSetBinding,
    WorthQueryApplicationInvariantComponents, WorthQueryApplicationInvariantContext,
    WorthQueryApplicationInvariantDerivedMemberRelation,
    WorthQueryApplicationInvariantDerivedOrbit, WorthQueryApplicationInvariantDerivedRelation,
    WorthQueryApplicationInvariantDerivedRelationBinding,
    WorthQueryApplicationInvariantDerivedWalk, WorthQueryApplicationInvariantDerivedWalkStep,
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantEntityBinding,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantGraphComponent, WorthQueryApplicationInvariantGraphComponents,
    WorthQueryApplicationInvariantGraphMember, WorthQueryApplicationInvariantManagedIndexBinding,
    WorthQueryApplicationInvariantOrbit, WorthQueryApplicationInvariantPreparationError,
    WorthQueryApplicationInvariantProjectedCaseBinding, WorthQueryApplicationInvariantReadView,
    WorthQueryApplicationInvariantRelation, WorthQueryApplicationInvariantRelationBinding,
    WorthQueryApplicationInvariantRule, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantWalk,
    WorthQueryApplicationInvariantWalkIter, WorthQueryApplicationInvariantWalkStep,
    WorthQueryEmptyInvariantRefusal, WorthQueryInvariantAccessDenial,
    WorthQueryInvariantAccessDenialKind, WorthQueryInvariantDerivedChanges,
    WorthQueryInvariantManagedIndexPair, WorthQueryInvariantManagedIndexRead,
    WorthQueryPreparedInvariantDerivedChanges, WorthQueryPreparedInvariantManagedIndex,
};
pub use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork;
pub use crate::domain_computation::primary_graph::{
    WorthQueryApplicationInvariantFactories, WorthQueryApplicationInvariantSchemaResolver,
};
pub use worth_foundational::facade::AspectValue;
/// Native kind identity carried as law-refusal diagnostics, without graph authority.
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::{EntityId, RelationId};
/// Original native contract identity and law parameters carried by typed refusals.
pub use worth_relational::facade::schema::{ContractId, SymmetryMode, UniquenessScope};
/// Original proposed or committed endpoints carried by relation-law denials.
pub use worth_relational::facade::transactions::{EntityReference, RelationCardinalityBoundary};

/// Relational owns the cooperative budget supplied to installed application rules.
pub use worth_relational::facade::runtime::CustomInvariantLeaseBudget;

pub use crate::domain_computation::vocabulary::provider_session::invariant_refusal_codec::WorthQueryInvariantRefusalDecodeDenial;
/// Original allocation refusal context carried by invariant read denials.
pub use worth_execution::{MemoryLimitDenial, MemoryLimitLevel};
/// Direction of traversal relative to a stored relation's source and target.
pub use worth_foundational::facade::RelationDirection;
