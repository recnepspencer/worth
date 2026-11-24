//! Exact selected-basis producer custody; a caller supplies no ownership roster.
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::data::PublicationError;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};
mod coverage;
pub(crate) mod graph;
pub(crate) use coverage::require_graph_coverage;

/// Prepared from the Native current-fact writers at both retained roots.
/// Selection retains original versions; absence retains the displaced current
/// publication and the owner's actual negative observation at the selected root.
#[derive(Clone, Debug)]
pub struct RelationalHistoricalOutputSelection {
    pub(crate) current: AdmittedRelationalBranchBasis,
    pub(crate) selected: AdmittedRelationalBranchBasis,
    pub(crate) publication: RelationalRequiredOutputPublication,
    pub(crate) predecessor: Option<RelationalRequiredOutputPublication>,
    pub(crate) select_version: bool,
    _funding: Arc<Option<worth_execution::ExecutionMemoryReservation>>,
}
impl RelationalHistoricalOutputSelection {
    /// The selected publication, or the current publication displaced by absence.
    pub fn publication(&self) -> &RelationalRequiredOutputPublication {
        &self.publication
    }
    pub fn selects_absence(&self) -> bool {
        !self.select_version
    }
}

pub(crate) fn prepare_historical_output_selection(
    runtime_instance_id: u64,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    current: &AdmittedRelationalBranchBasis,
    selected: &AdmittedRelationalBranchBasis,
    route: &RequiredOutputPublicationMeaning,
) -> Result<RelationalHistoricalOutputSelection, TransactionCommitError> {
    let subject = RecordRef::Entity(route.source_scope);
    require_bases(runtime_instance_id, installed, current, selected, &subject)?;
    installed.require_current_ownership_storage(current, route.source_scope)?;
    installed.require_current_ownership_storage(selected, route.source_scope)?;
    execution
        .run_native_with_checkpoint(|context| {
            let selected_roster = Arc::new(crate::publication::ownership::read(
                execution,
                selected,
                std::slice::from_ref(&subject),
                context,
            )?);
            let current_roster = Arc::new(crate::publication::ownership::read(
                execution,
                current,
                std::slice::from_ref(&subject),
                context,
            )?);
            let publication = on_route(installed, selected, selected_roster, route, context)?
                .ok_or_else(|| {
                    TransactionCommitError::publication(PublicationError::authored_retirement(
                        Cause::HistoricalSelectionRouteMissing,
                        subject.clone(),
                    ))
                })?;
            let predecessor = on_route(installed, current, current_roster, route, context)?;
            let funding = funding(execution, 1, context)?;
            Ok(RelationalHistoricalOutputSelection {
                current: current.clone(),
                selected: selected.clone(),
                publication,
                predecessor,
                select_version: true,
                _funding: funding,
            })
        })
        .map_err(custody::execution_stop)?
        .0
}

fn require_bases(
    runtime_instance_id: u64,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    current: &AdmittedRelationalBranchBasis,
    selected: &AdmittedRelationalBranchBasis,
    subject: &RecordRef,
) -> Result<(), TransactionCommitError> {
    if current.identity().runtime_instance_id() != runtime_instance_id
        || selected.identity().runtime_instance_id() != runtime_instance_id
        || installed.runtime_instance_id != runtime_instance_id
        || installed.schema_digest
            != (*current.owner_inner())
                .root
                .schema_authority()
                .authority_digest()
        || (*current.owner_inner())
            .root
            .schema_authority()
            .authority_digest()
            != (*selected.owner_inner())
                .root
                .schema_authority()
                .authority_digest()
        || !current.is_current()
    {
        return Err(TransactionCommitError::publication(
            PublicationError::authored_retirement(
                Cause::HistoricalSelectionBasisChanged,
                subject.clone(),
            ),
        ));
    }
    Ok(())
}
fn funding(
    execution: &ExecutionRequest<'_, '_>,
    count: usize,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Arc<Option<worth_execution::ExecutionMemoryReservation>>, TransactionCommitError> {
    let bytes = (count as u64)
        .saturating_mul(2 * std::mem::size_of::<RelationalHistoricalOutputSelection>() as u64 + 64);
    crate::execution::request_stage::checkpoint(context, bytes).map_err(custody::execution_stop)?;
    execution
        .reserve_retained_memory(bytes)
        .map(Arc::new)
        .map_err(custody::execution_stop)
}
fn same_route(
    left: &RequiredOutputPublicationMeaning,
    right: &RequiredOutputPublicationMeaning,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, TransactionCommitError> {
    crate::execution::request_stage::checkpoint(
        context,
        (left.family.len() + right.family.len() + 65) as u64,
    )
    .map_err(custody::execution_stop)?;
    Ok(left.same_route(right))
}
fn on_route(
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    roster: Arc<crate::publication::ownership::CurrentPublicationRoster>,
    route: &RequiredOutputPublicationMeaning,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Option<RelationalRequiredOutputPublication>, TransactionCommitError> {
    let mut found = None;
    for (origin, ordinal) in roster.coordinates() {
        if same_route(&origin.meaning, route, context)? {
            if found.is_some() {
                return Err(refuse(
                    origin.commit.commit_id,
                    RecordRef::Entity(route.source_scope),
                ));
            }
            found = Some(admit_publication(
                installed,
                basis,
                Arc::clone(&roster),
                Arc::clone(origin),
                ordinal,
            )?);
        }
    }
    Ok(found)
}
fn admit_publication(
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    roster: Arc<crate::publication::ownership::CurrentPublicationRoster>,
    origin: Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    ordinal: usize,
) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
    let source_kind = (*basis.owner_inner())
        .root
        .get_current_partition(origin.meaning.source_scope.partition_id)
        .and_then(|region| region.entity(origin.meaning.source_scope))
        .filter(|slot| slot.is_live())
        .and_then(|slot| slot.kind_id());
    if installed
        .source_scope_ownership
        .scope_kind(&origin.producer)
        != source_kind
        || !origin.source_scope_owned
        || installed
            .source_scope_ownership
            .scope_kind(&origin.producer)
            .is_none()
    {
        return Err(refuse(
            origin.commit.commit_id,
            RecordRef::Entity(origin.meaning.source_scope),
        ));
    }
    Ok(RelationalRequiredOutputPublication {
        runtime_instance_id: installed.runtime_instance_id,
        authority: installed
            .authority
            .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal),
        evidence: crate::publication::ownership::PublicationEvidence::Current(roster),
        origin,
        readmitted_at: Some(basis.clone()),
    })
}
pub(crate) fn refuse(
    publication: crate::history::data::CommitId,
    subject: RecordRef,
) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        Cause::HistoricalSelectionNotOwned { publication },
        subject,
    ))
}

/// The union may name records that did not exist at this basis. Admit point
/// liveness before calling the custody reader, whose selectors name existing
/// records. This filters presence only; it derives no publication ownership.
fn existing_entities(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    entities: impl Iterator<Item = crate::identity::data::EntityId>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<RecordRef>, TransactionCommitError> {
    let view = crate::validation::engine::state_view::InvariantStateView::new(
        (*basis.owner_inner()).root.as_ref(),
        crate::identity::data::VersionId(
            (*basis.owner_inner())
                .root
                .axes()
                .map_or(0, |axes| axes.storage_version),
        ),
    );
    let mut records = Vec::new();
    for id in entities {
        crate::execution::request_stage::checkpoint(context, 1).map_err(custody::execution_stop)?;
        (*basis.owner_inner())
            .root
            .demand_head_partition(execution, id.partition_id)
            .map_err(|stop| {
                crate::authority::commit::preparation::storage_read_denial(basis, stop)
            })?;
        if view.entity_metadata(id).is_some() {
            records.push(RecordRef::Entity(id));
        }
    }
    Ok(records)
}
