//! Empty census proofs preserve the real view audience and source currency.
use super::fixtures::*;
use crate::aggregate_identity::{AggregateReadDenial, PreparedAggregateCensus};
use crate::validation::data::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_foundational::facade::AggregateDefinitionIdentity;

type Proof = PreparedAggregateCensus<NativeMemberKey>;
#[derive(Default)]
struct Control {
    broad: Mutex<Option<Proof>>,
    prepared: AtomicBool,
    evaluated: AtomicBool,
}
#[derive(Clone, Copy)]
enum Mode {
    Broad,
    Narrow,
    Foreign,
}
struct Reader {
    mode: Mode,
    control: Arc<Control>,
}
struct Scope {
    proposed: Proof,
    committed: Proof,
}
fn registry_with_narrow() -> RelationalSchemaRegistry {
    let projected = NativeMemberDomain::Projected {
        projected: domain(),
    };
    let registry = registry()
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: identity("Isolated"),
            from: projected.clone(),
            to: projected,
            nodes: vec![
                DerivedRelationNode::CarrierLift {
                    domain: domain(),
                    case: CASES[0].into(),
                },
                DerivedRelationNode::Reverse { input: 0 },
                DerivedRelationNode::Compose { left: 1, right: 0 },
                DerivedRelationNode::CarrierLift {
                    domain: domain(),
                    case: CASES[1].into(),
                },
                DerivedRelationNode::Reverse { input: 3 },
                DerivedRelationNode::Compose { left: 4, right: 3 },
                DerivedRelationNode::Union { left: 2, right: 5 },
            ],
            laws: None,
        }])
        .unwrap();
    let mut own = descriptor();
    own.definition = AggregateDefinitionIdentity::from_bytes([43; 32]);
    own.membership_kind = KindId(23);
    own.source = AggregateIdentitySource::DerivedComponents {
        derived_components: identity("Isolated"),
    };
    crate::schema::aggregate_identity::register_aggregate_membership(
        registry,
        &domain().schema,
        SchemaVersionId(1),
        own,
    )
    .unwrap()
}
impl Reader {
    fn own_definition(&self) -> AggregateDefinitionIdentity {
        if matches!(self.mode, Mode::Narrow) {
            AggregateDefinitionIdentity::from_bytes([43; 32])
        } else {
            descriptor().definition
        }
    }
    fn borrowed(&self, view: &StructuralRelationView<'_>) {
        let broad = self.control.broad.lock().unwrap();
        let broad = broad
            .as_ref()
            .expect("first genuine rule prepared on this observation");
        let expected = if matches!(self.mode, Mode::Narrow) {
            AggregateReadDenial::Derived(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess),
            ))
        } else {
            AggregateReadDenial::Derived(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::ForeignObservation,
            ))
        };
        assert_eq!(
            view.aggregates().read_prepared_census(broad).err(),
            Some(expected)
        );
    }
    fn own(
        &self,
        proposed: &StructuralRelationView<'_>,
        committed: &StructuralRelationView<'_>,
        scope: &Scope,
    ) {
        assert!(proposed
            .aggregates()
            .read_prepared_census(&scope.proposed)
            .unwrap()
            .is_empty());
        assert!(committed
            .aggregates()
            .read_prepared_census(&scope.committed)
            .unwrap()
            .is_empty());
        let expected = Some(AggregateReadDenial::Derived(
            DerivedRelationReadDenial::Definition(DerivedRelationDenial::ForeignObservation),
        ));
        // These use the SAME fully allowed source; audience refusal cannot
        // counterfeit the independent proposed/committed currency check.
        assert_eq!(
            committed
                .aggregates()
                .read_prepared_census(&scope.proposed)
                .err(),
            expected
        );
        assert_eq!(
            proposed
                .aggregates()
                .read_prepared_census(&scope.committed)
                .err(),
            expected
        );
    }
}
impl CustomInvariantRule for Reader {
    type Scope = Scope;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        let name = match self.mode {
            Mode::Broad => "empty-census-0-broad",
            Mode::Narrow => "empty-census-1-narrow",
            Mode::Foreign => "empty-census-2-foreign",
        };
        let narrow = matches!(self.mode, Mode::Narrow);
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(name),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Exact empty census authority"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: if narrow {
                        vec![KindId(1), KindId(6)]
                    } else {
                        vec![KindId(1), KindId(6), KindId(7), KindId(8)]
                    },
                    read_relation_kinds: if narrow {
                        vec![]
                    } else {
                        vec![
                            KindId(INCIDENCE),
                            KindId(NEG_REGION),
                            KindId(POS_REGION),
                            KindId(NEG_EXTERIOR),
                            KindId(POS_EXTERIOR),
                        ]
                    },
                    affected_entity_kinds: vec![KindId(6)],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
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
    ) -> Result<Scope, CustomInvariantPreparationError> {
        let proposed = planner.relations();
        let committed = planner.committed_relations();
        if !matches!(self.mode, Mode::Broad) {
            self.borrowed(&proposed);
        }
        let definition = self.own_definition();
        let scope = Scope {
            proposed: proposed
                .aggregates()
                .prepare_projected_census_of_definition(definition)
                .unwrap(),
            committed: committed
                .aggregates()
                .prepare_projected_census_of_definition(definition)
                .unwrap(),
        };
        self.own(&proposed, &committed, &scope);
        if matches!(self.mode, Mode::Broad) {
            *self.control.broad.lock().unwrap() = Some(scope.proposed.clone());
        } else {
            self.control.prepared.store(true, Ordering::SeqCst);
        }
        Ok(scope)
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let proposed = context.relations();
        let committed = context.committed_relations();
        if !matches!(self.mode, Mode::Broad) {
            self.borrowed(&proposed);
        }
        self.own(&proposed, &committed, scope);
        if !matches!(self.mode, Mode::Broad) {
            self.control.evaluated.store(true, Ordering::SeqCst);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
#[test]
fn empty_census_needs_its_own_audience_posture_and_genuine_registry_observation() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let control = Arc::new(Control::default());
    let mut builder = RelationalRuntimeApi::builder().schema_registry(registry_with_narrow());
    for mode in [Mode::Broad, Mode::Narrow] {
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(Reader {
                mode,
                control: Arc::clone(&control),
            })
            .unwrap(),
        );
    }
    let runtime = builder.build().expect("valid native runtime configuration");
    // A genuine business touch runs both rules without ever creating a carrier.
    let born = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("empty-census-vertex").push(entity(6, 0)),
    );
    let truth = runtime
        .read_truth()
        .read_snapshot(&born.snapshot, execution)
        .unwrap()
        .unwrap();
    assert!(truth
        .entities()
        .iter()
        .all(|entity| entity.kind.kind_id != KindId(1)));
    assert!(control.prepared.load(Ordering::SeqCst));
    assert!(control.evaluated.load(Ordering::SeqCst));
    drop(truth);
    control.prepared.store(false, Ordering::SeqCst);
    control.evaluated.store(false, Ordering::SeqCst);
    let foreign = RelationalRuntimeApi::builder()
        .schema_registry(registry_with_narrow())
        .custom_invariant(
            CustomInvariantRegistration::new(Reader {
                mode: Mode::Foreign,
                control: Arc::clone(&control),
            })
            .unwrap(),
        )
        .build()
        .expect("valid native runtime configuration");
    // Full source access here proves the earlier audience denial is NOT the
    // reason the original runtime's genuine proof cannot be lent to this owner.
    commit(
        execution,
        &foreign,
        WorkerIntentBatch::new("foreign-empty-census").push(entity(6, 0)),
    );
    assert!(control.prepared.load(Ordering::SeqCst));
    assert!(control.evaluated.load(Ordering::SeqCst));
}
