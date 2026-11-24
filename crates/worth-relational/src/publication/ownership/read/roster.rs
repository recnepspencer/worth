//! Authenticated current selector rows and their exact retained coverage.
use crate::history::data::coordinates::CommitId;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::publication::ownership::record;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionMemoryReservation;

mod coverage;

#[derive(Debug)]
pub(crate) struct CurrentPublicationRoster {
    pub(in crate::publication::ownership) observed_position_work: u64,
    pub(in crate::publication::ownership) reached_relation_scopes:
        std::collections::BTreeSet<RelationId>,
    pub(in crate::publication::ownership) reached_scopes: std::collections::BTreeSet<EntityId>,
    pub(in crate::publication::ownership) reached_entities: std::collections::BTreeSet<EntityId>,
    pub(in crate::publication::ownership) reached_relations: std::collections::BTreeSet<RelationId>,
    pub(crate) basis: Option<crate::branch::basis_descriptor::RelationalBranchBasisDescriptor>,
    pub(in crate::publication::ownership) publications: Vec<PublicationRow>,
    pub(in crate::publication::ownership) codec_retired: Vec<PublicationRow>,
    pub(in crate::publication::ownership) catalogs: Vec<(EntityId, record::Catalog)>,
    pub(in crate::publication::ownership) edges: Vec<(EntityId, record::Edge)>,
    pub(crate) positions: Vec<crate::history::data::current_route_presence::PresenceRow>,
    pub(in crate::publication::ownership) links: Vec<LinkRow>,
    pub(in crate::publication::ownership) _funding: ExecutionMemoryReservation,
}
#[derive(Debug)]
pub(crate) struct PublicationRow {
    pub(in crate::publication::ownership) entity: EntityId,
    pub(in crate::publication::ownership) value: record::Publication,
    pub(in crate::publication::ownership) digest: [u8; 32],
}
#[derive(Debug)]
pub(in crate::publication::ownership) struct LinkRow {
    pub(in crate::publication::ownership) id: RelationId,
    pub(in crate::publication::ownership) kind: KindId,
    pub(in crate::publication::ownership) publication: CommitId,
    pub(in crate::publication::ownership) source: EntityId,
    pub(in crate::publication::ownership) target: EntityId,
}

impl CurrentPublicationRoster {
    pub(crate) fn covers(&self, records: &[RecordRef], scopes: &[RecordRef]) -> bool {
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
    pub(crate) fn codec_retired_origins(
        &self,
    ) -> impl Iterator<Item = &std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>>
    {
        self.codec_retired.iter().map(|row| &row.value.origin)
    }

    pub(crate) fn codec_retirement_subject(
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

    pub(crate) fn coordinates(
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
    pub(crate) fn origins(
        &self,
    ) -> impl ExactSizeIterator<
        Item = &std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    > {
        self.publications.iter().map(|row| &row.value.origin)
    }
}
