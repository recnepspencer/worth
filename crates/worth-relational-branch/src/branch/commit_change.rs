//! Join the sealed storage journal to its original canonical envelope once.
use super::maintenance::CommitChange;
use super::root::{RelationalBranchRoot, RelationalBranchRootCaptureDenial as Denial};
use crate::history::data::{CanonicalCommitEnvelope, RelationalDescriptiveTouch as Touch};
use crate::identity::data::PartitionId;
use crate::storage::residency::{PartitionHeadProbe, RelationalRegionHeadView};
use crate::storage::substrate::maintenance::{
    partition_records::project_records, ChangeWork, ChangedRecord, MaintenanceRefusal,
    MaintenanceRoute,
};
use crate::storage::RelationalPublishedPartitionDelta;
use crate::transactions::data::primitives::RecordRef;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_relational_storage::internal_port::storage::substrate::maintenance::partition_change::assemble_partition_change;
use worth_relational_storage::internal_port::storage::substrate::maintenance::partition_change::ChangedRecordOwnerPort as _;

/// Cost: changed slots times region/slot lookup paths, plus commit symbols.
pub fn build_commit_change(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    previous: Option<&RelationalBranchRoot>,
    delta: &RelationalPublishedPartitionDelta,
    envelope: Arc<CanonicalCommitEnvelope>,
    schema_changed: bool,
) -> Result<Arc<CommitChange>, Denial> {
    ChangeWork::run(execution, |work| {
        build(execution, previous, delta, envelope, schema_changed, work)
    })
}

fn build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    previous: Option<&RelationalBranchRoot>,
    delta: &RelationalPublishedPartitionDelta,
    envelope: Arc<CanonicalCommitEnvelope>,
    schema_changed: bool,
    work: &mut ChangeWork<'_>,
) -> Result<Arc<CommitChange>, Denial> {
    let mut partitions = BTreeMap::new();
    let mut routes = BTreeSet::new();
    let mut created = Vec::new();
    let mut retired = Vec::new();
    for id in delta.partition_ids() {
        work.prepare(33, 512)?;
        let (overlay, journal) = delta
            .publication(id)
            .ok_or(Denial::PublishedPartitionMissing(id))?;
        let records = project_records(
            id,
            admitted_before(execution, previous, id)?,
            overlay,
            journal,
            work,
        )?;
        for record in &records {
            add_record_routes(record, &mut routes);
            created.extend(record.created_identity().cloned());
            retired.extend(record.retired_identity().cloned());
        }
        routes.insert(MaintenanceRoute::Partition(id));
        partitions.insert(id, assemble_partition_change(journal.clone(), records));
    }
    if schema_changed {
        routes.insert(MaintenanceRoute::Schema);
    }
    add_touch_routes(&envelope, &mut routes, work)?;
    let mut required_symbols = BTreeSet::new();
    for (symbol, _) in &envelope.symbol_texts.entries {
        work.prepare(1, 256)?;
        required_symbols.insert(*symbol);
    }
    if !required_symbols.is_empty() {
        routes.insert(MaintenanceRoute::Symbols);
    }
    Ok(Arc::new(CommitChange {
        envelope: crate::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort::from_arc(envelope),
        partitions,
        schema_changed,
        required_symbols,
        created,
        retired,
        routes,
    }))
}

fn admitted_before<'root>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    previous: Option<&'root RelationalBranchRoot>,
    id: PartitionId,
) -> Result<Option<RelationalRegionHeadView<'root>>, Denial> {
    match previous.map(|root| root.probe_head_region(id)) {
        None | Some(PartitionHeadProbe::Absent) => Ok(None),
        Some(PartitionHeadProbe::Resident(view)) => Ok(Some(view)),
        Some(PartitionHeadProbe::Cold) => {
            let root = previous.ok_or(Denial::PublishedPartitionMissing(id))?;
            root.demand_region_head_cached(execution, id)
                .map_err(|failure| {
                    use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
                    match failure {
                        Failure::StorageUnavailable(cause) => Denial::StorageUnavailable(cause),
                        Failure::Request(cause) => Denial::Execution(cause),
                        Failure::Content(cause) => Denial::Content(cause),
                    }
                })?;
            root.accepted_head_region_view(id)
                .map(Some)
                .ok_or(Denial::PublishedPartitionMissing(id))
        }
    }
}

fn add_record_routes(record: &ChangedRecord, routes: &mut BTreeSet<MaintenanceRoute>) {
    for state in [record.before.as_ref(), record.after.as_ref()]
        .into_iter()
        .flatten()
    {
        routes.insert(match &state.record {
            RecordRef::Entity(_) => MaintenanceRoute::EntityKind(state.kind),
            RecordRef::Relation(_) => MaintenanceRoute::RelationKind(state.kind),
        });
    }
}

fn add_touch_routes(
    envelope: &CanonicalCommitEnvelope,
    routes: &mut BTreeSet<MaintenanceRoute>,
    work: &mut ChangeWork<'_>,
) -> Result<(), MaintenanceRefusal> {
    for touch in envelope
        .descriptive_touches()
        .exact_touches()
        .into_iter()
        .flatten()
    {
        work.prepare(1, 512)?;
        let route = match touch {
            Touch::FieldRevision {
                kind, aspect, path, ..
            } => MaintenanceRoute::Field {
                kind: *kind,
                aspect: aspect.clone(),
                path: path.clone(),
            },
            Touch::IndexMembership { index, .. } | Touch::IndexDefinition { index } => {
                MaintenanceRoute::Index(*index)
            }
            Touch::EntityLifecycle { kind, .. } => MaintenanceRoute::EntityKind(*kind),
            Touch::RelationLifecycle { kind, .. }
            | Touch::RelationMembership { kind, .. }
            | Touch::AdjacencyRevision { kind, .. } => MaintenanceRoute::RelationKind(*kind),
            Touch::AspectRevision { .. } => continue,
        };
        routes.insert(route);
    }
    Ok(())
}
