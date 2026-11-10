//! Contradictions start from an actual produced aggregate, not invented Original custody.
use super::super::*;
use crate::facade::runtime::*;
use crate::tests::support::{
    create_entity_outcome, release_test_commit_snapshot, test_schema_registry, TestDurableMedium,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;

struct Probe {
    point: InvariantExecutionPoint,
    calls: Arc<AtomicUsize>,
}
impl CustomInvariantRule for Probe {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(format!(
                    "semantic.family.{}",
                    self.point.diagnostic_label()
                )),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("actual semantic family"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract::default(),
                execution_point: self.point,
                groups: InvariantGroupSet::all(),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(CustomInvariantVerdict::Pass)
    }
}

fn recorded_execution() -> RelationalInvariantExecutionMeaning {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut builder = RelationalRuntimeApi::builder().schema_registry(test_schema_registry());
    for point in [
        InvariantExecutionPoint::CommitBoundary,
        InvariantExecutionPoint::MutationSensitive,
    ] {
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(Probe {
                point,
                calls: Arc::clone(&calls),
            })
            .unwrap(),
        );
    }
    let runtime = builder
        .build_durable(TestDurableMedium::new().attachment())
        .unwrap();
    let host = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&host);
    let committed = create_entity_outcome(&request, &runtime, "semantic-family-observation");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "both actual declared callbacks execute once"
    );
    let meaning = committed
        .semantic_result()
        .validation()
        .iter()
        .find(|meaning| meaning.metadata.execution_point == InvariantExecutionPoint::ProposedState)
        .unwrap()
        .clone();
    assert!(meaning.is_consistent());
    release_test_commit_snapshot(&runtime, &committed);
    meaning
}

fn custom_index(
    meaning: &RelationalInvariantExecutionMeaning,
    point: InvariantExecutionPoint,
) -> usize {
    meaning
        .checks
        .iter()
        .position(|check| {
            check.execution_point == point
                && matches!(&check.rule, InvariantReportedRule::Custom(_))
        })
        .unwrap()
}

#[test]
fn actual_aggregate_rejects_false_family_and_decision_claims() {
    let actual = recorded_execution();
    let index = custom_index(&actual, InvariantExecutionPoint::MutationSensitive);
    let mut wrong_family = actual.clone();
    wrong_family.checks[index].execution_point = InvariantExecutionPoint::GraphComposition;
    wrong_family.decisions[index].execution_point = InvariantExecutionPoint::GraphComposition;
    wrong_family.checks[index]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = InvariantObservationKind::Committed;
    assert!(
        !wrong_family.is_consistent(),
        "matching false tags do not add a fourth aggregate family"
    );
    let mut reporting_point_as_rule = actual.clone();
    reporting_point_as_rule.checks[index].execution_point = InvariantExecutionPoint::ProposedState;
    reporting_point_as_rule.decisions[index].execution_point =
        InvariantExecutionPoint::ProposedState;
    reporting_point_as_rule.checks[index]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = InvariantObservationKind::Committed;
    assert!(!reporting_point_as_rule.is_consistent());
    let mut contradicted_decision = actual;
    contradicted_decision.decisions[index].decision = InvariantDecisionKind::Violated;
    assert!(!contradicted_decision.is_consistent());
}

#[test]
fn actual_aggregate_rejects_contradicted_observation_and_current_version() {
    let actual = recorded_execution();
    let committed = custom_index(&actual, InvariantExecutionPoint::CommitBoundary);
    let applied = custom_index(&actual, InvariantExecutionPoint::MutationSensitive);
    let mut fabricated_aggregate = actual.clone();
    fabricated_aggregate.metadata.observation_kind = InvariantObservationKind::Speculative;
    fabricated_aggregate.checks[committed]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = InvariantObservationKind::Speculative;
    assert!(
        !fabricated_aggregate.is_consistent(),
        "coordinated false aggregate and committed-view tags still refuse"
    );
    let mut wrong_applied_view = actual.clone();
    wrong_applied_view.checks[applied]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = InvariantObservationKind::Committed;
    assert!(!wrong_applied_view.is_consistent());
    let mut wrong_current = actual;
    let current = wrong_current.metadata.current_version;
    wrong_current.checks[applied]
        .custom_provenance
        .as_mut()
        .unwrap()
        .current_version = VersionId(current.0.checked_add(1).unwrap());
    assert!(!wrong_current.is_consistent());
}

#[test]
fn standalone_committed_family_cannot_inherit_an_aggregate_view_exception() {
    let actual = recorded_execution();
    let index = custom_index(&actual, InvariantExecutionPoint::CommitBoundary);
    // Keep one genuine declared check and its original decision; contradict
    // the enclosing standalone observation and its copied custom observation.
    let mut wrong_observation = actual.clone();
    wrong_observation.checks = vec![actual.checks[index].clone()];
    wrong_observation.decisions = vec![actual.decisions[index].clone()];
    wrong_observation.metadata.execution_point = InvariantExecutionPoint::CommitBoundary;
    wrong_observation.metadata.observation_kind = InvariantObservationKind::Speculative;
    wrong_observation.checks[0]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = InvariantObservationKind::Speculative;
    assert!(!wrong_observation.is_consistent());
}

#[test]
fn data_consistency_keeps_execution_and_custom_versions_independent() {
    let mut description = recorded_execution();
    let index = custom_index(&description, InvariantExecutionPoint::CommitBoundary);
    // This varies descriptive fields only. No Original, artifact, source
    // admission or executed callback is authored from these chosen values.
    description.metadata.version = VersionId(11);
    description.checks[index]
        .custom_provenance
        .as_mut()
        .unwrap()
        .version = VersionId(12);
    assert!(
        description.is_consistent(),
        "data consistency does not impose a false equality between independent original fields"
    );
}
