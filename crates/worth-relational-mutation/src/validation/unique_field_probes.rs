//! Unique keys carried by the exact applied entity-slot delta.
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelStop, WorkCeilingDenial};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, PortableAspectPatchOperation};
use worth_relational_branch::internal_port::branch::structural_state::StructuralStateView;
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::query_locus_projection::authoritative_state_query_locus_value;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use crate::contracts::diagnostics::data::DiagnosticCode;
use crate::contracts::identity::data::{EntityId, KindId, PartitionId};
use crate::contracts::indexes::data::definition::DerivedIndexId;
use crate::contracts::storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey as Key;
use crate::contracts::transactions::data::intents::mutation_intent::{
    EntityMutationIntent, MutationIntent,
};
use crate::contracts::transactions::data::outcomes::{
    commit_conflict::{CommitConflict, CommitConflictOwnerPort as _},
    conflict_class::ConflictClass,
    transaction_commit_error::TransactionCommitError,
};
use crate::contracts::transactions::data::primitives::RecordRef;
use crate::contracts::validation::data::{
    execution::InvariantClass,
    results::{InvariantViolation, InvariantViolationFields},
};

#[derive(Debug)]
pub struct UniqueFieldProbeClaims {
    entity_kind_id: KindId,
    backing_index_id: DerivedIndexId,
    field_locator: AspectFieldLocator,
    keys: BTreeMap<Key, KeyChange>,
    _memory: Option<ExecutionMemoryReservation>,
}

#[derive(Debug, Default)]
struct KeyChange {
    writer: Option<(EntityId, AspectValue)>,
    retiring: BTreeSet<EntityId>,
}

#[derive(Clone, Copy)]
pub struct UniqueFieldScope<'a> {
    pub entity_kind_id: KindId,
    pub backing_index_id: DerivedIndexId,
    pub field_locator: &'a AspectFieldLocator,
}

impl UniqueFieldProbeClaims {
    pub fn scope(&self) -> UniqueFieldScope<'_> {
        UniqueFieldScope {
            entity_kind_id: self.entity_kind_id,
            backing_index_id: self.backing_index_id,
            field_locator: &self.field_locator,
        }
    }

    pub fn claims(
        &self,
    ) -> impl Iterator<Item = (&Key, EntityId, &AspectValue, &BTreeSet<EntityId>)> {
        self.keys.iter().filter_map(|(key, change)| {
            let (writer, value) = change.writer.as_ref()?;
            Some((key, *writer, value, &change.retiring))
        })
    }
}

impl UniqueFieldScope<'_> {
    pub fn duplicate_conflict(self, value: AspectValue) -> CommitConflict {
        let violation = self.violation(InvariantClass::CommitBoundary, value);
        CommitConflict::new(ConflictClass::InvariantViolation {
            code: violation.code,
            detail: violation.detail,
            fields: violation.fields,
        })
    }

    pub fn violation(self, class: InvariantClass, value: AspectValue) -> InvariantViolation {
        InvariantViolation {
            class,
            code: DiagnosticCode::InvariantViolation,
            detail: format!(
                "entity kind {} aspect field '{}:{}' must be unique, duplicate {:?} value",
                self.entity_kind_id.0,
                self.field_locator.aspect().aspect_key().as_str(),
                self.field_locator
                    .field_path()
                    .fields()
                    .first()
                    .map_or("", |field| field.as_str()),
                value.value_family()
            ),
            fields: InvariantViolationFields::UniqueEntityField {
                entity_kind_id: self.entity_kind_id,
                field_locator: self.field_locator.clone(),
                value,
            },
        }
    }
}

/// O(J log J), where J is the applied entity-slot journal. The final image
/// already folds the coupled chains and contains their net slot changes.
pub fn collect(
    execution: &ExecutionRequest<'_, '_>,
    scope: UniqueFieldScope<'_>,
    before: &StructuralStateView<'_>,
    after: &StructuralStateView<'_>,
    working: &WorkingState,
    intents: &[MutationIntent],
    rejudged: &[RecordRef],
) -> Result<Option<UniqueFieldProbeClaims>, TransactionCommitError> {
    let scope_bytes = (intents.len().saturating_add(rejudged.len()) as u64).saturating_mul(96);
    let _scope_memory = execution.reserve_retained_memory(scope_bytes)?;
    let mut explicit = BTreeSet::new();
    for intent in intents {
        charge(execution, map_search_work(explicit.len()))?;
        if let Some(entity) = explicitly_written_entity(execution, intent, scope.field_locator)? {
            explicit.insert((entity.partition_id, entity.slot_index()));
        }
    }
    for record in rejudged {
        charge(execution, map_search_work(explicit.len()))?;
        if let RecordRef::Entity(entity) = record {
            explicit.insert((entity.partition_id, entity.slot_index()));
        }
    }
    let slots = || {
        working
            .mutation_journal()
            .iter()
            .flat_map(|(partition, journal)| {
                journal.entity_slots.iter().map(|slot| (*partition, *slot))
            })
            .chain(explicit.iter().copied().filter(|(partition, slot)| {
                !working
                    .mutation_journal()
                    .get(partition)
                    .is_some_and(|journal| journal.entity_slots.contains(slot))
            }))
    };
    let changed = || {
        slots().map(|(partition, slot)| {
            charge(execution, map_search_work(explicit.len()))?;
            let old = field_value(before, scope, partition, slot);
            let new = field_value(after, scope, partition, slot);
            if let (Some((old_id, old_value)), Some((new_id, new_value))) = (old, new)
                && old_id == new_id
            {
                let width = Key::required_encoded_capacity_bytes(old_value).ok_or(
                    WorkCeilingDenial::Stopped(MapKernelStop::WorkCounterOverflow),
                )?;
                charge(execution, width)?;
                if old_value == new_value {
                    return Ok(if explicit.contains(&(partition, slot)) {
                        (None, new)
                    } else {
                        (None, None)
                    });
                }
            }
            Ok::<_, WorkCeilingDenial>((old, new))
        })
    };
    let mut bytes = scope.field_locator.owned_allocation_capacity_bytes() as u64;
    let mut has_writer = false;
    for delta in changed() {
        let (old, new) = delta?;
        for (_, value) in old.into_iter().chain(new) {
            let encoded = Key::required_encoded_capacity_bytes(value).ok_or(
                WorkCeilingDenial::Stopped(MapKernelStop::WorkCounterOverflow),
            )?;
            charge(execution, encoded.saturating_add(1))?;
            // Includes the map node, retiring set node, and both stored
            // and diagnostic copies of the value plus the encoded key.
            bytes = bytes.saturating_add(
                encoded
                    .saturating_add(
                        (value.owned_allocation_capacity_bytes() as u64).saturating_mul(2),
                    )
                    .saturating_add(512),
            );
        }
        has_writer |= new.is_some();
    }
    if !has_writer {
        return Ok(None);
    }
    let memory = execution.reserve_retained_memory(bytes)?;
    let mut keys = BTreeMap::<Key, KeyChange>::new();
    for delta in changed() {
        let (old, new) = delta?;
        let depth = map_search_work(keys.len());
        if let Some((id, value)) = old {
            let width = Key::required_encoded_capacity_bytes(value).unwrap_or(u64::MAX);
            charge(execution, depth.saturating_mul(width.saturating_add(1)))?;
            let change = keys.entry(Key::from_aspect_value(value)).or_default();
            charge(execution, map_search_work(change.retiring.len()))?;
            change.retiring.insert(id);
        }
        if let Some((id, value)) = new {
            let width = Key::required_encoded_capacity_bytes(value).unwrap_or(u64::MAX);
            charge(execution, depth.saturating_mul(width.saturating_add(1)))?;
            let change = keys.entry(Key::from_aspect_value(value)).or_default();
            if change.writer.is_some() {
                return Err(scope.duplicate_conflict(value.clone()).into());
            }
            change.writer = Some((id, value.clone()));
        }
    }
    keys.retain(|_, change| change.writer.is_some());
    Ok(Some(UniqueFieldProbeClaims {
        entity_kind_id: scope.entity_kind_id,
        backing_index_id: scope.backing_index_id,
        field_locator: scope.field_locator.clone(),
        keys,
        _memory: memory,
    }))
}

fn map_search_work(entries: usize) -> u64 {
    u64::from(usize::BITS - entries.leading_zeros()) * 11 + 1
}

fn explicitly_written_entity(
    execution: &ExecutionRequest<'_, '_>,
    intent: &MutationIntent,
    locator: &AspectFieldLocator,
) -> Result<Option<EntityId>, WorkCeilingDenial> {
    let MutationIntent::Entity(intent) = intent else {
        return Ok(None);
    };
    Ok(match intent {
        EntityMutationIntent::UpdateFields(write) => {
            charge(execution, map_search_work(write.fields.len()))?;
            write.fields.get(locator).map(|_| write.entity_id)
        }
        EntityMutationIntent::ApplyAspectPatch(write) => {
            let mut writes = false;
            for operation in write.aspect_patch.operations() {
                charge(execution, 1)?;
                if operation.basis().key() != locator.aspect().aspect_key() {
                    continue;
                }
                match operation {
                    PortableAspectPatchOperation::SetWhole { .. } => writes = true,
                    PortableAspectPatchOperation::PatchFields { field_sets, .. } => {
                        charge(execution, field_sets.len() as u64)?;
                        if let [field] = locator.field_path().fields() {
                            writes |= field_sets.iter().any(|set| set.field() == field);
                        }
                    }
                    PortableAspectPatchOperation::ClearWhole { .. } => {}
                }
            }
            writes.then_some(write.entity_id)
        }
        EntityMutationIntent::Revalidate(_)
        | EntityMutationIntent::Replace(_)
        | EntityMutationIntent::Delete(_) => None,
    })
}

fn field_value<'a>(
    view: &StructuralStateView<'a>,
    scope: UniqueFieldScope<'_>,
    partition: PartitionId,
    slot: usize,
) -> Option<(EntityId, &'a AspectValue)> {
    let metadata = view.entity_metadata_for_slot(partition, slot)?;
    if metadata.kind_id != scope.entity_kind_id {
        return None;
    }
    let value = authoritative_state_query_locus_value(
        view.entity_aspect_state(metadata.entity_id),
        scope.field_locator,
    )?;
    Some((metadata.entity_id, value))
}

pub fn charge(execution: &ExecutionRequest<'_, '_>, units: u64) -> Result<(), WorkCeilingDenial> {
    execution
        .run_native_with_checkpoint(|context| {
            crate::contracts::execution::request_stage::checkpoint(context, units)
        })?
        .0
        .map_err(WorkCeilingDenial::Stopped)
}

#[derive(Debug, Default)]
pub struct UniqueFieldProbeBatch {
    claims: Vec<UniqueFieldProbeClaims>,
    definition_frontier: Option<u64>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl UniqueFieldProbeBatch {
    pub fn reserve(
        execution: &ExecutionRequest<'_, '_>,
        count: usize,
        definition_frontier: u64,
    ) -> Result<Self, WorkCeilingDenial> {
        let bytes = count
            .checked_mul(size_of::<UniqueFieldProbeClaims>())
            .ok_or(WorkCeilingDenial::Stopped(
                MapKernelStop::WorkCounterOverflow,
            ))?;
        let memory = execution.reserve_retained_memory(bytes as u64)?;
        Ok(Self {
            claims: Vec::with_capacity(count),
            definition_frontier: Some(definition_frontier),
            _memory: memory,
        })
    }

    pub fn push(&mut self, claims: UniqueFieldProbeClaims) {
        debug_assert!(self.claims.len() < self.claims.capacity());
        self.claims.push(claims);
    }

    pub fn claims(&self) -> &[UniqueFieldProbeClaims] {
        &self.claims
    }

    pub fn definition_frontier(&self) -> Option<u64> {
        self.definition_frontier
    }
}
