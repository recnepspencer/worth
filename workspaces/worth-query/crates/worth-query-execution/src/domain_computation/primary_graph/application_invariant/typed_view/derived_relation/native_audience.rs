//! Real Native rules cannot lend a broad source proof to a narrower audience.
use super::native_fixture::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    runtime::*,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

#[derive(Default)]
struct Control {
    root: Mutex<Option<EntityId>>,
    expected: Mutex<Option<EntityId>>,
    broad: Mutex<Option<PreparedDerivedRelationRead>>,
    prepared: AtomicBool,
    evaluated: AtomicBool,
}
struct Reader {
    narrow: bool,
    own: DerivedRelationHandle,
    foreign: DerivedRelationHandle,
    control: Arc<Control>,
}
impl Reader {
    fn check_broad(&self, view: &StructuralRelationView<'_>) {
        let broad = self.control.broad.lock().unwrap();
        let proof = broad
            .as_ref()
            .expect("first registered rule prepared this genuine candidate");
        assert_eq!(
            view.read_prepared_derived_relations(proof).unwrap_err(),
            DerivedRelationReadDenial::Structural(StructuralWalkDenialKind::Read(
                StructuralReadError::OutsideDeclaredAccess,
            )),
            "the exact cause must be audience refusal, not a foreign observation"
        );
    }
    fn own(
        &self,
        view: &StructuralRelationView<'_>,
        root: EntityId,
    ) -> PreparedDerivedRelationRead {
        assert_eq!(
            view.prepare_derived_relations_from(&self.foreign, root)
                .err(),
            Some(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::ForeignDefinition
            ))
        );
        let proof = view
            .prepare_derived_relations_from(&self.own, root)
            .unwrap();
        if self.narrow {
            assert_eq!(
                view.read_prepared_derived_relations(&proof)
                    .unwrap()
                    .endpoints(),
                [self.control.expected.lock().unwrap().unwrap()]
            );
        }
        proof
    }
}
impl CustomInvariantRule for Reader {
    type Scope = Option<PreparedDerivedRelationRead>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(if self.narrow {
                    "narrow-source-consumer"
                } else {
                    "broad-source-producer"
                }),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Exact derived source audience"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId::new(1)],
                    read_relation_kinds: vec![KindId::new(if self.narrow { 3 } else { 2 })],
                    affected_entity_kinds: vec![KindId::new(1)],
                    affected_relation_kinds: vec![],
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
        let Some(root) = *self.control.root.lock().unwrap() else {
            return Ok(None);
        };
        let view = planner.relations();
        if self.narrow {
            self.check_broad(&view);
        }
        let proof = self.own(&view, root);
        if self.narrow {
            self.control.prepared.store(true, Ordering::SeqCst);
        } else {
            *self.control.broad.lock().unwrap() = Some(proof.clone());
        }
        Ok(Some(proof))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let Some(proof) = scope else {
            return Ok(CustomInvariantVerdict::Pass);
        };
        let view = context.relations();
        if self.narrow {
            self.check_broad(&view);
            assert_eq!(
                view.read_prepared_derived_relations(proof)
                    .unwrap()
                    .endpoints(),
                [self.control.expected.lock().unwrap().unwrap()]
            );
            self.control.evaluated.store(true, Ordering::SeqCst);
        } else {
            view.read_prepared_derived_relations(proof).unwrap();
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
#[test]
fn exact_observation_proof_refuses_a_narrow_audience_while_its_own_source_succeeds() {
    let schema = registry();
    let foreign = registry();
    let control = Arc::new(Control::default());
    let mut builder = RelationalRuntimeApi::builder().schema_registry(schema.clone());
    for narrow in [false, true] {
        let kind = if narrow { 3 } else { 2 };
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(Reader {
                narrow,
                own: schema.derived_relation(&identity(kind)).unwrap(),
                foreign: foreign.derived_relation(&identity(kind)).unwrap(),
                control: Arc::clone(&control),
            })
            .unwrap(),
        );
    }
    let runtime = builder.build();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let mut birth = WorkerIntentBatch::new("audience-birth");
    for index in 0..3 {
        birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId::new(1),
            client_key: created(index).client_key,
            fields: Default::default(),
        })));
    }
    for (kind, target) in [(2, 1), (3, 2)] {
        birth = birth.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId::new(kind),
                client_key: ClientKey::raw(format!("edge-{kind}")),
                source: EntityReference::Created(created(0)),
                target: EntityReference::Created(created(target)),
                fields: Default::default(),
            },
        )));
    }
    let born = commit(&runtime, &execution, birth);
    *control.root.lock().unwrap() = Some(born.created_entity(&created(0)).unwrap());
    *control.expected.lock().unwrap() = Some(born.created_entity(&created(2)).unwrap());
    let root = control.root.lock().unwrap().unwrap();
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("audience-revalidation").push(MutationIntent::Entity(
            EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id: root }),
        )),
    );
    assert!(control.prepared.load(Ordering::SeqCst));
    assert!(control.evaluated.load(Ordering::SeqCst));
}
