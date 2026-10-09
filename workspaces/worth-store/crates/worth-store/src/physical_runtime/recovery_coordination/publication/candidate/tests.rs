use super::*;

fn candidates(
    generation: u64,
    publication: u64,
    head_generation: u64,
) -> Box<[PhysicalRecoveryPublicationCandidate]> {
    let mut artifacts = vec![
        RecordArtifactFile::RootManifest { generation },
        RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: head_generation,
            block: 1,
        },
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Previous,
            publication,
        },
        RecordArtifactFile::RootSelectorCandidate {
            role: RootSelectorRole::Current,
            publication,
        },
        RecordArtifactFile::CatalogCandidate { publication },
    ];
    artifacts.sort_unstable();
    artifacts
        .into_iter()
        .map(|artifact| {
            let bytes: Box<[u8]> = vec![1].into_boxed_slice();
            PhysicalRecoveryPublicationCandidate::new(artifact, bytes, Sha256::digest([1]).into())
                .unwrap()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[test]
fn release_head_candidate_keeps_publication_generation_and_closure_checks() {
    let generation = 13;
    let publication = 17;
    let protocol = RecoveryRootProtocolPublicationPlan::from_catalog_candidate(
        RecordArtifactFile::CatalogCandidate { publication },
    )
    .unwrap();
    assert!(super::super::PhysicalRecoveryPublicationCommand::new(
        [1; 32],
        generation,
        candidates(generation, publication, generation),
        protocol,
    )
    .is_some());
    assert!(super::super::PhysicalRecoveryPublicationCommand::new(
        [1; 32],
        generation,
        candidates(generation, publication, generation - 1),
        protocol,
    )
    .is_none());
    let mut missing_root = candidates(generation, publication, generation).into_vec();
    missing_root.retain(|candidate| {
        !matches!(
            candidate.artifact(),
            RecordArtifactFile::RootManifest { .. }
        )
    });
    assert!(super::super::PhysicalRecoveryPublicationCommand::new(
        [1; 32],
        generation,
        missing_root.into_boxed_slice(),
        protocol,
    )
    .is_none());
    let mut unordered = candidates(generation, publication, generation).into_vec();
    unordered.swap(0, 1);
    assert!(super::super::PhysicalRecoveryPublicationCommand::new(
        [1; 32],
        generation,
        unordered.into_boxed_slice(),
        protocol,
    )
    .is_none());
    assert!(super::super::PhysicalRecoveryPublicationCommand::new(
        [1; 32],
        generation,
        candidates(generation, publication + 1, generation),
        protocol,
    )
    .is_none());
}
