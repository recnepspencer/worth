//! Actual installed invariant receivers retain definition, audience and liveness gates.
use super::*;

#[derive(Default)]
struct Observed {
    root: Option<EntityId>,
    target: Option<EntityId>,
    prepared: Option<Arc<PreparedDerivedRelationRead>>,
    wide: usize,
    narrow: usize,
}
struct Reader {
    definition: DerivedRelationHandle,
    foreign: DerivedRelationHandle,
    observed: Arc<Mutex<Observed>>,
    narrow: bool,
    cancel: Option<CancellationSource>,
}
impl CustomInvariantRule for Reader {
    type Scope = Option<Arc<PreparedDerivedRelationRead>>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let name = if self.narrow {
            "derived-narrow"
        } else {
            "derived-wide"
        };
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(name),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: name.into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(4)],
                    read_relation_kinds: if self.narrow { vec![] } else { vec![KindId(2)] },
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
        let root = self.observed.lock().unwrap().root;
        if self.narrow || root.is_none() {
            return Ok(None);
        }
        let prepared = Arc::new(
            planner
                .relations()
                .prepare_derived_relations_from(&self.definition, root.unwrap())
                .unwrap(),
        );
        self.observed.lock().unwrap().prepared = Some(prepared.clone());
        Ok(Some(prepared))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let view = context.relations();
        if self.narrow {
            if let Some(prepared) = self.observed.lock().unwrap().prepared.as_ref() {
                assert!(matches!(
                    view.read_prepared_derived_relations(prepared),
                    Err(DerivedRelationReadDenial::Structural(
                        StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess)
                    ))
                ));
            } else {
                return Ok(CustomInvariantVerdict::Pass);
            }
            self.observed.lock().unwrap().narrow += 1;
        } else if let Some(prepared) = scope {
            let (root, target) = {
                let state = self.observed.lock().unwrap();
                (state.root.unwrap(), state.target.unwrap())
            };
            assert_eq!(
                view.read_prepared_derived_relations(prepared)
                    .unwrap()
                    .endpoints(),
                &[target.into()]
            );
            // Both the warm view and its clone must keep actual registered identity.
            assert_eq!(
                view.clone()
                    .read_prepared_derived_relations(prepared)
                    .unwrap()
                    .endpoints(),
                &[target.into()]
            );
            assert!(matches!(
                view.derived_read_identity(&self.foreign, root),
                Err(DerivedRelationReadDenial::Definition(
                    DerivedRelationDenial::ForeignDefinition
                ))
            ));
            self.observed.lock().unwrap().wide += 1;
            if let Some(cancellation) = &self.cancel {
                cancellation.cancel();
                assert!(matches!(
                    view.derived_read_identity(&self.definition, root),
                    Err(DerivedRelationReadDenial::Structural(
                        StructuralWalkDenialKind::Interrupted(
                            CustomInvariantInterruption::Cancelled
                        )
                    ))
                ));
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
fn definition(registry: &RelationalSchemaRegistry) -> DerivedRelationHandle {
    registry
        .derived_relation(&DerivedRelationIdentity {
            schema: SchemaId("managed-source-court".into()),
            version: SchemaVersionId(1),
            name: "directory-items".into(),
        })
        .unwrap()
}
#[test]
fn derived_admission_preserves_real_prepared_audience_identity_and_cancelled_request() {
    for cancel in [false, true] {
        let (schema, _) = registry();
        let actual = definition(&schema);
        let foreign = definition(&registry().0);
        assert_eq!(actual.parts(), foreign.parts());
        let observed = Arc::new(Mutex::new(Observed::default()));
        let cancellation = CancellationSource::new();
        let mut builder = RelationalRuntimeApi::builder().schema_registry(schema);
        for narrow in [false, true]
            .into_iter()
            .filter(|narrow| !cancel || !narrow)
        {
            builder = builder.custom_invariant(
                CustomInvariantRegistration::new(Reader {
                    definition: actual.clone(),
                    foreign: foreign.clone(),
                    observed: observed.clone(),
                    narrow,
                    cancel: cancel.then(|| cancellation.clone()),
                })
                .unwrap(),
            );
        }
        let runtime = builder.build().unwrap();
        let host =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&host);
        let mut birth = WorkerIntentBatch::new("derived-admission-birth");
        for (kind, label) in [(1, "root"), (4, "target")] {
            birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(kind),
                client_key: ClientKey::raw(label),
                fields: Default::default(),
            })));
        }
        let born = commit(&runtime, &execution, birth);
        let root = born.created_entity(&created(1, "root")).unwrap();
        let target = born.created_entity(&created(4, "target")).unwrap();
        {
            let mut state = observed.lock().unwrap();
            state.root = Some(root);
            state.target = Some(target);
        }
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("derived-admission-edge").push(MutationIntent::Create(
                    CreateIntent::Relation(RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(2),
                        client_key: ClientKey::raw("edge"),
                        source: EntityReference::Existing(root),
                        target: EntityReference::Existing(target),
                        fields: Default::default(),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let result = transaction.commit(&execution, &runtime);
        if cancel {
            assert!(
                matches!(result, Err(TransactionCommitError::Execution { denial, .. })
                if matches!(denial.kind, CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)))
            );
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .1,
                basis
            );
        } else {
            result.unwrap();
        }
        let state = observed.lock().unwrap();
        assert_eq!(state.wide, 1);
        assert_eq!(state.narrow, usize::from(!cancel));
    }
}
