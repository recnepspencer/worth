//! Join the sealed storage journal to its canonical envelope once per commit.
use super::change::{ChangedRecord, CommitChange, PartitionChange, RecordChangeState};
use super::{ChangeWork, MaintenanceRefusal, MaintenanceRoute};
use crate::branch::{RelationalBranchRoot, RelationalBranchRootCaptureDenial as Denial};
use crate::history::data::{CanonicalCommitEnvelope, RelationalDescriptiveTouch as Touch};
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::overlay::PartitionState;
use crate::storage::residency::{PartitionHeadProbe, RelationalRegionHeadView};
use crate::storage::substrate::{EntityRecordKind, RecordKind, RelationRecordKind, SlotView};
use crate::storage::RelationalPublishedPartitionDelta;
use crate::transactions::data::RecordRef;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl CommitChange {
    /// Cost: changed slots times region/slot lookup paths, plus commit symbols.
    pub(crate) fn from_publication(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: Option<&RelationalBranchRoot>,
        delta: &RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
        schema_changed: bool,
    ) -> Result<Arc<Self>, Denial> {
        ChangeWork::run(execution, |work| {
            Self::build(execution, previous, delta, envelope, schema_changed, work)
        })
    }

    fn build(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: Option<&RelationalBranchRoot>,
        delta: &RelationalPublishedPartitionDelta,
        envelope: Arc<CanonicalCommitEnvelope>,
        schema_changed: bool,
        work: &mut ChangeWork<'_>,
    ) -> Result<Arc<Self>, Denial> {
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
            partitions.insert(
                id,
                PartitionChange {
                    journal: journal.clone(),
                    records,
                },
            );
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
        Ok(Arc::new(Self {
            envelope,
            partitions,
            schema_changed,
            required_symbols,
            created,
            retired,
            routes,
        }))
    }
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

fn project_records(
    id: PartitionId,
    before: Option<RelationalRegionHeadView<'_>>,
    after: &PartitionState,
    journal: &crate::storage::overlay::PartitionMutationJournal,
    work: &mut ChangeWork<'_>,
) -> Result<Vec<ChangedRecord>, Denial> {
    let mut records = Vec::new();
    for &slot in &journal.entity_slots {
        work.prepare(65, 256)?;
        let identity = |generation| RecordRef::Entity(EntityId::new(id, slot as u64, generation));
        records.push(ChangedRecord {
            before: before.as_ref().and_then(|old| {
                state::<EntityRecordKind>(old.entity_arena().get_slot(slot), identity)
            }),
            after: state::<EntityRecordKind>(after.entity_arena.get_slot(slot), identity),
        });
    }
    for &slot in &journal.relation_slots {
        work.prepare(65, 256)?;
        let identity =
            |generation| RecordRef::Relation(RelationId::new(id, slot as u64, generation));
        records.push(ChangedRecord {
            before: before.as_ref().and_then(|old| {
                state::<RelationRecordKind>(old.relation_arena().get_slot(slot), identity)
            }),
            after: state::<RelationRecordKind>(after.relation_arena.get_slot(slot), identity),
        });
    }
    Ok(records)
}

impl ChangedRecord {
    fn created_identity(&self) -> Option<&RecordRef> {
        let after = self.after.as_ref()?;
        self.before
            .as_ref()
            .is_none_or(|before| before.record != after.record)
            .then_some(&after.record)
    }
    fn retired_identity(&self) -> Option<&RecordRef> {
        let before = self.before.as_ref()?;
        self.after
            .as_ref()
            .is_none_or(|after| {
                after.record != before.record
                    || after.retired_at != before.retired_at && after.retired_at.is_some()
            })
            .then_some(&before.record)
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

fn state<K: RecordKind>(
    view: Option<SlotView<'_, K>>,
    record: impl FnOnce(u32) -> RecordRef,
) -> Option<RecordChangeState> {
    let view = view?;
    Some(RecordChangeState {
        record: record(view.generation()),
        kind: view.kind_id()?,
        lifecycle: view.lifecycle(),
        retired_at: view.retired_at(),
    })
}

impl From<MaintenanceRefusal> for Denial {
    fn from(cause: MaintenanceRefusal) -> Self {
        match cause {
            MaintenanceRefusal::Execution(cause) => Self::Execution(cause),
            MaintenanceRefusal::MissingPredecessor | MaintenanceRefusal::UnavailableChange => {
                Self::CanonicalEnvelopeMismatch
            }
        }
    }
}
