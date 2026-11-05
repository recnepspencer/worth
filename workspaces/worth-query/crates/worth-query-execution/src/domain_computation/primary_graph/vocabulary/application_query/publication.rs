//! Moved whole from `domain_computation::primary_graph::application_query::derived_view::publication`: the items below and every impl they own.
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::CanonicalFieldPath;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_runtime_world::facade::CompositeCommitIdentity;
use worth_runtime_world::facade::ProductBranchIdentity;
use worth_runtime_world::facade::ProductBranchIncarnation;

/// Canonical changes are translated to these dependency-level events before
/// publication. A field event also changes aspect readers; a whole-aspect
/// event changes every field reader of that aspect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewChange {
    /// Record lifecycle can change, so every dependency on the entity is affected.
    Entity(EntityId),
    Aspect(EntityId, AspectKey),
    Field(EntityId, AspectKey, CanonicalFieldPath),
    Adjacency(EntityId, KindId, u8),
}

pub struct ViewPublicationBasis<'a> {
    pub before: &'a CompositeCommitIdentity,
    pub relational_branch: &'a BranchId,
    pub product_branch: &'a ProductBranchIdentity,
    pub incarnation: ProductBranchIncarnation,
}
