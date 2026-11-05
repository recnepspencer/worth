//! Moved whole from `domain_computation::primary_graph::vocabulary::freshness`: the items below and every impl they own.
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::RelationId;

/// Fixed native revision evidence for a producer decision that used its
/// authenticated principal. A value rewrite changes its field revision and
/// therefore refuses reuse even when the new value compares equal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPrincipalReuseWitness {
    pub(in crate::domain_computation) mapping: EntityId,
    pub(in crate::domain_computation) mapping_kind: KindId,
    pub(in crate::domain_computation) mapping_identity_revision:
        worth_relational::facade::runtime::RelationalFieldRevision,
    pub(in crate::domain_computation) mapping_status_revision:
        worth_relational::facade::runtime::RelationalFieldRevision,
    pub(in crate::domain_computation) relation: RelationId,
    pub(in crate::domain_computation) relation_kind: KindId,
    pub(in crate::domain_computation) principal: EntityId,
    pub(in crate::domain_computation) principal_kind: KindId,
    pub(in crate::domain_computation) principal_identity_revision:
        worth_relational::facade::runtime::RelationalFieldRevision,
    pub(in crate::domain_computation) adjacency_revision:
        Option<worth_relational::facade::identity::VersionId>,
}
