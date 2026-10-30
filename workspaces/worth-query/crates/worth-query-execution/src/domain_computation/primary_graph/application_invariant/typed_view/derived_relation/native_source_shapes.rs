//! Exact interpretations share the real evaluator, never each other's proof.
use super::native_fixture::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::*, runtime::*, schema::*, symbols::ClientKey, transactions::*,
};

#[derive(Default)]
struct Control {
    members: Mutex<Vec<EntityId>>,
    evaluated: AtomicBool,
}
struct Reader {
    definition: DerivedRelationHandle,
    control: Arc<Control>,
}
impl Reader {
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    fn check(&self, view: &StructuralRelationView<'_>, proofs: &[PreparedDerivedRelationRead]) {
        let members = self.control.members.lock().unwrap();
        let adjacency = DerivedRelationReadShape::Adjacency { reverse: false };
        let reachable = DerivedRelationReadShape::Reachable { reverse: false };
        let component = DerivedRelationReadShape::Component;
        let mut expected_adjacency = [
            NativeMemberKey::Entity(members[1]),
            NativeMemberKey::Entity(members[2]),
        ];
        expected_adjacency.sort();
        assert_eq!(
            view.read_prepared_derived_source(&proofs[0], adjacency)
                .unwrap()
                .endpoints(),
            expected_adjacency
        );
        let mut expected_members = members
            .iter()
            .copied()
            .map(NativeMemberKey::Entity)
            .collect::<Vec<_>>();
        expected_members.sort();
        for (proof, shape) in [(&proofs[1], reachable), (&proofs[2], component)] {
            assert_eq!(
                view.read_prepared_derived_source(proof, shape)
                    .unwrap()
                    .endpoints(),
                expected_members
            );
            let data = view.read_prepared_derived_source(proof, shape).unwrap();
            assert_eq!(data.physical_relations().len(), 4);
            assert!(
                data.adjacency_dependencies()
                    .iter()
                    .any(|probe| probe.anchor == members[4] && probe.outgoing),
                "terminal empty adjacency is retained"
            );
        }
        assert_ne!(proofs[0].identity(), proofs[1].identity());
        assert_ne!(proofs[1].identity(), proofs[2].identity());
        assert_eq!(
            view.read_prepared_derived_source(&proofs[3], reachable)
                .unwrap()
                .endpoints(),
            [members[4].into()]
        );
        assert_eq!(
            view.read_prepared_derived_source(&proofs[4], component)
                .unwrap()
                .endpoints(),
            expected_members
        );
        let reverse = DerivedRelationReadShape::Reachable { reverse: true };
        let mut predecessors = [
            NativeMemberKey::Entity(members[0]),
            NativeMemberKey::Entity(members[1]),
            NativeMemberKey::Entity(members[4]),
        ];
        predecessors.sort();
        assert_eq!(
            view.read_prepared_derived_source(&proofs[5], reverse)
                .unwrap()
                .endpoints(),
            predecessors
        );
        assert_eq!(
            view.read_prepared_derived_source(&proofs[5], reachable)
                .unwrap_err(),
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::InvalidExpression)
        );
        assert_eq!(
            view.read_prepared_derived_source(&proofs[0], component)
                .unwrap_err(),
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::InvalidExpression)
        );
        assert_eq!(
            view.read_prepared_derived_relations(&proofs[1])
                .unwrap_err(),
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::InvalidExpression)
        );
        let mut expected_steps = [(0, 0), (1, 1), (2, 1), (3, 2), (4, 2)]
            .map(|(index, distance)| (NativeMemberKey::Entity(members[index]), distance));
        expected_steps
            .sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
        assert_eq!(
            view.read_prepared_derived_steps(&proofs[1], reachable)
                .unwrap()
                .iter()
                .map(|step| (step.member.clone(), step.distance))
                .collect::<Vec<_>>(),
            expected_steps
        );
    }
}
impl CustomInvariantRule for Reader {
    type Scope = Option<Vec<PreparedDerivedRelationRead>>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("derived-source-interpretations"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Actual derived traversal interpretations"),
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
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let members = self.control.members.lock().unwrap();
        let Some(&root) = members.first() else {
            return Ok(None);
        };
        let terminal = members[4];
        drop(members);
        let view = planner.relations();
        let mut proofs = [
            DerivedRelationReadShape::Adjacency { reverse: false },
            DerivedRelationReadShape::Reachable { reverse: false },
            DerivedRelationReadShape::Component,
        ]
        .into_iter()
        .map(|shape| {
            view.prepare_derived_source(&self.definition, root, shape)
                .unwrap()
        })
        .collect::<Vec<_>>();
        for shape in [
            DerivedRelationReadShape::Reachable { reverse: false },
            DerivedRelationReadShape::Component,
            DerivedRelationReadShape::Reachable { reverse: true },
        ] {
            proofs.push(
                view.prepare_derived_source(&self.definition, terminal, shape)
                    .unwrap(),
            );
        }
        self.check(&view, &proofs);
        assert_eq!(
            planner
                .committed_relations()
                .read_prepared_derived_source(&proofs[2], DerivedRelationReadShape::Component)
                .unwrap_err(),
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::ForeignObservation)
        );
        Ok(Some(proofs))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if let Some(proofs) = scope {
            self.check(&context.relations(), proofs);
            self.control.evaluated.store(true, Ordering::SeqCst);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn prepared_adjacency_reachability_and_component_sources_preserve_shape_and_empty_support() {
    let registry = registry();
    let control = Arc::new(Control::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry.clone())
        .custom_invariant(
            CustomInvariantRegistration::new(Reader {
                definition: registry.derived_relation(&identity(2)).unwrap(),
                control: control.clone(),
            })
            .unwrap(),
        )
        .build()
        .expect("valid native fixture configuration");
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let mut birth = WorkerIntentBatch::new("shape-source-birth");
    for index in 0..5 {
        birth = birth.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: created(index).client_key,
            fields: Default::default(),
        })));
    }
    for (index, (source, target)) in [(0, 1), (0, 2), (1, 4), (2, 3)].into_iter().enumerate() {
        birth = birth.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(format!("shape-edge-{index}")),
                source: EntityReference::Created(created(source)),
                target: EntityReference::Created(created(target)),
                fields: Default::default(),
            },
        )));
    }
    let born = commit(&runtime, &execution, birth);
    let members = (0..5)
        .map(|index| born.created_entity(&created(index)).unwrap())
        .collect::<Vec<_>>();
    let root = members[0];
    *control.members.lock().unwrap() = members;
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("shape-source-check").push(MutationIntent::Entity(
            EntityMutationIntent::Revalidate(RevalidateEntityIntent { entity_id: root }),
        )),
    );
    assert!(control.evaluated.load(Ordering::SeqCst));
}
