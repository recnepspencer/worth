//! Real RelationalPublicationCompanion + trigger invariant driving PersistentSourceRows.
use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
use worth_execution::ExecutionRequest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::managed_index::source::PersistentSourceRows;
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, KindId},
    mvcc::{
        CompanionBranchCellSlot, CompanionPreflightStop, PreparedPublicationCompanionEffect,
        PublicationCompanionPreflight, RelationalPublicationCompanion, ReservedCompanionBranchCell,
    },
    runtime::{
        CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
        CustomInvariantExecutionError, CustomInvariantOperationalMetadata,
        CustomInvariantPreparationError, CustomInvariantRule, CustomInvariantRuleId,
        CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
        CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
        InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
        ManagedIndexBranchImage, ManagedIndexSourcePreparation,
        ManagedIndexSourcePreparationDenial, ManagedIndexSourceProgram, ManagedIndexSourceScope,
        PreparedManagedIndexSourceRead, PreparedManagedIndexSourceRoot,
    },
};

pub(super) enum ProbeMode {
    DropFromOccurrences(EntityId),
    DropRoutedRowAndKey(EntityId),
    MissingRecordDelta,
}

#[derive(Default)]
pub(super) struct ObservedState {
    pub rows: BTreeMap<EntityId, Arc<PersistentSourceRows>>,
    pub last_changed_entities: BTreeSet<EntityId>,
    pub last_changed_occurrence_records: Option<Vec<EntityId>>,
    pub last_dependency_entity_count: usize,
    pub last_delta: (Vec<EntityId>, Vec<EntityId>),
    pub probe: Option<ProbeMode>,
    pub last_probe: Option<Result<(), ManagedIndexSourcePreparationDenial>>,
    source_cells: BTreeMap<BranchId, CompanionBranchCellSlot<ManagedIndexBranchImage>>,
}

pub(super) struct Companion {
    pub state: Arc<Mutex<ObservedState>>,
    pub graph: WorthQueryInstalledGraphReadContract,
}
impl std::fmt::Debug for Companion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SelfRefOccurrenceCompanion")
    }
}
impl Companion {
    fn reserve_source(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<ReservedCompanionBranchCell<ManagedIndexBranchImage>, CompanionPreflightStop> {
        let existing = self
            .state
            .lock()
            .unwrap()
            .source_cells
            .get(context.branch_id())
            .cloned();
        if let Some(slot) = existing {
            if let Some(cell) = slot.admitted() {
                return cell.reserve_preflight(context);
            }
            if let Some(cell) = slot.pending_for_preflight(context)? {
                return cell.reserve_preflight(context);
            }
        }
        let cell = context.mint_managed_index_source_cell()?;
        self.state
            .lock()
            .unwrap()
            .source_cells
            .insert(context.branch_id().clone(), cell.lookup_slot());
        cell.reserve_preflight(context)
    }

    fn prepare_scope(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: ManagedIndexSourceScope<'_>,
    ) -> Result<PreparedManagedIndexSourceRoot, ManagedIndexSourcePreparationDenial> {
        let mut state = self.state.lock().unwrap();
        state.last_changed_entities = scope.changed_entities().collect();
        state.last_changed_occurrence_records = scope
            .changed_occurrence_record_entities()
            .map(|ids| ids.to_vec());
        if let Some(prev) = state.rows.get(&scope.scope()).cloned() {
            state.last_dependency_entity_count = prev.dependency_entity_count_for_test();
            if let Some(probe) = state.probe.take() {
                state.last_probe = Some(match probe {
                    ProbeMode::DropFromOccurrences(drop_id) => {
                        let filtered: Vec<_> = scope
                            .prepare_changed_occurrences()?
                            .into_iter()
                            .filter(|(before, after)| {
                                before.as_ref().map(|r| r.entity()) != Some(drop_id)
                                    && after.as_ref().map(|r| r.entity()) != Some(drop_id)
                            })
                            .collect();
                        prev.update_for_test(execution, &scope, &filtered, &self.graph, false)
                            .map(|_| ())
                    }
                    ProbeMode::DropRoutedRowAndKey(drop_id) => {
                        let changed_keys: BTreeSet<EntityId> = scope
                            .prepare_changed_occurrences()?
                            .into_iter()
                            .flat_map(|(before, after)| {
                                before
                                    .map(|r| r.entity())
                                    .into_iter()
                                    .chain(after.map(|r| r.entity()))
                            })
                            .filter(|entity| *entity != drop_id)
                            .collect();
                        prev.check_complete_with_override_for_test(
                            &scope,
                            Some(drop_id),
                            None,
                            &changed_keys,
                        )
                    }
                    ProbeMode::MissingRecordDelta => {
                        let changed_keys: BTreeSet<EntityId> = scope
                            .prepare_changed_occurrences()?
                            .into_iter()
                            .flat_map(|(before, after)| {
                                before
                                    .map(|r| r.entity())
                                    .into_iter()
                                    .chain(after.map(|r| r.entity()))
                            })
                            .collect();
                        prev.check_complete_with_override_for_test(
                            &scope,
                            None,
                            Some(None),
                            &changed_keys,
                        )
                    }
                });
            }
            let changed = scope.prepare_changed_occurrences()?;
            let (next, delta) =
                prev.update_for_test(execution, &scope, &changed, &self.graph, false)?;
            state.last_delta = (delta.side(true).to_vec(), delta.side(false).to_vec());
            let next = Arc::new(next);
            state.rows.insert(scope.scope(), Arc::clone(&next));
            Ok(scope.seal(
                Some(prev as Arc<dyn Any + Send + Sync>),
                Some(next as Arc<dyn Any + Send + Sync>),
                delta,
            ))
        } else {
            let rows = PersistentSourceRows::acquire_for_test(execution, &scope, &self.graph, 64)?;
            let rows = Arc::new(rows);
            state.rows.insert(scope.scope(), Arc::clone(&rows));
            Ok(scope.seal(
                None,
                Some(rows as Arc<dyn Any + Send + Sync>),
                Default::default(),
            ))
        }
    }
}
impl RelationalPublicationCompanion for Companion {
    fn prepare_managed_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, ManagedIndexSourcePreparationDenial> {
        source
            .scopes()
            .map(|scope| self.prepare_scope(execution, scope))
            .collect()
    }

    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let mark = context
            .mint_selected_branch_cell(Arc::new(()))?
            .reserve_preflight(context)?;
        let source = self.reserve_source(context)?;
        let image = context.prepare_managed_source_image(&source)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, image)
    }
}

pub(super) struct SourceTriggerRule {
    pub program: ManagedIndexSourceProgram,
    pub entity_kinds: Vec<KindId>,
    pub relation_kinds: Vec<KindId>,
}
impl CustomInvariantRule for SourceTriggerRule {
    type Scope = Arc<PreparedManagedIndexSourceRead>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("self-ref-occurrence-test"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Self-referential Source occurrence test".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: self.entity_kinds.clone(),
                    read_relation_kinds: self.relation_kinds.clone(),
                    affected_entity_kinds: self.entity_kinds.clone(),
                    affected_relation_kinds: self.relation_kinds.clone(),
                    include_relation_endpoint_entity_touches: true,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(Arc::new(
            planner
                .prepare_managed_index_source(&self.program)
                .map_err(|cause| CustomInvariantPreparationError::new(format!("{cause:?}")))?,
        ))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let _ = context
            .read_managed_index_source(scope)
            .map_err(|cause| CustomInvariantExecutionError::new(format!("{cause:?}")))?;
        Ok(CustomInvariantVerdict::Pass)
    }
}
