//! Moved whole from `domain_computation::primary_graph::vocabulary::freshness`: the items below and every impl they own.
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::RelationId;

/// Fixed native revision evidence for a producer decision that used its
/// authenticated principal. A value rewrite changes its field revision and
/// therefore refuses reuse even when the new value compares equal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPrincipalReuseWitness {
    mapping: EntityId,
    mapping_kind: KindId,
    mapping_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    mapping_status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    relation: RelationId,
    relation_kind: KindId,
    principal: EntityId,
    principal_kind: KindId,
    principal_identity_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    adjacency_revision: Option<worth_relational::facade::identity::VersionId>,
}
