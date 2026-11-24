//! Authenticated current selector rows and their exact retained coverage.
use crate::history::data::coordinates::CommitId;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::publication::ownership::record;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionMemoryReservation;

pub mod coverage;

#[derive(Debug)]
pub struct CurrentPublicationRoster {
    pub(in crate::internal_port::publication::ownership) observed_position_work: u64,
    pub(in crate::internal_port::publication::ownership) reached_relation_scopes:
        std::collections::BTreeSet<RelationId>,
    pub(in crate::internal_port::publication::ownership) reached_scopes:
        std::collections::BTreeSet<EntityId>,
    pub(in crate::internal_port::publication::ownership) reached_entities:
        std::collections::BTreeSet<EntityId>,
    pub(in crate::internal_port::publication::ownership) reached_relations:
        std::collections::BTreeSet<RelationId>,
    pub basis: Option<crate::branch::basis_descriptor::RelationalBranchBasisDescriptor>,
    pub publications: Vec<PublicationRow>,
    pub codec_retired: Vec<PublicationRow>,
    pub catalogs: Vec<(EntityId, record::Catalog)>,
    pub edges: Vec<(EntityId, record::Edge)>,
    pub positions: Vec<crate::history::data::current_route_presence::PresenceRow>,
    pub links: Vec<LinkRow>,
    pub _funding: ExecutionMemoryReservation,
}
#[derive(Debug)]
pub struct PublicationRow {
    pub entity: EntityId,
    pub value: record::Publication,
    pub(in crate::internal_port::publication::ownership) digest: [u8; 32],
}
#[derive(Debug)]
pub struct LinkRow {
    pub id: RelationId,
    pub kind: KindId,
    pub(in crate::internal_port::publication::ownership) publication: CommitId,
    pub source: EntityId,
    pub(in crate::internal_port::publication::ownership) target: EntityId,
}

impl CurrentPublicationRoster {
    pub fn covers(&self, records: &[RecordRef], scopes: &[RecordRef]) -> bool {
        coverage::covers(
            &self.reached_entities,
            &self.reached_relations,
            &self.reached_scopes,
            &self.reached_relation_scopes,
            records,
            scopes,
        )
    }

    /// A retired closure is not a published role grant. This coordinate only
    /// names the custody owner that must prepare its historical material.
    pub fn codec_retired_origins(
        &self,
    ) -> impl Iterator<Item = &std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>>
    {
        self.codec_retired.iter().map(|row| &row.value.origin)
    }

    pub fn codec_retirement_subject(
        &self,
    ) -> Option<(
        crate::history::data::CommitId,
        crate::identity::data::EntityId,
    )> {
        self.codec_retired.first().map(|row| {
            (
                row.value.origin.commit.commit_id,
                row.value.origin.meaning.source_scope,
            )
        })
    }

    pub fn coordinates(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
            usize,
        ),
    > {
        self.publications
            .iter()
            .map(|row| (&row.value.origin, row.value.ordinal))
    }
    pub fn origins(
        &self,
    ) -> impl ExactSizeIterator<
        Item = &std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    > {
        self.publications.iter().map(|row| &row.value.origin)
    }
}
