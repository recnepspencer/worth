//! Actual initialization-step fates gate the original live schema cutover.
use super::*;
use std::sync::Mutex;
use worth_relational::facade::{branch::*, durability::*, runtime::*};

#[derive(Clone, Copy, Debug)]
enum Answer {
    NoEffect,
    Indeterminate,
}
#[derive(Debug)]
struct InitialStep {
    actual: Box<dyn RelationalDurableMedium>,
    answer: Answer,
    offered: Arc<Mutex<Option<RelationalSourceInitialInputsArtifact>>>,
}
impl RelationalDurableMedium for InitialStep {
    fn publish_initial_inputs(
        &self,
        artifact: &RelationalSourceInitialInputsArtifact,
        _: &ExecutionRequest<'_, '_>,
    ) -> DurablePublicationFate<RelationalInitialInputsPublicationDetail> {
        let mut offered = self.offered.lock().unwrap();
        assert!(
            offered.is_none(),
            "the unresolved publication cannot be offered again"
        );
        *offered = Some(artifact.clone());
        let detail =
            RelationalInitialInputsPublicationDetail::Physical("owner-step interruption".into());
        match self.answer {
            Answer::NoEffect => DurablePublicationFate::ProvenNoEffect { detail },
            Answer::Indeterminate => DurablePublicationFate::Indeterminate { detail },
        }
    }
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.actual.cold_source()
    }
    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        self.actual.read_transition(sequence)
    }
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        self.actual.publish(artifact)
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.actual.publish_transition(artifact)
    }
}

#[test]
fn initialization_no_effect_preserves_original_cells_and_indeterminate_blocks_later_publication() {
    for answer in [Answer::NoEffect, Answer::Indeterminate] {
        let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&policy);
        let medium = TestDurableMedium::new();
        let offered = Arc::new(Mutex::new(None));
        let mut runtime = RelationalRuntimeApi::builder()
            .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
            .schema_registry(worth_testkit_relational::durable_publication::schema())
            .build_durable(Box::new(InitialStep {
                actual: medium.attachment(),
                answer,
                offered: offered.clone(),
            }))
            .unwrap();
        let main = runtime.main_branch_identity();
        let original = current(&request, &runtime);
        let permission = runtime.begin_fresh_home_initialization(&request).unwrap();
        let denied = runtime
            .prepare_fresh_home_schema_installation(&request, permission)
            .unwrap()
            .install_with_managed_sources(&request, additions(), vec![], vec![])
            .unwrap_err();
        assert_eq!(current(&request, &runtime), original);
        assert!(medium.offered().is_empty());
        assert!(medium
            .artifact_bytes(worth_relational::facade::publication::PatchStreamPosition(
                1
            ))
            .is_none());
        let offered = offered.lock().unwrap();
        let artifact = offered
            .as_ref()
            .expect("actual producer artifact reached its owner's step");
        use RelationalInitialInputsPublicationStop as Stop;
        match answer {
            Answer::NoEffect => {
                assert!(matches!(denied.publication_stop(), Some(Stop::Refused(_))));
                let complete = capture(&runtime, &request);
                assert_eq!(
                    initial(complete.source_artifact())["entries"]
                        .as_array()
                        .unwrap()
                        .len(),
                    1
                );
            }
            Answer::Indeterminate => {
                let Some(Stop::Unresolved(unresolved)) = denied.publication_stop() else {
                    panic!("typed unresolved initialization required");
                };
                assert_eq!(
                    unresolved.subject(),
                    RelationalDurabilityUnresolvedSubject::InitialInputs {
                        revision: artifact.revision(),
                        digest: artifact.payload_digest(),
                    }
                );
                assert!(matches!(runtime.archive_branch(&request, &main),
                    Err(RelationalBranchTransitionStop::DurabilityUnresolved(actual)) if &actual == unresolved));
                assert_eq!(current(&request, &runtime), original);
            }
        }
    }
}

#[test]
fn birth_admission_preserves_typed_no_effect_and_unresolved_publication_fates() {
    for answer in [Answer::NoEffect, Answer::Indeterminate] {
        let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&policy);
        let medium = TestDurableMedium::new();
        let offered = Arc::new(Mutex::new(None));
        let mut runtime = RelationalRuntimeApi::builder()
            .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
            .schema_registry(worth_testkit_relational::durable_publication::schema())
            .build_durable(Box::new(InitialStep {
                actual: medium.attachment(),
                answer,
                offered: offered.clone(),
            }))
            .unwrap();
        let original = current(&request, &runtime);
        let main = runtime.main_branch_identity();
        let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                &request,
                RelationalDurableRecoverySource::new(None, vec![], vec![], source),
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let denied = runtime
            .durability_recovery()
            .admit_fresh_home_source(&request, plan)
            .unwrap_err();
        let RelationalFreshSourceAdmissionStop::Publication(denied) = denied else {
            panic!("Birth must preserve its original publisher's typed outcome");
        };
        assert_eq!(current(&request, &runtime), original);
        assert!(medium.offered().is_empty());
        assert!(medium
            .artifact_bytes(worth_relational::facade::publication::PatchStreamPosition(
                1
            ))
            .is_none());
        let offered = offered.lock().unwrap();
        let artifact = offered
            .as_ref()
            .expect("the actual Birth reached its owner's step");
        assert_eq!(artifact.revision(), 1);
        match answer {
            Answer::NoEffect => assert!(matches!(
                denied,
                RelationalInitialInputsPublicationStop::Refused(
                    RelationalDurablePublicationRefusal::ProvenNoEffect { .. }
                )
            )),
            Answer::Indeterminate => {
                let RelationalInitialInputsPublicationStop::Unresolved(unresolved) = denied else {
                    panic!("Birth indeterminacy must carry its original unresolved subject");
                };
                assert_eq!(
                    unresolved.subject(),
                    RelationalDurabilityUnresolvedSubject::InitialInputs {
                        revision: artifact.revision(),
                        digest: artifact.payload_digest(),
                    }
                );
                assert!(matches!(runtime.archive_branch(&request, &main),
                    Err(RelationalBranchTransitionStop::DurabilityUnresolved(actual))
                        if actual == unresolved));
                assert_eq!(current(&request, &runtime), original);
            }
        }
    }
}
