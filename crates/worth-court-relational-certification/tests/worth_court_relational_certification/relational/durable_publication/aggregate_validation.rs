//! Contradictions start from an actual produced aggregate, not invented Original custody.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        RelationalCanonicalCommitArtifact as Artifact,
        RelationalCanonicalCommitArtifactRefusal as ArtifactRefusal,
        RelationalCanonicalPositionAdmissionDenial as PositionDenial,
    },
    identity::VersionId,
    runtime::*,
    transactions::{
        RelationalCommitSemanticResult, RelationalCommitSemanticResultRefusal,
        RelationalInvariantExecutionMeaning,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, schema},
};

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

fn recorded_execution() -> Fixture {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut builder = RelationalRuntimeApi::builder().schema_registry(schema());
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
    let medium = TestDurableMedium::new();
    let runtime = builder.build_durable(medium.attachment()).unwrap();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
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
    let original = medium.offered().pop().unwrap();
    let semantic = serde_json::to_value(committed.semantic_result()).unwrap();
    let slot = committed
        .semantic_result()
        .validation()
        .iter()
        .position(|entry| entry.metadata.execution_point == InvariantExecutionPoint::ProposedState)
        .unwrap();
    let fixture = Fixture {
        meaning,
        semantic,
        frame: original.as_bytes().to_vec(),
        slot,
    };
    fixture.require_valid(&fixture.meaning);
    release_test_commit_snapshot(&runtime, &committed);
    fixture
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
    let fixture = recorded_execution();
    let actual = fixture.meaning.clone();
    let index = custom_index(&actual, InvariantExecutionPoint::MutationSensitive);
    let committed_view = actual.checks
        [custom_index(&actual, InvariantExecutionPoint::CommitBoundary)]
    .custom_provenance
    .as_ref()
    .unwrap()
    .observation_kind;
    let mut wrong_family = actual.clone();
    wrong_family.checks[index].execution_point = InvariantExecutionPoint::GraphComposition;
    wrong_family.decisions[index].execution_point = InvariantExecutionPoint::GraphComposition;
    wrong_family.checks[index]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = committed_view;
    fixture.require_invalid(&wrong_family);
    let mut reporting_point_as_rule = actual.clone();
    reporting_point_as_rule.checks[index].execution_point = InvariantExecutionPoint::ProposedState;
    reporting_point_as_rule.decisions[index].execution_point =
        InvariantExecutionPoint::ProposedState;
    reporting_point_as_rule.checks[index]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = committed_view;
    fixture.require_invalid(&reporting_point_as_rule);
    let mut contradicted_decision = actual;
    contradicted_decision.decisions[index].decision = InvariantDecisionKind::Violated;
    fixture.require_invalid(&contradicted_decision);
}

#[test]
fn actual_aggregate_rejects_contradicted_observation_and_current_version() {
    let fixture = recorded_execution();
    let actual = fixture.meaning.clone();
    let committed = custom_index(&actual, InvariantExecutionPoint::CommitBoundary);
    let applied = custom_index(&actual, InvariantExecutionPoint::MutationSensitive);
    let committed_view = actual.checks[committed]
        .custom_provenance
        .as_ref()
        .unwrap()
        .observation_kind;
    let speculative_view = actual.checks[applied]
        .custom_provenance
        .as_ref()
        .unwrap()
        .observation_kind;
    let mut fabricated_aggregate = actual.clone();
    fabricated_aggregate.metadata.observation_kind = speculative_view;
    fabricated_aggregate.checks[committed]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = speculative_view;
    fixture.require_invalid(&fabricated_aggregate);
    let mut wrong_applied_view = actual.clone();
    wrong_applied_view.checks[applied]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = committed_view;
    fixture.require_invalid(&wrong_applied_view);
    let mut wrong_current = actual;
    let current = wrong_current.metadata.current_version;
    wrong_current.checks[applied]
        .custom_provenance
        .as_mut()
        .unwrap()
        .current_version = VersionId(current.0.checked_add(1).unwrap());
    fixture.require_invalid(&wrong_current);
}

#[test]
fn standalone_committed_family_cannot_inherit_an_aggregate_view_exception() {
    let fixture = recorded_execution();
    let actual = fixture.meaning.clone();
    let index = custom_index(&actual, InvariantExecutionPoint::CommitBoundary);
    let speculative_view = actual.checks
        [custom_index(&actual, InvariantExecutionPoint::MutationSensitive)]
    .custom_provenance
    .as_ref()
    .unwrap()
    .observation_kind;
    // Keep one genuine declared check and its original decision; contradict
    // the enclosing standalone observation and its copied custom observation.
    let mut wrong_observation = actual.clone();
    wrong_observation.checks = vec![actual.checks[index].clone()];
    wrong_observation.decisions = vec![actual.decisions[index].clone()];
    wrong_observation.metadata.execution_point = InvariantExecutionPoint::CommitBoundary;
    wrong_observation.metadata.observation_kind = speculative_view;
    wrong_observation.checks[0]
        .custom_provenance
        .as_mut()
        .unwrap()
        .observation_kind = speculative_view;
    fixture.require_invalid(&wrong_observation);
}

#[test]
fn data_consistency_keeps_execution_and_custom_versions_independent() {
    let fixture = recorded_execution();
    let mut description = fixture.meaning.clone();
    let index = custom_index(&description, InvariantExecutionPoint::CommitBoundary);
    // This varies descriptive fields only. No Original, artifact, source
    // admission or executed callback is authored from these chosen values.
    description.metadata.version = VersionId(11);
    description.checks[index]
        .custom_provenance
        .as_mut()
        .unwrap()
        .version = VersionId(12);
    fixture.require_valid(&description);
}

struct Fixture {
    meaning: RelationalInvariantExecutionMeaning,
    semantic: serde_json::Value,
    frame: Vec<u8>,
    slot: usize,
}
impl Fixture {
    fn encoded(&self, meaning: &RelationalInvariantExecutionMeaning) -> Vec<u8> {
        let mut semantic = self.semantic.clone();
        semantic["validation"][self.slot] = serde_json::to_value(meaning).unwrap();
        let semantic: RelationalCommitSemanticResult = serde_json::from_value(semantic).unwrap();
        let canonical = crate::messagepack_value_spans::field(&self.frame[66..], "canonical");
        let old = crate::messagepack_value_spans::field(canonical, "semantic_result");
        let offset = old.as_ptr() as usize - self.frame.as_ptr() as usize;
        let mut frame = self.frame.clone();
        frame.splice(
            offset..offset + old.len(),
            rmp_serde::to_vec_named(&semantic).unwrap(),
        );
        crate::canonical_commit_frame::reseal(&mut frame);
        Artifact::from_untrusted_bytes(frame.clone())
            .expect("frame integrity passes before semantic admission");
        frame
    }
    fn require_valid(&self, meaning: &RelationalInvariantExecutionMeaning) {
        let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let frame = self.encoded(meaning);
        let admitted = Artifact::readmit(&frame, &execution).unwrap();
        assert_eq!(
            &admitted.envelope().semantic_result().validation()[self.slot],
            meaning
        );
    }
    fn require_invalid(&self, meaning: &RelationalInvariantExecutionMeaning) {
        let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let frame = self.encoded(meaning);
        let refusal = Artifact::readmit(&frame, &execution).unwrap_err();
        assert!(
            matches!(
                refusal,
                ArtifactRefusal::PositionAdmission(PositionDenial::SemanticResult(
                    RelationalCommitSemanticResultRefusal::ValidationMismatch
                ))
            ),
            "{refusal:?}"
        );
    }
}
