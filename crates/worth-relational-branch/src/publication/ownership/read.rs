//! Read and authenticate the current private roster without historical members.
use super::read_request::{checkpoint, corrupt, damaged, request};
use super::{record, record_payload as payload};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::structural_state::StructuralStateView;
use crate::history::data::coordinates::CommitId;
use crate::identity::data::RelationId;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::CurrentPublicationStorage;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
pub mod entity;
pub mod integrity;
pub mod linked;
pub mod selector;
pub use linked::selectors as linked_selectors;
pub mod anchors;
pub mod reach;
pub mod roster;
pub use roster::CurrentPublicationRoster;
pub(super) use roster::LinkRow;
pub use roster::PublicationRow;
pub mod source;
pub use reach::claims as publication_claims;
use source::OwnershipReadSource;

pub fn read(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    records: &[RecordRef],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    read_selected(execution, basis, records, records, None, context)
}

/// Field-only writes select claims; destructive and lifecycle selectors also
/// discover the source's own scope and position incidence.
pub fn read_selected(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    records: &[RecordRef],
    scopes: &[RecordRef],
    after: Option<&StructuralStateView<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    let roster = read_source(
        execution,
        OwnershipReadSource::Admitted(basis),
        records,
        scopes,
        after,
        context,
    )?;
    super::observation::record(&roster, basis);
    Ok(roster)
}

/// A candidate observation cannot be reused as admitted current publication custody.
pub fn read_candidate(
    execution: &ExecutionRequest<'_, '_>,
    root: &crate::branch::root::RelationalBranchRoot,
    records: &[RecordRef],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    read_source(
        execution,
        OwnershipReadSource::Candidate(root),
        records,
        records,
        None,
        context,
    )
}

fn read_source(
    execution: &ExecutionRequest<'_, '_>,
    source: OwnershipReadSource<'_>,
    records: &[RecordRef],
    scopes: &[RecordRef],
    after: Option<&StructuralStateView<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    let mut roster = empty(execution, source)?;
    let root = source.root();
    let Some(storage) = &(*root
        .schema_authority()
        .registry()
        .owner_current_publication_storage())
    else {
        return Ok(roster);
    };
    let view = StructuralStateView::new(
        root,
        crate::identity::data::VersionId(root.axes().map_or(0, |a| a.storage_version)),
    );
    reach::read(
        execution,
        source,
        storage,
        &view,
        records,
        scopes,
        after,
        &mut roster,
        context,
    )?;
    anchors::validate(
        &mut roster,
        &root.output_publication_positions,
        scopes,
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
            request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
    funding.resize(bytes).map_err(|cause| {
        request(crate::execution::denial::lease_denial(
            worth_execution::LeaseDenial::MemoryExhausted(cause),
        ))
    })
}

fn empty(
    execution: &ExecutionRequest<'_, '_>,
    source: OwnershipReadSource<'_>,
) -> Result<CurrentPublicationRoster, TransactionCommitError> {
    let funding = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(request)?
        .map_err(|cause| request(crate::execution::denial::lease_denial(cause)))?;
    Ok(CurrentPublicationRoster {
        observed_position_work: 0,
        reached_relation_scopes: Default::default(),
        reached_scopes: Default::default(),
        reached_entities: Default::default(),
        reached_relations: Default::default(),
        basis: source.descriptor(),
        publications: Vec::new(),
        codec_retired: Vec::new(),
        catalogs: Vec::new(),
        edges: Vec::new(),
        links: Vec::new(),
        positions: Vec::new(),
        _funding: funding,
    })
}
