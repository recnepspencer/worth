//! Original custom enforcement and aggregate request versions are distinct facts.
use super::*;
use crate::facade::identity::VersionId;
use crate::validation::data::*;
use crate::validation::engine::InvariantObservationKind;
use std::sync::{Arc, Mutex};

struct OnePlannedEntity {
    observed: Arc<Mutex<Vec<(InvariantObservationKind, VersionId, VersionId)>>>,
}
impl CustomInvariantRule for OnePlannedEntity {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("ordinary.original-enforcement-version"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("One actual planned entity"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: Vec::new(),
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: Vec::new(),
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
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.observed.lock().unwrap().push((
            context.observation_kind(),
            context.version_id(),
            context.current_version_id(),
        ));
        Ok(if context.touched().planned_entity_creates().len() == 1 {
            CustomInvariantVerdict::Pass
        } else {
            CustomInvariantVerdict::Violation(CustomInvariantRefusal::Empty)
        })
    }
}
#[test]
fn semantic_result_preserves_independent_original_execution_and_enforcement_versions() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(OnePlannedEntity {
                observed: observed.clone(),
            })
            .unwrap(),
        )
        .build_durable(TestDurableMedium::new().attachment())
        .unwrap();
    let predecessor = runtime.current_version_id();
    let committed = create_entity_outcome(execution, &runtime, "actual-custom-bootstrap");
    let callback = observed.lock().unwrap().clone();
    assert!(
        !callback.is_empty(),
        "the original ordinary producer actually ran the installed custom rule"
    );
    let semantic = committed.semantic_result();
    let (execution_index, check_index, execution_meaning, custom) = semantic
        .validation()
        .iter()
        .enumerate()
        .find_map(|(execution_index, meaning)| {
            meaning
                .checks
                .iter()
                .enumerate()
                .find_map(|(check_index, check)| {
                    check
                        .custom_provenance
                        .as_ref()
                        .map(|custom| (execution_index, check_index, meaning, custom))
                })
        })
        .unwrap();
    let original_execution = &committed.validation().invariant_executions()[execution_index];
    assert_eq!(
        execution_meaning.metadata.execution_point,
        original_execution.metadata().execution_point()
    );
    assert_eq!(
        execution_meaning.metadata.observation_kind,
        original_execution.metadata().observation_kind()
    );
    assert_eq!(
        execution_meaning.metadata.version,
        original_execution.metadata().version_id()
    );
    assert_eq!(
        execution_meaning.metadata.current_version,
        original_execution.metadata().current_version_id()
    );
    assert_eq!(
        execution_meaning.metadata.execution_point,
        InvariantExecutionPoint::ProposedState
    );
    assert_eq!(
        execution_meaning.metadata.version,
        committed.commit.version_id
    );
    assert_eq!(execution_meaning.metadata.current_version, predecessor);
    assert_eq!(custom.version, committed.commit.version_id);
    assert_ne!(custom.version, execution_meaning.metadata.current_version, "the actual custom callback checked the proposed effect while retaining its predecessor basis");
    assert!(callback.contains(&(
        custom.observation_kind,
        custom.version,
        custom.current_version
    )));
    assert_eq!(
        custom.observation_kind,
        execution_meaning.metadata.observation_kind
    );
    assert_eq!(
        custom.current_version,
        execution_meaning.metadata.current_version
    );
    let artifact = attached_test_medium(&runtime).offered().pop().unwrap();
    let readmitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    assert_eq!(readmitted.envelope().semantic_result(), semantic);

    // Preserve the complete original image but contradict its actual decision.
    // Repair only outer framing so admission must reject semantic disagreement.
    let original_decision = &semantic.validation()[execution_index].decisions[check_index];
    let encoded = rmp_serde::to_vec_named(original_decision).unwrap();
    let mut malformed = original_decision.clone();
    malformed.decision = InvariantDecisionKind::Violated;
    let replacement = rmp_serde::to_vec_named(&malformed).unwrap();
    let mut bytes = artifact.as_bytes().to_vec();
    let offsets = bytes
        .windows(encoded.len())
        .enumerate()
        .filter_map(|(offset, span)| (span == encoded).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 1);
    bytes.splice(offsets[0]..offsets[0] + encoded.len(), replacement);
    reseal_canonical_commit_frame(&mut bytes);
    assert!(Artifact::from_untrusted_bytes(bytes.clone()).is_ok());
    assert!(matches!(Artifact::readmit(&bytes, execution), Err(crate::facade::durability::RelationalCanonicalCommitArtifactRefusal::PositionAdmission(crate::runtime::RelationalCanonicalPositionAdmissionDenial::SemanticResult(crate::transactions::data::RelationalCommitSemanticResultRefusal::ValidationMismatch)))));
    release_test_commit_snapshot(&runtime, &committed);
}
