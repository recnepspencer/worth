//! B5 negative court: a disposed World original whose Native image or commit is
//! still present must still reject contradictory digest or position coordinates.
use super::*;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::{CanonicalDigestId, WorldOriginalFate};
use worth_proof::{HomeIncarnationFence, HomeWorldSelectionOwner, VerifiedHomeWorldSelection};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence,
    durability::{
        DurablePublicationFate, RelationalBranchTransitionArtifact,
        RelationalCanonicalCommitArtifact, RelationalDurableColdSource, RelationalDurableMedium,
        RelationalDurableTransitionReadFailure, RelationalRecoveryStop,
        RelationalWorldEffectCoordinates, RelationalWorldOriginalDescription,
        RelationalWorldSourceCatalog, RelationalWorldSourceDescription as Source,
    },
    publication::PatchStreamPosition,
    runtime::RelationalRuntimeApi,
};

#[derive(Debug)]
struct SelectionMedium {
    inner: Box<dyn RelationalDurableMedium>,
    selection: Arc<VerifiedHomeWorldSelection<Arc<RelationalWorldSourceCatalog>>>,
}

impl RelationalDurableMedium for SelectionMedium {
    fn home_world_selection(
        &self,
    ) -> Option<Arc<VerifiedHomeWorldSelection<Arc<RelationalWorldSourceCatalog>>>> {
        Some(self.selection.clone())
    }
    fn checkpoint_selection(
        &self,
        subject: &worth_relational::facade::durability::OpaqueCapturedCheckpointSubject,
    ) -> worth_relational::facade::durability::CheckpointSelectionObservation {
        self.inner.checkpoint_selection(subject)
    }
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.inner.cold_source()
    }
    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        self.inner.read_transition(sequence)
    }
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        self.inner.publish(artifact)
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.inner.publish_transition(artifact)
    }
}

fn recover_with_catalog(
    execution: &ExecutionRequest<'_, '_>,
    medium: &Arc<TestDurableMedium>,
    input: worth_relational::facade::durability::RelationalDurableRecoverySource,
    use_checkpoint: bool,
    catalog: Arc<RelationalWorldSourceCatalog>,
) -> Result<(), RelationalRecoveryStop> {
    let owner = HomeWorldSelectionOwner::from_retained_home_owner(
        [0x5A; 16],
        HomeIncarnationFence::from_owner(std::num::NonZeroU64::new(1).unwrap()),
        Arc::new(()),
    );
    let authority = owner.authority();
    let (admitted, _completion) = owner.admit_source(catalog);
    let verified = Arc::new(authority.verify_source(admitted).unwrap());
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
        .schema_registry(worth_testkit_relational::durable_publication::schema())
        .build_durable(Box::new(SelectionMedium {
            inner: medium.attachment(),
            selection: verified,
        }))
        .unwrap();
    if use_checkpoint {
        runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(worth_testkit_relational::initial_input_source::additions())
            .unwrap();
    }
    let plan = runtime.durability().recovery_plan_from_source(
        execution,
        input,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )?;
    let _ = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)?;
    Ok(())
}

fn build_catalog(
    execution: &ExecutionRequest<'_, '_>,
    head: Recorded,
    original_source: Source,
) -> Arc<RelationalWorldSourceCatalog> {
    let mut originals = ExecutionRequestArrayBuilder::allocate(*execution, 1).unwrap();
    originals
        .push(
            RelationalWorldOriginalDescription::from_recorded(
                execution,
                CanonicalDigestId::new([0x77; 32]),
                original_source,
                WorldOriginalFate::Released,
                None,
                None,
            )
            .unwrap()
            .with_source_disposed(true),
        )
        .unwrap();
    let mut heads = ExecutionRequestArrayBuilder::allocate(*execution, 1).unwrap();
    heads.push(Source::Recorded(head)).unwrap();
    RelationalWorldSourceCatalog::from_recorded(
        execution,
        originals.seal().unwrap(),
        heads.seal().unwrap(),
        None,
    )
    .unwrap()
}

#[test]
fn disposed_original_source_still_rejects_contradictory_digest_and_position_when_present() {
    use worth_testkit_relational::initial_input_source::{
        capture_external, home_image_input, initialized,
    };
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for use_checkpoint in [false, true] {
        let (source, medium) = if use_checkpoint {
            initialized(execution)
        } else {
            let m = TestDurableMedium::new();
            (durable_runtime(&m), m)
        };
        let assoc1 = Association::from_bytes(b"disposed-source-predecessor-assoc").unwrap();
        let p1 = perform(execution, &source, &assoc1, "disposed-source-predecessor");
        let before_basis = p1.next_basis().clone();
        let before_lease = source.retain_component_basis(&before_basis).unwrap();
        let r1 = source.settle_performed_publication(p1).unwrap();
        release_test_commit_snapshot(&source, &r1);
        let before = source
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(execution, &before_basis)
            .unwrap();
        let assoc2 = Association::from_bytes(b"later-live-head-assoc").unwrap();
        let p2 = perform(execution, &source, &assoc2, "later-live-head");
        let head_basis = p2.next_basis().clone();
        let r2 = source.settle_performed_publication(p2).unwrap();
        release_test_commit_snapshot(&source, &r2);
        let head = source
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(execution, &head_basis)
            .unwrap();
        drop(head_basis);
        let complete = use_checkpoint.then(|| {
            let c = capture_external(&source, &medium, execution);
            medium.file_checkpoint(c.checkpoint().clone());
            c
        });
        source.release_component_basis(before_lease).unwrap();
        drop(before_basis);
        drop(source);
        let make_input = || match &complete {
            Some(c) => home_image_input(&medium, c, execution).unwrap(),
            None => medium.recovery_source(),
        };

        // 1. Untampered disposed Source::Recorded succeeds.
        recover_with_catalog(
            execution,
            &medium,
            make_input(),
            use_checkpoint,
            build_catalog(execution, head.clone(), Source::Recorded(before.clone())),
        )
        .expect("exact disposed recorded source with available image must recover");

        // 2. Matching image descriptor & receipt with mutated artifact_digest MUST fail.
        let mut wrong_digest = serde_json::to_value(&before).unwrap();
        let digest: CanonicalDigestId =
            serde_json::from_value(wrong_digest["target"]["Canonical"]["artifact_digest"].clone())
                .unwrap();
        let mut bytes = *digest.bytes();
        bytes[0] ^= 0xFF;
        wrong_digest["target"]["Canonical"]["artifact_digest"] =
            serde_json::to_value(CanonicalDigestId::new(bytes)).unwrap();
        let wrong_digest: Recorded = serde_json::from_value(wrong_digest).unwrap();
        assert!(matches!(
            recover_with_catalog(
                execution,
                &medium,
                make_input(),
                use_checkpoint,
                build_catalog(execution, head.clone(), Source::Recorded(wrong_digest)),
            ),
            Err(RelationalRecoveryStop::Durability(_))
        ));

        // 3. Matching image descriptor & receipt with mutated position MUST fail.
        let mut wrong_position = serde_json::to_value(&before).unwrap();
        let pos = wrong_position["target"]["Canonical"]["position"]
            .as_u64()
            .unwrap();
        wrong_position["target"]["Canonical"]["position"] = serde_json::json!(pos + 99);
        let wrong_position: Recorded = serde_json::from_value(wrong_position).unwrap();
        assert!(matches!(
            recover_with_catalog(
                execution,
                &medium,
                make_input(),
                use_checkpoint,
                build_catalog(execution, head.clone(), Source::Recorded(wrong_position)),
            ),
            Err(RelationalRecoveryStop::Durability(_))
        ));

        // 4. Source::Published with mutated digest MUST fail even when source_disposed = true.
        let Target::Canonical(target) = before.target() else {
            panic!("expected canonical source");
        };
        let mut bad_bytes = *target.artifact_digest();
        bad_bytes[0] ^= 0xAA;
        let bad_published = Source::Published(RelationalWorldEffectCoordinates::from_recorded(
            target.receipt().commit_id,
            PatchStreamPosition(target.position().0),
            CanonicalDigestId::new(bad_bytes),
        ));
        assert!(matches!(
            recover_with_catalog(
                execution,
                &medium,
                make_input(),
                use_checkpoint,
                build_catalog(execution, head, bad_published),
            ),
            Err(RelationalRecoveryStop::Durability(_))
        ));
    }
}
