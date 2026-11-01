//! Real Native preparation/observation receivers; no application read is synthesized.
use super::*;
use worth_relational::facade::mvcc::{
    CompanionBranchCellSlot, CompanionPreflightStop, PreparedPublicationCompanionEffect,
    PublicationCompanionPreflight, RelationalPublicationCompanion, ReservedCompanionBranchCell,
};

#[derive(Default)]
pub(crate) struct Court {
    pub(crate) opaque_roots: Mutex<BTreeMap<EntityId, Arc<dyn std::any::Any + Send + Sync>>>,
    pub(crate) source_cells:
        Mutex<BTreeMap<BranchId, CompanionBranchCellSlot<ManagedIndexBranchImage>>>,
    pub staged: Mutex<BTreeSet<EntityId>>,
    pub observed: Mutex<BTreeSet<EntityId>>,
    pub expected: Mutex<BTreeMap<EntityId, (Vec<EntityId>, Vec<EntityId>)>>,
    previous: Mutex<Option<Arc<PreparedManagedIndexSourceRead>>>,
    pub foreign_observation: AtomicUsize,
    pub checking_previous_proposal: std::sync::atomic::AtomicBool,
}

pub(crate) struct SourceOwner(pub Arc<Court>);
impl std::fmt::Debug for SourceOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeSourceCourtOwner")
    }
}

fn rows(
    source: ManagedIndexSourceRows<'_>,
) -> Result<Vec<EntityId>, ManagedIndexSourcePreparationDenial> {
    let read = source.acquire_all()?;
    Ok(source
        .occurrences(&read)?
        .iter()
        .map(|occurrence| occurrence.entity())
        .collect())
}

impl SourceOwner {
    fn reserve_source(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<ReservedCompanionBranchCell<ManagedIndexBranchImage>, CompanionPreflightStop> {
        let existing = self
            .0
            .source_cells
            .lock()
            .unwrap()
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
        self.0
            .source_cells
            .lock()
            .unwrap()
            .insert(context.branch_id().clone(), cell.lookup_slot());
        cell.reserve_preflight(context)
    }
}

struct ForkLookup {
    court: Arc<Court>,
    branch: BranchId,
    slot: CompanionBranchCellSlot<ManagedIndexBranchImage>,
}
impl Drop for ForkLookup {
    fn drop(&mut self) {
        if self.slot.admitted().is_some() {
            return;
        }
        let mut slots = self.court.source_cells.lock().unwrap();
        if slots
            .get(&self.branch)
            .is_some_and(|slot| slot.same_registration(&self.slot))
        {
            slots.remove(&self.branch);
        }
    }
}

impl RelationalPublicationCompanion for SourceOwner {
    fn prepare_managed_source_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &worth_relational::facade::mvcc::ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, CompanionPreflightStop> {
        execution
            .consult()
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        let branch = target.target_branch().clone();
        let slot = target.lookup_slot();
        self.0
            .source_cells
            .lock()
            .unwrap()
            .insert(branch.clone(), slot.clone());
        Ok(Box::new(ForkLookup {
            court: self.0.clone(),
            branch,
            slot,
        }))
    }

    fn prepare_managed_source(
        &self,
        _execution: &ExecutionRequest<'_, '_>,
        source: &ManagedIndexSourcePreparation<'_>,
    ) -> Result<Vec<PreparedManagedIndexSourceRoot>, ManagedIndexSourcePreparationDenial> {
        let mut roots = Vec::new();
        for scope in source.scopes() {
            let committed = if scope.committed_scope_exists()? {
                Some(Arc::new(rows(scope.committed()?)?) as Arc<dyn std::any::Any + Send + Sync>)
            } else {
                None
            };
            let proposed = if scope.scope_exists()? {
                let actual_rows = rows(scope.proposed())?;
                let opaque = self
                    .0
                    .opaque_roots
                    .lock()
                    .unwrap()
                    .get(&scope.scope())
                    .cloned();
                Some(opaque.unwrap_or_else(|| Arc::new(actual_rows)))
            } else {
                None
            };
            self.0.staged.lock().unwrap().insert(scope.scope());
            roots.push(scope.seal(committed, proposed));
        }
        Ok(roots)
    }

    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        // The same Native court now retains its real managed image, allowing
        // history material/affinity to be judged through actual observations.
        // The payload remains Native membership rows, not a Query World root.
        let mark = context
            .mint_selected_branch_cell(Arc::new(()))?
            .reserve_preflight(context)?;
        let source = self.reserve_source(context)?;
        let image = context.prepare_managed_source_image(&source)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, image)
    }
}

pub(crate) struct Reader {
    pub maximum_work: Option<std::num::NonZeroU64>,
    pub program: ManagedIndexSourceProgram,
    pub court: Arc<Court>,
}

impl CustomInvariantRule for Reader {
    type Scope = Arc<PreparedManagedIndexSourceRead>;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("native-managed-source-scopes"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Complete Native Source scope court".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: self.maximum_work,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(4)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: vec![KindId(1), KindId(4)],
                    affected_relation_kinds: vec![KindId(2)],
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
        let read = context
            .read_managed_index_source(scope)
            .expect("same actual observation and installed audience");
        let expected = self.court.expected.lock().unwrap();
        for key in expected.keys() {
            assert!(
                self.court.staged.lock().unwrap().contains(key),
                "ALL roots precede the FIRST rule callback"
            );
        }
        if let Some((before, after)) = expected.get(&scope.scope()) {
            assert_eq!(&rows(read.committed().unwrap()).unwrap(), before);
            assert_eq!(&rows(read.proposed()).unwrap(), after);
            assert_eq!(
                read.committed_root()
                    .unwrap()
                    .downcast_ref::<Vec<EntityId>>()
                    .unwrap(),
                before
            );
            assert_eq!(
                read.proposed_root()
                    .unwrap()
                    .downcast_ref::<Vec<EntityId>>()
                    .unwrap(),
                after
            );
        }
        self.court.observed.lock().unwrap().insert(scope.scope());
        let mut previous = self.court.previous.lock().unwrap();
        if !self.court.checking_previous_proposal.load(Ordering::SeqCst) {
            if previous.is_none() {
                *previous = Some(scope.clone());
            }
        } else if previous
            .as_ref()
            .is_some_and(|old| old.scope() == scope.scope())
        {
            if let Some(old) = previous.as_ref() {
                assert!(matches!(
                    context.read_managed_index_source(old),
                    Err(ManagedIndexSourcePreparationDenial::Source(
                        DerivedRelationReadDenial::Definition(
                            DerivedRelationDenial::ForeignObservation
                        )
                    ))
                ));
                self.court
                    .foreign_observation
                    .fetch_add(1, Ordering::SeqCst);
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
