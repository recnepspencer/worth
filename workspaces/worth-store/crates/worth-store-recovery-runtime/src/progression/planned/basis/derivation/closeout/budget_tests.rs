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
