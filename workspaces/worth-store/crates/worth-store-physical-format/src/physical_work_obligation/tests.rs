use super::*;

const DIGEST: [u8; 32] = [0xa5; 32];

#[test]
fn v6_round_trips_every_target_code() {
    let artifact = PhysicalWorkArtifactCode::BootstrapCatalog;
    let targets = [
        (
            PhysicalWorkObligationTargetCode::Range {
                artifact,
                offset: 9,
                byte_count: 17,
            },
            Some(DIGEST),
        ),
        (
            PhysicalWorkObligationTargetCode::WalArtifactInterval {
                segment: 4,
                generation: 7,
                offset: 8,
                byte_count: 11,
            },
            Some(DIGEST),
        ),
        (
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence: 8,
                action: PhysicalWorkCheckpointActionCode::CreateCandidate { byte_count: 113 },
            },
            Some(DIGEST),
        ),
        (
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence: 8,
                action: PhysicalWorkCheckpointActionCode::AppendCandidate {
                    offset: 113,
                    byte_count: 29,
                },
            },
            Some(DIGEST),
        ),
        (
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence: 8,
                action: PhysicalWorkCheckpointActionCode::SynchronizeCandidate,
            },
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence: 8,
                action: PhysicalWorkCheckpointActionCode::RemoveCandidate,
            },
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence: 8,
                action: PhysicalWorkCheckpointActionCode::PublishCandidate,
            },
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::Checkpoint {
                sequence: 8,
                action: PhysicalWorkCheckpointActionCode::SynchronizeNamespace,
            },
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::ArtifactFileSynchronization(artifact),
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::ArtifactParentSynchronization(artifact),
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::CatalogReplacement(
                PhysicalWorkArtifactCode::CatalogCandidate { publication: 11 },
            ),
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization,
            None,
        ),
        (
            PhysicalWorkObligationTargetCode::WalSegmentReclamation {
                segment: 17,
                generation: 3,
            },
            None,
        ),
    ];
    for (operation, (target, digest)) in targets.into_iter().enumerate() {
        let value = PhysicalWorkObligationV6::new(
            [7; 16],
            1,
            2,
            operation as u64 + 1,
            PhysicalWorkObligationOperationCode::RootPublication,
            target,
            digest,
        )
        .unwrap();
        let encoded = encode_physical_work_obligation_v6(value);
        assert_eq!(decode_physical_work_obligation_v6(&encoded), Ok(value));
    }
}

#[test]
fn v6_rejects_checksum_reserved_and_target_shape_damage() {
    let value = PhysicalWorkObligationV6::new(
        [7; 16],
        1,
        2,
        3,
        PhysicalWorkObligationOperationCode::WalReclamation,
        PhysicalWorkObligationTargetCode::WalSegmentReclamation {
            segment: 17,
            generation: 3,
        },
        None,
    )
    .unwrap();
    let mut encoded = encode_physical_work_obligation_v6(value);
    encoded[10] = 1;
    assert_eq!(
        decode_physical_work_obligation_v6(&encoded),
        Err(PhysicalWorkObligationV6Denial::ReservedFieldNonZero)
    );
    let mut encoded = encode_physical_work_obligation_v6(value);
    encoded[64] = 1;
    let checksum = super::checksum::calculate(&encoded);
    encoded[128..].copy_from_slice(&checksum);
    assert_eq!(
        decode_physical_work_obligation_v6(&encoded),
        Err(PhysicalWorkObligationV6Denial::InvalidTarget)
    );
}
