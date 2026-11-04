//! Read and authenticate the current private roster without historical members.
use super::{checkpoint, corrupt, record};
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::CommitId,
    identity::data::{EntityId, KindId, RelationId},
    schema::data::CurrentPublicationStorage,
    storage::overlay::CurrentPartitionAccess,
    transactions::data::{RecordRef, TransactionCommitError},
    validation::engine::state_view::InvariantStateView,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};
mod integrity;
mod payload;

#[derive(Debug)]
pub(crate) struct CurrentPublicationRoster {
    pub(crate) basis: crate::branch::RelationalBranchBasisDescriptor,
    pub(in crate::publication::ownership) publications: Vec<PublicationRow>,
    pub(in crate::publication::ownership) catalogs: Vec<(EntityId, record::Catalog)>,
    pub(in crate::publication::ownership) edges: Vec<(EntityId, record::Edge)>,
    pub(in crate::publication::ownership) links: Vec<LinkRow>,
    pub(in crate::publication::ownership) _funding: ExecutionMemoryReservation,
}
#[derive(Debug)]
pub(crate) struct PublicationRow {
    pub(in crate::publication::ownership) entity: EntityId,
    pub(in crate::publication::ownership) value: record::Publication,
    pub(super) digest: [u8; 32],
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

pub(crate) fn read(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    let funding = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(super::request)?
        .map_err(|cause| super::request(crate::execution::denial::lease_denial(cause)))?;
    let mut roster = CurrentPublicationRoster {
        basis: basis.descriptor().clone(),
        publications: Vec::new(),
        catalogs: Vec::new(),
        edges: Vec::new(),
        links: Vec::new(),
        _funding: funding,
    };
    let root = &basis.inner.root;
    let Some(storage) = &root
        .schema_authority()
        .registry()
        .current_publication_storage
    else {
        return Ok(roster);
    };
    let view = InvariantStateView::new(
        root.as_ref(),
        crate::identity::data::VersionId(root.axes().map_or(0, |axes| axes.storage_version)),
    );
    for partition in root.partition_ids_iter() {
        checkpoint(context, 1)?;
        root.demand_head_partition(execution, partition)
            .map_err(|cause| {
                crate::authority::commit::preparation::storage_read_denial(basis, cause)
            })?;
        let Some(head) = root.get_current_partition(partition) else {
            continue;
        };
        for slot in head.entity_arena().live_slots() {
            checkpoint(context, 1)?;
            let Some(row) = head.entity_arena().get_slot(slot) else {
                continue;
            };
            let kind = row.kind_id();
            let aspect = if kind == Some(storage.catalog_kind) {
                crate::schema::current_publication_ownership::CATALOG
            } else if kind == Some(storage.publication_kind) {
                crate::schema::current_publication_ownership::PUBLICATION
            } else if kind == Some(storage.edge_kind) {
                crate::schema::current_publication_ownership::EDGE
            } else {
                continue;
            };
            let id = EntityId::new(partition, slot as u64, row.generation());
            let (publication, bytes) = view
                .entity_aspect_state(id)
                .and_then(|state| record::fields(state, aspect))
                .ok_or_else(|| corrupt(CommitId(0), RecordRef::Entity(id)))?;
            let subject = RecordRef::Entity(id);
            if kind == Some(storage.catalog_kind) {
                let value =
                    payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
                retain::<(EntityId, record::Catalog)>(&mut roster._funding)?;
                roster.catalogs.push((id, value));
            } else if kind == Some(storage.publication_kind) {
                let value: record::Publication =
                    payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
                if value.origin.commit.commit_id != publication {
                    return Err(corrupt(publication, subject));
                }
                use sha2::Digest;
                let digest = sha2::Sha256::digest(bytes).into();
                // The selected closure's exact-capacity Vec is retained by the
                // same roster evidence, so quote its entries here as well.
                retain::<(
                    PublicationRow,
                    crate::history::data::SelectedOutputPublication,
                )>(&mut roster._funding)?;
                roster.publications.push(PublicationRow {
                    entity: id,
                    value,
                    digest,
                });
            } else {
                let value: record::Edge =
                    payload::decode(bytes, publication, &subject, &mut roster._funding, context)?;
                if value.publication != publication {
                    return Err(corrupt(publication, subject));
                }
                retain::<(EntityId, record::Edge)>(&mut roster._funding)?;
                roster.edges.push((id, value));
            }
        }
        for slot in head.relation_arena().live_slots() {
            checkpoint(context, 1)?;
            let Some(row) = head.relation_arena().get_slot(slot) else {
                continue;
            };
            let Some(kind) = row.kind_id() else {
                continue;
            };
            if ![
                storage.publication_link,
                storage.member_link,
                storage.edge_link,
                storage.scope_link,
            ]
            .contains(&kind)
            {
                continue;
            }
            let id = RelationId::new(partition, slot as u64, row.generation());
            let (publication, bytes) = view
                .relation_aspect_state(id)
                .and_then(|state| {
                    record::fields(state, crate::schema::current_publication_ownership::LINK)
                })
                .ok_or_else(|| corrupt(CommitId(0), RecordRef::Relation(id)))?;
            if !bytes.is_empty() {
                return Err(corrupt(publication, RecordRef::Relation(id)));
            }
            let endpoints = row
                .extra()
                .endpoints
                .as_ref()
                .ok_or_else(|| corrupt(publication, RecordRef::Relation(id)))?;
            retain::<LinkRow>(&mut roster._funding)?;
            roster.links.push(LinkRow {
                id,
                kind,
                publication,
                source: endpoints.source,
                target: endpoints.target,
            });
        }
    }
    integrity::validate(&roster, storage, &view, context)?;
    Ok(roster)
}

fn retain<T>(funding: &mut ExecutionMemoryReservation) -> Result<(), TransactionCommitError> {
    let bytes = funding
        .bytes()
        .checked_add((4 * std::mem::size_of::<T>()) as u64)
        .ok_or_else(|| {
            super::request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    funding.resize(bytes).map_err(|cause| {
        super::request(crate::execution::denial::lease_denial(
            worth_execution::LeaseDenial::MemoryExhausted(cause),
        ))
    })
}
