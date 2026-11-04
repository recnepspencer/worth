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
pub(crate) use payload::decode as decode_payload;
pub(in crate::publication::ownership) mod anchors;
mod reach;

#[derive(Debug)]
pub(crate) struct CurrentPublicationRoster {
    pub(super) reached_entities: std::collections::BTreeSet<EntityId>,
    pub(super) reached_relations: std::collections::BTreeSet<RelationId>,
    pub(crate) basis: crate::branch::RelationalBranchBasisDescriptor,
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
    pub(crate) fn covers(&self, records: &[RecordRef]) -> bool {
        records.iter().all(|r| match r {
            RecordRef::Entity(id) => self.reached_entities.contains(id),
            RecordRef::Relation(id) => self.reached_relations.contains(id),
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

pub(crate) fn read(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    records: &[RecordRef],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    let funding = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(super::request)?
        .map_err(|cause| super::request(crate::execution::denial::lease_denial(cause)))?;
    let mut roster = CurrentPublicationRoster {
        reached_entities: Default::default(),
        reached_relations: Default::default(),
        basis: basis.descriptor().clone(),
        publications: Vec::new(),
        codec_retired: Vec::new(),
        catalogs: Vec::new(),
        edges: Vec::new(),
        links: Vec::new(),
        positions: Vec::new(),
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
        crate::identity::data::VersionId(root.axes().map_or(0, |a| a.storage_version)),
    );
    reach::read(
        execution,
        basis,
        storage,
        &view,
        records,
        &mut roster,
        context,
    )?;
    anchors::validate(
        &mut roster,
        &root.output_publication_positions,
        records,
        context,
    )?;
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

fn damaged(record: &RecordRef) -> TransactionCommitError {
    use crate::storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    let partition = match record {
        RecordRef::Entity(id) => id.partition_id,
        RecordRef::Relation(id) => id.partition_id,
    };
    super::request(
        crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
            RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition),
                cause: RelationalStorageUnavailableCause::Damaged,
            },
        ),
    )
}
