use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, PersistedRecordIdentity,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1,
};

// A format-owner planned first upsert, not a fabricated recovery authority.
fn effect_fixture() -> (
    PersistedReleaseCustodyHeadEffectV1,
    PhysicalRecordFormatDeclaration,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = |ordinal| PersistedRecordIdentity::new([1; 16], ordinal).unwrap();
    let publication = BlobGenerationPublicationV1::new(
        [7; 16],
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    let basis = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap();
    let next = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation()).unwrap(),
        record(9),
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(publication.store()),
        None,
        11,
        1,
        false,
    )
    .unwrap();
    let limits = ReleaseCustodyHeadTransitionLimitsV1::new(1, 3, 64 * 1024).unwrap();
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        None,
        1,
        &[],
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next,
        },
        12,
        6,
        format,
        limits,
    )
    .unwrap();
    let effect = PersistedReleaseCustodyHeadEffectV1::new_upsert(
        6,
        11,
        basis,
        vec![],
        planned,
        format,
        limits,
    )
    .unwrap();
    (effect, format)
}

#[test]
fn effect_observation_denies_joint_window_before_actual_read() {
    let (effect, format) = effect_fixture();
    let count = effect.source_path().len() + effect.node_writes().len();
    let required = effect
        .verification_additional_peak_bytes(format)
        .unwrap()
        .max(
            count as u64 * std::mem::size_of::<SelectedArtifactSlice>() as u64
                + u64::from(format.page_size().bytes()),
        );
    let mut reads = 0;
    let denied = observe_effect_with_read(&effect, 11, format, required - 1, |_, _| {
        reads += 1;
        Err(Denial::MissingFrame)
    });
    assert!(matches!(denied, Err(Denial::BoundExceeded)));
    assert_eq!(reads, 0);

    let witnessed = observe_effect_with_read(&effect, 11, format, required, |reference, _| {
        reads += 1;
        Ok(effect
            .node_writes()
            .iter()
            .find(|write| write.reference() == reference)
            .unwrap()
            .frame()
            .to_vec())
    })
    .expect("same genuine format effect fits the exact admitted observation window");
    assert_eq!(reads, count);
    assert_eq!(witnessed.len(), count);
}

#[test]
fn effect_observation_rejects_changed_actual_frame() {
    let (effect, format) = effect_fixture();
    let mut reads = 0;
    let denied = observe_effect_with_read(&effect, 11, format, 1024 * 1024, |reference, _| {
        reads += 1;
        let mut bytes = effect
            .node_writes()
            .iter()
            .find(|write| write.reference() == reference)
            .unwrap()
            .frame()
            .to_vec();
        bytes[0] ^= 1;
        Ok(bytes)
    });
    assert!(matches!(denied, Err(Denial::ControlFrame)));
    assert_eq!(reads, 1);
}
