//! The live original ACK is the same fork recovered from the real Complete image.
use super::*;
use worth_relational::facade::{
    branch::{RelationalBranchTransition, RelationalBranchTransitionStop, RelationalForkDenial},
    history::BranchId,
    mvcc::RelationalCallerAssociation,
};

#[test]
fn associated_fork_preserves_original_acknowledgement_and_complete_image_provenance() {
    associated_fork_reopens(false);
}

#[test]
fn associated_fork_tail_reopens_with_original_acknowledgement_and_exact_source() {
    associated_fork_reopens(true);
}

fn associated_fork_reopens(capture_before_fork: bool) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let (source, medium) = initialized(request);
    let commit = create_entity_outcome(request, &source, "source-root");
    release_test_commit_snapshot(&source, &commit);
    let before = current(request, &source);
    let prior_capture = capture_before_fork.then(|| capture_external(&source, &medium, request));
    let services = source.owner_component_services();
    let port = services.fork_port();
    let (_, admitted_source) = port.observe_fork_source(&BranchId("main".into())).unwrap();
    let reservation = port
        .reserve_fork_target(BranchId("sibling".into()))
        .unwrap();
    let association =
        RelationalCallerAssociation::from_bytes(b"original-world-creation-attempt").unwrap();
    let performed = port
        .fork_reserved_associated_with_basis(
            request,
            reservation,
            admitted_source,
            association.clone(),
        )
        .unwrap();
    let (outcome, basis, proof) = performed.into_parts();
    let after = services
        .basis_port()
        .export_recorded_current_component(request, &basis)
        .unwrap();
    let original = proof.consume(|evidence| {
        let persisted = medium.recovery_source().transitions()[0].clone();
        assert_eq!(
            evidence.artifact_digest(),
            persisted.payload_digest().as_bytes()
        );
        assert_eq!(
            evidence.positioned().caller_association(),
            Some(&association)
        );
        let admitted =
            RelationalBranchTransitionArtifact::readmit(persisted.as_bytes(), request).unwrap();
        assert_eq!(admitted, *evidence.positioned());
        evidence.positioned().clone()
    });
    assert!(
        matches!(original.transition(), RelationalBranchTransition::Fork { target, .. } if target == outcome.target_identity().branch_id())
    );
    let digest = original.recorded_artifact_digest().unwrap();
    let sequence = original.sequence();
    let capture = prior_capture.unwrap_or_else(|| capture_external(&source, &medium, request));
    medium.file_checkpoint(capture.checkpoint().clone());
    let input = home_image_input(&medium, &capture, request).unwrap();
    let reads = medium.canonical_member_reads();
    drop((capture, basis, outcome, services, port, source));
    let mut restored = receiver(&medium);
    let plan = restored
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = restored
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    let admitted = authority
        .admit_recorded_fork(
            request,
            &restored,
            sequence,
            &association,
            &digest,
            &before,
            &after,
        )
        .unwrap();
    assert_eq!(admitted.original(), &original);
    assert_eq!(admitted.original().recorded_artifact_digest(), Some(digest));
    let (_, target, _) = admitted.into_parts();
    assert_eq!(
        target.into_basis().descriptor().runtime_instance_id(),
        restored.runtime_instance_id()
    );
    assert_eq!(medium.canonical_member_reads(), reads);

    let foreign = RelationalCallerAssociation::from_bytes(b"unrelated-attempt").unwrap();
    assert!(matches!(
        authority
            .admit_recorded_fork(request, &restored, sequence, &foreign, &digest, &before, &after)
            .unwrap_err()
            .cause(),
        RelationalForkReadmissionCause::AssociationMismatch
    ));
    let mut changed = digest;
    changed[0] ^= 1;
    assert!(matches!(
        authority
            .admit_recorded_fork(
                request,
                &restored,
                sequence,
                &association,
                &changed,
                &before,
                &after
            )
            .unwrap_err()
            .cause(),
        RelationalForkReadmissionCause::ArtifactDigestMismatch
    ));
    let unrelated = worth_testkit_relational::durable_publication::memory_runtime();
    assert!(matches!(
        authority
            .admit_recorded_fork(
                request,
                &unrelated,
                sequence,
                &association,
                &digest,
                &before,
                &after
            )
            .unwrap_err()
            .cause(),
        RelationalForkReadmissionCause::ForeignRuntime
    ));
}

#[test]
fn memory_only_fork_cannot_issue_an_associated_durable_publication() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = worth_testkit_relational::durable_publication::memory_runtime();
    let commit = create_entity_outcome(request, &source, "memory-source");
    release_test_commit_snapshot(&source, &commit);
    let port = source.owner_component_services().fork_port();
    let (_, source_basis) = port.observe_fork_source(&BranchId("main".into())).unwrap();
    let target = BranchId("never-installed".into());
    let reservation = port.reserve_fork_target(target.clone()).unwrap();
    let association = RelationalCallerAssociation::from_bytes(b"world-attempt").unwrap();
    assert!(matches!(
        port.fork_reserved_associated_with_basis(request, reservation, source_basis, association),
        Err(RelationalBranchTransitionStop::Denied(
            RelationalForkDenial::DurableAssociationRequiresMedium
        ))
    ));
    assert!(source.branch_identity(&target).is_err());
}

#[test]
fn fork_artifact_refuses_a_missing_required_association_posture() {
    use sha2::{Digest, Sha256};
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let (source, medium) = initialized(request);
    let commit = create_entity_outcome(request, &source, "required-association");
    release_test_commit_snapshot(&source, &commit);
    let port = source.owner_component_services().fork_port();
    let (_, basis) = port.observe_fork_source(&BranchId("main".into())).unwrap();
    let target = port
        .reserve_fork_target(BranchId("required-target".into()))
        .unwrap();
    let association =
        RelationalCallerAssociation::from_bytes(b"required-creation-association").unwrap();
    let performed = port
        .fork_reserved_associated_with_basis(request, target, basis, association.clone())
        .unwrap();
    let original = medium.recovery_source().transitions()[0].clone();
    RelationalBranchTransitionArtifact::readmit(original.as_bytes(), request).unwrap();
    let key = rmp_serde::to_vec("caller_association").unwrap();
    let value = rmp_serde::to_vec(&association).unwrap();
    let mut suffix = key;
    suffix.extend_from_slice(&value);
    assert!(original.as_bytes().ends_with(&suffix));
    let mut missing = original.as_bytes()[..original.as_bytes().len() - suffix.len()].to_vec();
    assert_eq!(missing[66], 0x85);
    missing[66] = 0x84;
    let length = missing.len() - 66;
    missing[58..66].copy_from_slice(&(length as u64).to_le_bytes());
    let digest = Sha256::digest(&missing[66..]);
    missing[26..58].copy_from_slice(&digest);
    RelationalBranchTransitionArtifact::from_untrusted_bytes(missing.clone()).unwrap();
    assert!(matches!(
        RelationalBranchTransitionArtifact::readmit(&missing, request),
        Err(RelationalBranchTransitionArtifactRefusal::NonCanonicalPayload)
    ));
    drop(performed);
}

#[test]
fn stopped_association_funding_preserves_the_request_cause_without_a_fork() {
    use worth_execution::{ExecutionWorkCeiling, MapKernelStop, WorkCeilingDenial};
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&policy);
    let (source, _) = initialized(request);
    let commit = create_entity_outcome(request, &source, "stopped-association");
    release_test_commit_snapshot(&source, &commit);
    let port = source.owner_component_services().fork_port();
    let (_, basis) = port.observe_fork_source(&BranchId("main".into())).unwrap();
    let name = BranchId("stopped-fork".into());
    let target = port.reserve_fork_target(name.clone()).unwrap();
    let association = RelationalCallerAssociation::from_bytes(b"stopped-original").unwrap();
    let (stopped, _) = request
        .run(ExecutionWorkCeiling::new(Some(0)), |_| {
            request.run_native_with_checkpoint(|context| {
                assert_eq!(context.checkpoint(1), Err(MapKernelStop::WorkCeiling));
                assert!(matches!(
                    port.fork_reserved_associated_with_basis(request, target, basis, association),
                    Err(RelationalBranchTransitionStop::RequestStopped(
                        WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)
                    ))
                ));
            })
        })
        .unwrap();
    assert!(stopped.is_err());
    assert!(source.branch_identity(&name).is_err());
    drop(port.reserve_fork_target(name).unwrap());
}
