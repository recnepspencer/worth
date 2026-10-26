//! A durable fork with a lost answer remains World-owned recovery work.
use super::*;
use std::sync::{Arc, Mutex};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalBranchTransitionArtifact,
    RelationalCanonicalCommitArtifact, RelationalDurableColdSource, RelationalDurableMedium,
    RelationalDurableMemberLocator, RelationalDurablePublication, RelationalDurableRecoverySource,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::storage::RelationalStorageUnavailableCause;

fn schema() -> worth_relational::facade::schema::RelationalSchemaRegistry {
    use worth_relational::facade::schema::{
        EntityKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    };
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: worth_relational::facade::identity::KindId(1),
            kind_name: "source".into(),
            schema_id: SchemaId("world-fork".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
}

#[derive(Debug, Default)]
struct Journal {
    commits: Vec<RelationalCanonicalCommitArtifact>,
    transitions: Vec<RelationalBranchTransitionArtifact>,
}

#[derive(Debug)]
struct LostForkAnswer(Arc<Mutex<Journal>>);
impl RelationalDurableColdSource for LostForkAnswer {
    fn read_member(
        &self,
        _locator: RelationalDurableMemberLocator,
        _limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        // This fixture journals artifacts without publishing a checkpoint.
        Err(RelationalStorageUnavailableCause::SourceUnavailable)
    }

    fn read_commit(
        &self,
        position: worth_relational::facade::publication::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        let journal = self.0.lock().unwrap();
        let artifact = journal
            .commits
            .iter()
            .find(|artifact| artifact.position() == position)
            .ok_or(RelationalStorageUnavailableCause::SourceUnavailable)?;
        let bytes = artifact.as_bytes();
        if bytes.len() as u64 > limit {
            return Err(RelationalStorageUnavailableCause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.to_vec())
    }
}
impl RelationalDurableMedium for LostForkAnswer {
    fn read_transition(
        &self,
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<
        RelationalBranchTransitionArtifact,
        worth_relational::facade::durability::RelationalDurableTransitionReadFailure,
    > {
        self.0.lock().unwrap().transitions.iter().find(|artifact| artifact.sequence() == sequence).cloned()
            .ok_or(worth_relational::facade::durability::RelationalDurableTransitionReadFailure::Missing(sequence))
    }

    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        self.0.lock().unwrap().commits.push(artifact.clone());
        RelationalDurablePublication::Committed
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        self.0.lock().unwrap().transitions.push(artifact.clone());
        RelationalDurablePublication::Indeterminate {
            detail: "answer lost after append".into(),
        }
    }
}

#[test]
fn an_unresolved_durable_fork_retains_its_attempt_until_reopen() {
    let journal = Arc::new(Mutex::new(Journal::default()));
    let relational = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build_durable(Box::new(LostForkAnswer(Arc::clone(&journal))));
    let mut fixture = crate::branch::reference_test_fixture::real_fixture_with_relational_runtime(
        relational, 12, 12,
    );
    let owner = TestOwner::new(fixture.owner_inputs(
        fork_budgets(3, 4),
        RuntimeWorldClock::from_source(super::super::FixedClock),
    ))
    .unwrap();
    let initial = bootstrap_root(&owner, &mut fixture);
    seed_relational_source(&owner, &mut fixture, initial);
    let source = current_root_observation(&owner);
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let cancellation = RuntimeWorldCancellationSource::new();
    let outcome = owner
        .create_product_branch(RuntimeWorldBranchCreationRequest::new(
            execution,
            source.clone(),
            fork_intent(
                "unresolved-product",
                relational_fork("unresolved-native"),
                signal_fork("untouched-signal"),
            ),
            &cancellation.token(),
        ))
        .expect("a possibly durable effect cannot be an admission denial");
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(retained) = outcome else {
        panic!("unresolved fork cannot publish a product branch");
    };
    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::RelationalDurabilityUnresolved
    );
    assert_eq!(retained.owner_effect_count(), 1);
    assert_eq!(
        retained.progress().relational_posture(),
        crate::publication::RelationalAttemptProgressPosture::DurabilityUnresolved
    );
    assert_eq!(
        retained.progress().signal_posture(),
        crate::publication::SignalAttemptProgressPosture::Untouched
    );
    assert_eq!(
        retained.next_actions(),
        [
            crate::recovery::ProductUnpublishedNextAction::ReleaseObligations,
            crate::recovery::ProductUnpublishedNextAction::Inspect,
            crate::recovery::ProductUnpublishedNextAction::CloseOwner,
        ]
    );
    assert_eq!(owner.recovery_record_count(), 1);
    assert_eq!(
        current_root_observation(&owner).snapshot(),
        source.snapshot()
    );
    let handle = owner.recovery_handles().pop().unwrap();
    assert!(owner
        .inspect_recovery(&handle)
        .unwrap()
        .destination_branch()
        .is_some());

    let durable = journal.lock().unwrap();
    assert_eq!(
        durable.transitions.len(),
        1,
        "the medium really retained the fork"
    );
    let recovery_source = RelationalDurableRecoverySource::new(
        None,
        durable.commits.clone(),
        durable.transitions.clone(),
        Arc::new(LostForkAnswer(Arc::clone(&journal))),
    );
    drop(durable);
    let mut reopened = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build();
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery_source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(reopened
        .branch_identity(&BranchId("unresolved-native".into()))
        .is_ok());
}
