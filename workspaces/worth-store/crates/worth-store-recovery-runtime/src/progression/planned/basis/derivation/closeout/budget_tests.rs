use super::*;

#[test]
fn publication_actions_share_live_candidate_storage_and_deny_before_reservation() {
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let root = DurablePhysicalRootManifest::builder(6, 7, 4, 19)
        .admit()
        .unwrap();
    let bytes = root.encode(format).into_boxed_slice();
    let retained =
        bytes.len() as u64 + std::mem::size_of::<RecoveryPublicationCandidateArtifact>() as u64;
    let candidate = RecoveryPublicationCandidateArtifact {
        artifact: RecordArtifactFile::RootManifest { generation: 6 },
        payload_digest: sha2::Sha256::digest(&bytes).into(),
        bytes,
    };
    let slots = PlanningResidentAllowance::slot_bytes::<RecoveryPublicationAction>(4).unwrap();
    let mut denied = PlanningResidentAllowance::new(retained, retained + slots - 1).unwrap();
    assert_eq!(
        publication_actions(std::slice::from_ref(&candidate), &mut denied),
        Err(ExecutionBasisDenial::RecoveryMemoryBytes {
            observed: retained + slots
        })
    );
    assert_eq!(denied.used(), retained);
    let mut admitted = PlanningResidentAllowance::new(retained, retained + 2 * slots).unwrap();
    let actions = publication_actions(std::slice::from_ref(&candidate), &mut admitted).unwrap();
    assert_eq!(
        actions.as_ref(),
        &[
            RecoveryPublicationAction::MaterializeRootCandidate {
                artifact: candidate.artifact()
            },
            RecoveryPublicationAction::SynchronizeRootCandidate {
                artifact: candidate.artifact()
            },
            RecoveryPublicationAction::ReplaceRootProtocol,
            RecoveryPublicationAction::SynchronizeStoreNamespace,
        ]
    );
    assert_eq!(admitted.used(), retained + slots);
    assert_eq!(admitted.peak(), retained + 2 * slots);
}

#[test]
fn publication_allocation_preserves_real_cause_and_does_not_mislabel_image_construction() {
    let mut allowance = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
    let memory = allowance.reserve::<u8>(usize::MAX).unwrap_err();
    let denial = candidate_denial(memory.into());
    let ExecutionBasisDenial::PublicationCandidateAllocation {
        requested_bytes,
        cause,
    } = denial
    else {
        panic!("candidate allocation must preserve its responsible boundary");
    };
    assert_eq!(requested_bytes, usize::MAX as u64);
    assert_eq!(
        cause,
        Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err()
    );
}

#[test]
fn plan_body_slots_refuse_one_byte_short_and_overlap_under_allowance() {
    let base_slot = PlanningResidentAllowance::slot_bytes::<RecoveryBaseImagePlanBody>(1).unwrap();
    let pub_slot = PlanningResidentAllowance::slot_bytes::<RecoveryPublicationPlanBody>(1).unwrap();
    let exp_payload = PlanningResidentAllowance::slot_bytes::<ExpectationPayload>(1).unwrap();
    let exp_movement = pub_slot + 2 * exp_payload;
    let basis_slot = PlanningResidentAllowance::slot_bytes::<ExecutionBasisState>(1).unwrap();

    // Base-image body slot denies one byte short before allocating.
    let mut denied_base = PlanningResidentAllowance::new(19, 19 + base_slot - 1).unwrap();
    assert_eq!(
        denied_base.reserve::<RecoveryBaseImagePlanBody>(1),
        Err(PlanningMemoryDenial::RecoveryMemoryBytes {
            observed: 19 + base_slot
        })
    );
    assert_eq!((denied_base.used(), denied_base.peak()), (19, 19));

    // Publication body + expectation conversion overlap while base-image body stays live.
    let overlap_peak = (19 + base_slot + pub_slot + exp_payload + exp_movement)
        .max(19 + base_slot + pub_slot + exp_payload + 2 * basis_slot);
    let mut denied_overlap = PlanningResidentAllowance::new(19, overlap_peak - 1).unwrap();
    let _base_inner = denied_overlap.reserve::<RecoveryBaseImagePlanBody>(1).unwrap();
    let _pub_inner = denied_overlap
        .reserve::<RecoveryPublicationPlanBody>(1)
        .unwrap();
    let failure = ExpectationBacking::prepare(&mut denied_overlap)
        .and_then(|_exp| ExecutionBasisBacking::prepare(&mut denied_overlap));
    assert!(matches!(
        failure,
        Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) if observed == overlap_peak
    ));

    let mut admitted = PlanningResidentAllowance::new(19, overlap_peak).unwrap();
    let base_inner = admitted.reserve::<RecoveryBaseImagePlanBody>(1).unwrap();
    let pub_inner = admitted.reserve::<RecoveryPublicationPlanBody>(1).unwrap();
    let exp_backing = ExpectationBacking::prepare(&mut admitted).unwrap();
    let basis_backing = ExecutionBasisBacking::prepare(&mut admitted).unwrap();
    let expected_used = 19
        + PlanningResidentAllowance::vector_bytes(&base_inner).unwrap()
        + PlanningResidentAllowance::vector_bytes(&pub_inner).unwrap()
        + exp_backing.backing_bytes().unwrap()
        + basis_slot;
    assert_eq!(admitted.used(), expected_used);
    assert_eq!(admitted.peak(), overlap_peak);
    drop((basis_backing, exp_backing, pub_inner, base_inner));
}
