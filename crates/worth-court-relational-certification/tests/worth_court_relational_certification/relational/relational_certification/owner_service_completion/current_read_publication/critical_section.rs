//! Observe both actual branch state locks at the durable publication boundary.
use super::*;
use std::sync::{Arc, Mutex};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence,
    durability::{
        DurablePublicationFate, RelationalBranchTransitionArtifact,
        RelationalCanonicalCommitArtifact, RelationalDurableColdSource, RelationalDurableMedium,
        RelationalDurableTransitionReadFailure, RelationalInitialInputsPublicationDetail,
        RelationalSourceInitialInputsArtifact,
    },
    runtime::RelationalRuntimeApi,
};
use worth_testkit_relational::durable_medium::TestDurableMedium;

#[derive(Debug, Default)]
struct CutoverSample {
    pending: Option<(AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasis)>,
    locks: Option<(bool, bool)>,
}

#[derive(Debug)]
struct ObservedMedium {
    journal: Box<dyn RelationalDurableMedium>,
    sample: Arc<Mutex<CutoverSample>>,
}

impl RelationalDurableMedium for ObservedMedium {
    fn publish_initial_inputs(
        &self,
        artifact: &RelationalSourceInitialInputsArtifact,
        request: &ExecutionRequest<'_, '_>,
    ) -> DurablePublicationFate<RelationalInitialInputsPublicationDetail> {
        self.journal.publish_initial_inputs(artifact, request)
    }

    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.journal.cold_source()
    }

    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        self.journal.read_transition(sequence)
    }

    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        let mut sample = self.sample.lock().unwrap();
        if let Some((source, target)) = &sample.pending {
            sample.locks = Some((
                source.publication_state_locked_for_test(),
                target.publication_state_locked_for_test(),
            ));
        }
        drop(sample);
        self.journal.publish(artifact)
    }

    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.journal.publish_transition(artifact)
    }
}

#[test]
fn read_and_target_cells_stay_locked_at_actual_cutover_in_both_orders_and_same_cell() {
    for branch in ["aaa", "zzz", "main"] {
        let host = serial_request();
        let execution = &ExecutionRequest::serial(&host);
        let sample = Arc::new(Mutex::new(CutoverSample::default()));
        let medium = ObservedMedium {
            journal: TestDurableMedium::new().attachment(),
            sample: Arc::clone(&sample),
        };
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(fixture().build_registry())
            .build_durable(Box::new(medium))
            .unwrap();
        let original = create(execution, &runtime, "control", "allowed");
        let source = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let identity = if branch == "main" {
            runtime.main_branch_identity()
        } else {
            fork(execution, &runtime, branch)
        };
        let target = runtime.admit_branch_basis(&identity).unwrap();
        assert!(!source.publication_state_locked_for_test());
        assert!(!target.publication_state_locked_for_test());
        let mut candidate = prepare(execution, &runtime, &target);
        candidate.require_current_read(&source).unwrap();
        sample.lock().unwrap().pending = Some((source.clone(), target.clone()));
        let outcome = runtime
            .publication_port()
            .compare_and_publish(execution, candidate);
        let (source_locked, target_locked) = {
            let mut sample = sample.lock().unwrap();
            sample.pending = None;
            sample
                .locks
                .take()
                .expect("the original receiver reached durable publication")
        };
        assert!(
            source_locked,
            "C was not protected before the D replacement"
        );
        assert!(target_locked, "D was not protected at its own replacement");
        assert!(!source.publication_state_locked_for_test());
        assert!(!target.publication_state_locked_for_test());
        let RelationalPublicationOutcome::Performed(performed) = outcome else {
            panic!("the lawful actual cutover must perform: {outcome:?}");
        };
        let result = runtime
            .settlement_port()
            .settle_performed_publication(performed)
            .unwrap();
        let output = changed_entities(&result)[0];
        assert_eq!(
            values(execution, &runtime, &result, [output, output]),
            ["produced", "produced"].map(string_aspect_value)
        );
        release_test_commit_snapshot(&runtime, &result);
        release_test_commit_snapshot(&runtime, &original);
    }
}
