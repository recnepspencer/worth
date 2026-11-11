//! A real later commit reads the full-generation ID retired by an earlier commit.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{identity::*, runtime::*, schema::*, transactions::*};
use worth_testkit_relational::derived_relation::{commit, created, identity, registry};
struct StaleReader {
    definition: DerivedRelationHandle,
    root: Arc<Mutex<Option<EntityId>>>,
    evaluated: Arc<AtomicBool>,
}
impl CustomInvariantRule for StaleReader {
    type Scope = bool;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("stale-source"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Earlier retired source"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: vec![KindId(1)],
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
    ) -> Result<bool, CustomInvariantPreparationError> {
        let Some(root) = *self.root.lock().unwrap() else {
            return Ok(false);
        };
        for view in [planner.committed_relations(), planner.relations()] {
            assert_eq!(
                view.prepare_derived_relations_from(&self.definition, root)
                    .err(),
                Some(DerivedRelationReadDenial::Structural(
                    StructuralWalkDenialKind::Read(StructuralReadError::RecordUnavailable)
                ))
            );
        }
        Ok(true)
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        checked: &bool,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if *checked {
            self.evaluated.store(true, Ordering::SeqCst);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
#[test]
fn a_stale_derived_source_read_through_an_entity_deleted_by_an_earlier_commit_still_refuses() {
    let registry = registry();
    let root = Arc::new(Mutex::new(None));
    let evaluated = Arc::new(AtomicBool::new(false));
    let rule = StaleReader {
        definition: registry.derived_relation(&identity(2)).unwrap(),
        root: root.clone(),
        evaluated: evaluated.clone(),
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(CustomInvariantRegistration::new(rule).unwrap())
        .build()
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let mut batch = WorkerIntentBatch::new("birth");
    for i in 0..3 {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: created(i).client_key,
            fields: Default::default(),
        })));
    }
    let born = commit(&runtime, &execution, batch);
    let members: Vec<_> = (0..3)
        .map(|i| born.created_entity(&created(i)).unwrap())
        .collect();
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("earlier-delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: members[1],
            }),
        )),
    );
    *root.lock().unwrap() = Some(members[1]);
    // This candidate also deletes an entity. Its unrelated retirement must not
    // turn the earlier full-generation ID into an admitted empty source read.
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("later-delete")
            .push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent {
                    entity_id: members[2],
                },
            )))
            .push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
                RevalidateEntityIntent {
                    entity_id: members[0],
                },
            ))),
    );
    assert!(
        evaluated.load(Ordering::SeqCst),
        "both actual stale reads executed in the later candidate"
    );
}
