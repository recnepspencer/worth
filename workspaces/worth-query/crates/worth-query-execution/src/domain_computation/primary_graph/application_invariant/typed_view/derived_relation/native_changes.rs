//! Real rewiring proves paired pullback, frozen reborrow and audience refusal.
use super::native_fixture::*;
use std::collections::BTreeSet;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::*, runtime::*, schema::*, symbols::ClientKey, transactions::*,
};

#[derive(Default)]
struct Control {
    stage: AtomicUsize,
    members: Mutex<Vec<EntityId>>,
    previous: Mutex<Option<PreparedDerivedRelationChanges>>,
    evaluated: AtomicUsize,
    underdeclared_prepared: AtomicUsize,
}
struct Reader {
    definition: DerivedRelationHandle,
    narrow: bool,
    control: Arc<Control>,
}
impl CustomInvariantRule for Reader {
    type Scope = Option<PreparedDerivedRelationChanges>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(if self.narrow {
                    "narrow-candidate-changes"
                } else {
                    "broad-candidate-changes"
                }),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Actual candidate change source"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: vec![KindId(if self.narrow { 3 } else { 2 })],
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: vec![KindId(2), KindId(3)],
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
        if self.control.stage.load(Ordering::SeqCst) == 0 {
            return Ok(None);
        }
        Ok(Some(
            planner
                .prepare_derived_relation_changes(&self.definition)
                .unwrap(),
        ))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let Some(proof) = scope else {
            return Ok(CustomInvariantVerdict::Pass);
        };
        let data = context
            .read_prepared_derived_relation_changes(proof)
            .unwrap();
        if self.narrow {
            assert!(
                data.affected_roots().is_empty(),
                "unmodified kind3 source remains empty"
            );
            let previous = self.control.previous.lock().unwrap();
            assert_eq!(
                context
                    .read_prepared_derived_relation_changes(previous.as_ref().unwrap())
                    .err(),
                Some(DerivedRelationReadDenial::Structural(
                    StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess)
                ))
            );
        } else {
            let stage = self.control.stage.load(Ordering::SeqCst);
            let members = self.control.members.lock().unwrap();
            if stage == 1 {
                // Authored edges are 0->1, 1->2, 3->4. Rewiring 1->3 can
                // affect two-step roots0/1; 2 has a genuinely empty suffix.
                assert_eq!(
                    data.affected_roots(),
                    &BTreeSet::from([members[0].into(), members[1].into()])
                );
                assert!(data
                    .support()
                    .iter()
                    .any(|support| support.endpoints().is_empty()
                        && support
                            .adjacency_dependencies()
                            .iter()
                            .any(|read| read.anchor == members[2])));
            } else {
                assert!(
                    data.affected_roots().is_empty(),
                    "revalidation makes no topology delta"
                );
                let previous = self.control.previous.lock().unwrap();
                assert_eq!(
                    context
                        .read_prepared_derived_relation_changes(previous.as_ref().unwrap())
                        .err(),
                    Some(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    ))
                );
            }
            *self.control.previous.lock().unwrap() = Some(proof.clone());
        }
        self.control.evaluated.fetch_add(1, Ordering::SeqCst);
        Ok(CustomInvariantVerdict::Pass)
    }
}

struct UnderdeclaredAffected(Reader);
impl CustomInvariantRule for UnderdeclaredAffected {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = self.0.descriptor();
        descriptor.identity.rule_id =
            CustomInvariantRuleId::new("underdeclared-affected-candidate-changes");
        // Keep the broad reader's complete read audience, but omit the
        // composed recipe's physical kind2 from affected declarations.
        descriptor.operational.access.affected_relation_kinds = vec![KindId(3)];
        descriptor
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        if self.0.control.stage.load(Ordering::SeqCst) != 2 {
            return Ok(());
        }
        let root = self.0.control.members.lock().unwrap()[0];
        // The actual kind1 revalidation schedules this installed rule even
        // though kind2 relation changes are excluded from its touched set.
        assert!(planner
            .touched()
            .direct_visible_entity_ids()
            .contains(&root));
        for view in [planner.committed_relations(), planner.relations()] {
            view.prepare_derived_relations_from(&self.0.definition, root)
                .expect("the complete recipe is readable from both Native views");
        }
        assert_eq!(
            planner
                .prepare_derived_relation_changes(&self.0.definition)
                .err(),
            Some(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess)
            ))
        );
        self.0
            .control
            .underdeclared_prepared
            .fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn candidate_changes_retain_both_pullbacks_empty_support_and_exact_proposal_audience() {
    let mut registry = registry();
    let composed = DerivedRelationIdentity {
        name: "two-step-changes".into(),
        ..identity(2)
    };
    registry = registry
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: composed.clone(),
            from: KindId(1).into(),
            to: KindId(1).into(),
            nodes: vec![
                DerivedRelationNode::Stored {
                    relation: KindId(2),
                    from: KindId(1),
                    to: KindId(1),
                },
                DerivedRelationNode::Compose { left: 0, right: 0 },
            ],
            laws: None,
        }])
        .unwrap();
    let control = Arc::new(Control::default());
    let mut builder = RelationalRuntimeApi::builder().schema_registry(registry.clone());
    for narrow in [false, true] {
        let source = if narrow {
            identity(3)
        } else {
            composed.clone()
        };
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(Reader {
                definition: registry.derived_relation(&source).unwrap(),
                narrow,
                control: control.clone(),
            })
            .unwrap(),
        );
    }
    builder = builder.custom_invariant(
        CustomInvariantRegistration::new(UnderdeclaredAffected(Reader {
            definition: registry.derived_relation(&composed).unwrap(),
            narrow: false,
            control: control.clone(),
        }))
        .unwrap(),
    );
    let runtime = builder.build().expect("valid native candidate fixture");
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let mut birth = WorkerIntentBatch::new("candidate-changes-birth");
    for i in 0..5 {
        birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: created(i).client_key,
            fields: Default::default(),
        })));
    }
    for (i, (a, b)) in [(0, 1), (1, 2), (3, 4)].into_iter().enumerate() {
        birth = birth.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(format!("change-edge-{i}")),
                source: EntityReference::Created(created(a)),
                target: EntityReference::Created(created(b)),
                fields: Default::default(),
            },
        )));
    }
    let born = commit(&runtime, &execution, birth);
    let members: Vec<_> = (0..5)
        .map(|i| born.created_entity(&created(i)).unwrap())
        .collect();
    let changed = born
        .created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("change-edge-1"),
            source: EntityReference::Created(created(1)),
            target: EntityReference::Created(created(2)),
        })
        .unwrap();
    *control.members.lock().unwrap() = members.clone();
    control.stage.store(1, Ordering::SeqCst);
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("candidate-source-rewire").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: changed,
                kind_id: KindId(2),
                source: EntityReference::Existing(members[1]),
                target: EntityReference::Existing(members[3]),
            }),
        )),
    );
    let first = control.evaluated.load(Ordering::SeqCst);
    assert!(
        first >= 2,
        "both installed rules evaluated the real candidate"
    );
    let underdeclared_before = control.underdeclared_prepared.load(Ordering::SeqCst);
    control.stage.store(2, Ordering::SeqCst);
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("candidate-source-next-proposal").push(MutationIntent::Entity(
            EntityMutationIntent::Revalidate(RevalidateEntityIntent {
                entity_id: members[0],
            }),
        )),
    );
    assert!(control.evaluated.load(Ordering::SeqCst) > first);
    assert!(
        control.underdeclared_prepared.load(Ordering::SeqCst) > underdeclared_before,
        "the installed underdeclared rule reached the exact preparation refusal"
    );
}
