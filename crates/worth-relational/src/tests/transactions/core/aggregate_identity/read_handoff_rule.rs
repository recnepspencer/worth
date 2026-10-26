//! Installed Native rules retain real aggregate reads across preparation and seal.
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::aggregate_identity::{AggregateRead, AggregateReadDenial};
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::*;
use worth_execution::{CancellationSource, ExecutionMemoryReservation, SerialMemoryBudget};

#[derive(Clone, Copy)]
pub(super) enum ReadMode {
    Proposed,
    Both,
    Committed,
}

#[derive(Default)]
pub(super) struct Control {
    pub enabled: AtomicBool,
    pub roots: Mutex<Vec<EntityId>>,
    pub captures: Mutex<Vec<Capture>>,
    pub pressure: Mutex<Option<SerialMemoryBudget>>,
    pub held: Mutex<Option<ExecutionMemoryReservation>>,
    pub cancel: Mutex<Option<CancellationSource>>,
}

pub(super) struct Capture {
    pub rule: usize,
    pub evaluation: bool,
    pub proposed: Vec<AggregateRead>,
    pub committed: Vec<AggregateRead>,
    pub errors: Vec<AggregateReadDenial>,
}

struct ReaderRule {
    id: usize,
    mode: ReadMode,
    maximum: Option<NonZeroU64>,
    control: Arc<Control>,
    maintenance: bool,
}

pub(super) fn registration(
    id: usize,
    mode: ReadMode,
    maximum: Option<NonZeroU64>,
    control: &Arc<Control>,
) -> CustomInvariantRegistration {
    CustomInvariantRegistration::new(ReaderRule {
        id,
        mode,
        maximum,
        control: Arc::clone(control),
        maintenance: false,
    })
    .unwrap()
}

pub(super) fn maintenance_registration(control: &Arc<Control>) -> CustomInvariantRegistration {
    CustomInvariantRegistration::new(ReaderRule {
        id: 0,
        mode: ReadMode::Both,
        maximum: None,
        control: Arc::clone(control),
        maintenance: true,
    })
    .unwrap()
}

impl Control {
    pub fn arm(&self, roots: Vec<EntityId>) {
        self.captures.lock().unwrap().clear();
        *self.roots.lock().unwrap() = roots;
        self.enabled.store(true, Ordering::SeqCst);
    }
}

impl ReaderRule {
    fn observe(
        &self,
        proposed: StructuralRelationView<'_>,
        committed: StructuralRelationView<'_>,
        fallback: &[EntityId],
        evaluation: bool,
    ) -> Vec<AggregateRead> {
        if !self.control.enabled.load(Ordering::SeqCst) {
            return Vec::new();
        }
        let selected = self.control.roots.lock().unwrap();
        let roots = if selected.is_empty() {
            fallback
        } else {
            &selected
        };
        let mut capture = Capture {
            rule: self.id,
            evaluation,
            proposed: Vec::new(),
            committed: Vec::new(),
            errors: Vec::new(),
        };
        // The request owner deliberately occupies its own original ledger. No
        // replacement request or synthetic preparation error enters this court.
        if let Some(memory) = self.control.pressure.lock().unwrap().as_ref() {
            let mut held = self.control.held.lock().unwrap();
            if held.is_none() {
                *held = Some(
                    memory
                        .reserve(memory.limit().unwrap() - memory.charged_memory_bytes())
                        .unwrap(),
                );
            }
        }
        for &root in roots {
            if !matches!(self.mode, ReadMode::Committed) {
                match proposed.aggregates().of_source_relation(KindId(2), root) {
                    Ok(read) => capture.proposed.push(read),
                    Err(error) => capture.errors.push(error),
                }
            }
            if !matches!(self.mode, ReadMode::Proposed) {
                match committed.aggregates().of_source_relation(KindId(2), root) {
                    Ok(read) => capture.committed.push(read),
                    Err(error) => capture.errors.push(error),
                }
            }
        }
        let scope = capture.proposed.clone();
        let succeeded = !capture.proposed.is_empty() || !capture.committed.is_empty();
        self.control.captures.lock().unwrap().push(capture);
        if succeeded {
            if let Some(cancel) = self.control.cancel.lock().unwrap().as_ref() {
                cancel.cancel();
            }
        }
        scope
    }
}

impl CustomInvariantRule for ReaderRule {
    type Scope = Vec<AggregateRead>;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(format!("test.aggregate-handoff.{}", self.id)),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Native aggregate reader"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: self.maximum,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: if self.maintenance {
                        vec![KindId(1), KindId(3)]
                    } else {
                        vec![KindId(1)]
                    },
                    affected_relation_kinds: if self.maintenance {
                        vec![KindId(2), KindId(5)]
                    } else {
                        vec![KindId(2)]
                    },
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
        Ok(self.observe(
            planner.relations(),
            planner.committed_relations(),
            planner.touched().visible_entity_ids(),
            false,
        ))
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let roots = scope.iter().map(AggregateRead::anchor).collect::<Vec<_>>();
        self.observe(
            context.relations(),
            context.committed_relations(),
            &roots,
            true,
        );
        // Deliberately swallow read errors. Native typed failures must survive
        // this successful verdict at the real invariant coordinator boundary.
        Ok(CustomInvariantVerdict::Pass)
    }
}
