//! Original custom enforcement and aggregate request versions are distinct facts.
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::runtime::InvariantObservationKind;
use worth_relational::facade::runtime::*;
use worth_relational::facade::{
    config::RelationalRuntimeProfile, durability::RelationalCanonicalCommitArtifact as Artifact,
    identity::KindId, runtime::RelationalRuntimeApi,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, schema},
};

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
fn semantic_result_preserves_custom_enforcement_version_distinct_from_aggregate_request() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(schema())
        .custom_invariant(
            CustomInvariantRegistration::new(OnePlannedEntity {
                observed: observed.clone(),
            })
            .unwrap(),
        )
        .build_durable(medium.attachment())
        .unwrap();
    let (_, predecessor_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let predecessor = predecessor_basis.observation().version_id();
    let committed = create_entity_outcome(execution, &runtime, "actual-custom-bootstrap");
    let callback = observed.lock().unwrap().clone();
    assert!(
        !callback.is_empty(),
        "the original ordinary producer actually ran the installed custom rule"
    );
    let semantic = committed.semantic_result();
    let (execution_meaning, custom) = semantic
        .validation()
        .iter()
        .find_map(|meaning| {
            meaning.checks.iter().find_map(|check| {
                check
                    .custom_provenance
                    .as_ref()
                    .map(|custom| (meaning, custom))
            })
        })
        .unwrap();
    assert_eq!(execution_meaning.metadata.version, predecessor);
    assert_eq!(custom.version, committed.commit.version_id);
    assert_ne!(custom.version, execution_meaning.metadata.version, "the aggregate request describes the predecessor; the real custom callback checked the proposed effect");
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
    let artifact = medium.offered().pop().unwrap();
    let readmitted = Artifact::readmit(artifact.as_bytes(), execution).unwrap();
    assert_eq!(readmitted.envelope().semantic_result(), semantic);

    release_test_commit_snapshot(&runtime, &committed);
}
