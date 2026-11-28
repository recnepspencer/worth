//! The installed invariant coordinator accepts a handled named aggregate refusal.
use super::fixture::{commit, connect, relation, start};
use super::schema::{self, COUPLING, FACE};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{
        AggregateReadDenial, CustomInvariantAccessContract, CustomInvariantDescriptor,
        CustomInvariantExecutionContext, CustomInvariantExecutionError,
        CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
        CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
        CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
        CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
        InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
        PinnedAggregateReadScope, RelationalRuntimeApi, StructuralRelationView,
    },
    transactions::EntityReference,
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

#[derive(Default)]
struct Probe {
    face: Mutex<Option<EntityId>>,
    evaluated: AtomicBool,
}

struct Rule(Arc<Probe>);

impl Rule {
    fn read(view: StructuralRelationView<'_>, face: EntityId) {
        let definition = schema::descriptor(false).definition;
        let reads = view.aggregates();
        assert!(matches!(
            reads.of_definition(definition, face),
            Err(AggregateReadDenial::SourceUnavailable)
        ));
        assert!(matches!(
            reads.of_source_relation(COUPLING, face),
            Err(AggregateReadDenial::SourceUnavailable)
        ));
        assert!(matches!(
            reads.all_of_definition(definition),
            Err(AggregateReadDenial::SourceUnavailable)
        ));
    }
}

impl CustomInvariantRule for Rule {
    type Scope = Option<EntityId>;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.named-aggregate-source"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Named aggregate source".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![FACE],
                    read_relation_kinds: vec![COUPLING],
                    affected_entity_kinds: vec![FACE],
                    affected_relation_kinds: vec![COUPLING],
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
        let face = *self.0.face.lock().unwrap();
        if let Some(face) = face {
            Self::read(planner.committed_relations(), face);
            Self::read(planner.relations(), face);
        }
        Ok(face)
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if let Some(face) = *scope {
            Self::read(context.committed_relations(), face);
            Self::read(context.relations(), face);
            self.0.evaluated.store(true, Ordering::Relaxed);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn named_aggregate_rule_refusal_leaves_the_commit_publishable() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let probe = Arc::new(Probe::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema::registry(false, true))
        .custom_invariant(CustomInvariantRegistration::new(Rule(Arc::clone(&probe))).unwrap())
        .build()
        .unwrap();
    let born = start(request, &runtime, false, true);
    *probe.face.lock().unwrap() = Some(born.face);
    let endpoint = EntityReference::Existing(born.face);
    let replacement = relation(COUPLING, "replacement", endpoint.clone(), endpoint);
    let published = commit(
        request,
        &runtime,
        super::disconnect(born.coupling).push(connect(&replacement)),
        Vec::new(),
    );
    assert!(probe.evaluated.load(Ordering::Relaxed));
    let pin = runtime
        .read_truth()
        .pin_read_capability(&published.snapshot, &[], request)
        .unwrap();
    let group = pin
        .read_aggregate(request, PinnedAggregateReadScope::new(COUPLING, born.face))
        .unwrap();
    assert_eq!(group.members().collect::<Vec<_>>(), [born.face]);
}
