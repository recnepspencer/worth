//! Genuine Native prepared reads over permissive physical histories.
use super::*;
#[derive(Clone, Default)]
pub(in super::super::super) struct Control {
    pub request: Arc<Mutex<Option<(EntityId, bool)>>>,
    pub results: Arc<Mutex<Vec<Result<Vec<EntityId>, DerivedRelationReadDenial>>>>,
    pub cancel_after_read: Arc<Mutex<Option<CancellationSource>>>,
}
pub(in super::super::super) struct Reader {
    pub maximum_work: Option<std::num::NonZeroU64>,
    pub path: DerivedRelationHandle,
    pub control: Control,
}
impl CustomInvariantRule for Reader {
    type Scope = Option<PreparedDerivedRelationRead>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("native-ordered-event-reader"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Actual ordered-event Native reader".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: self.maximum_work,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(4)],
                    read_relation_kinds: vec![KindId(2), KindId(3), KindId(5)],
                    affected_entity_kinds: vec![KindId(1), KindId(4)],
                    affected_relation_kinds: vec![KindId(2), KindId(3), KindId(5)],
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
        let Some((anchor, reverse)) = *self.control.request.lock().unwrap() else {
            return Ok(None);
        };
        let view = planner.relations();
        match view.prepare_derived_source(
            &self.path,
            anchor,
            DerivedRelationReadShape::Adjacency { reverse },
        ) {
            Ok(read) => {
                let data = view
                    .read_prepared_derived_source(
                        &read,
                        DerivedRelationReadShape::Adjacency { reverse },
                    )
                    .unwrap();
                self.control.results.lock().unwrap().push(Ok(data
                    .endpoints()
                    .iter()
                    .map(|key| key.entity().unwrap())
                    .collect()));
                // Terminal predecessor and empty intro/retire rosters must all
                // stay in the same genuine prepared read's physical support.
                assert!(!data.adjacency_dependencies().is_empty());
                if !reverse {
                    for &(event, kind) in data.entity_dependencies() {
                        if kind != KindId(1) {
                            continue;
                        }
                        for relation in [KindId(2), KindId(3), KindId(5)] {
                            assert!(
                                data.adjacency_dependencies()
                                    .iter()
                                    .any(|probe| probe.anchor == event
                                        && probe.relation == relation
                                        && probe.outgoing),
                                "every chain event retains all three complete leaf rosters"
                            );
                        }
                    }
                }
                if let Some(cancel) = self.control.cancel_after_read.lock().unwrap().take() {
                    cancel.cancel();
                    let cause = view
                        .prepare_derived_source(
                            &self.path,
                            anchor,
                            DerivedRelationReadShape::Adjacency { reverse },
                        )
                        .err()
                        .unwrap();
                    self.control.results.lock().unwrap().push(Err(cause));
                }
                Ok(Some(read))
            }
            Err(cause) => {
                self.control.results.lock().unwrap().push(Err(cause));
                Ok(None)
            }
        }
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if let Some(read) = scope {
            let reverse = self.control.request.lock().unwrap().unwrap().1;
            context
                .relations()
                .read_prepared_derived_source(read, DerivedRelationReadShape::Adjacency { reverse })
                .unwrap();
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
pub(super) fn ask(
    runtime: &RelationalRuntime,
    control: &Control,
    execution: &ExecutionRequest<'_, '_>,
    anchor: EntityId,
    reverse: bool,
) -> Result<Vec<EntityId>, DerivedRelationReadDenial> {
    ask_via(runtime, control, execution, anchor, reverse, anchor)
}
pub(super) fn ask_via(
    runtime: &RelationalRuntime,
    control: &Control,
    execution: &ExecutionRequest<'_, '_>,
    anchor: EntityId,
    reverse: bool,
    touched: EntityId,
) -> Result<Vec<EntityId>, DerivedRelationReadDenial> {
    *control.request.lock().unwrap() = Some((anchor, reverse));
    control.results.lock().unwrap().clear();
    commit(
        runtime,
        execution,
        WorkerIntentBatch::new("probe-event-read").push(MutationIntent::Entity(
            EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id: touched }),
        )),
    );
    let results = control.results.lock().unwrap();
    assert!(!results.is_empty(), "genuine Native read callback ran");
    assert!(results.iter().all(|result| result == &results[0]));
    results[0].clone()
}
