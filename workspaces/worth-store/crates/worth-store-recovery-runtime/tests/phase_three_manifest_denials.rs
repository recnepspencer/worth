#[allow(dead_code)]
mod phase_three_support;

use phase_three_support::*;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};
use worth_store_recovery_runtime::{
    PhysicalManifestObservationDenial, PhysicalRecoveryIntegrityObservationOutcome,
    PhysicalRecoveryIntegrityRejection, PhysicalRecoveryRootProtocolDenial,
    PhysicalRecoverySourceDenial,
};

#[test]
fn missing_and_truncated_manifest_blocks_retain_exact_typed_denials() {
    let missing = manifest_case("missing-routing-block", |path| {
        std::fs::remove_file(path).unwrap();
    });
    assert!(matches!(
        manifest_denial(&missing),
        PhysicalManifestObservationDenial::Integrity {
            reference,
            denial: PhysicalRecoveryRootProtocolDenial::Absent,
        } if reference.generation() == 1 && reference.block() == 1
    ));
    assert_eq!(missing.evidence().integrity_observation_count(), 1);
    assert!(matches!(
        missing.evidence().integrity_observations(),
        [observation]
            if observation.scope().artifact_family()
                == PhysicalIntegrityArtifactFamily::RootRoutingBlock
                && observation.outcome()
                    == PhysicalRecoveryIntegrityObservationOutcome::Rejected(
                        PhysicalRecoveryIntegrityRejection::MissingBoundedArtifact,
                    )
    ));

    let undecodable = manifest_case("undecodable-routing-block", |path| {
        std::fs::write(path, b"not-a-durable-routing-block").unwrap();
    });
    assert!(matches!(
        manifest_denial(&undecodable),
        PhysicalManifestObservationDenial::Integrity {
            reference,
            denial: PhysicalRecoveryRootProtocolDenial::Integrity(
                PhysicalIntegrityRejection::Damaged(localization)
            ),
        } if reference.generation() == 1
            && reference.block() == 1
            && localization.cause() == PhysicalDamageCause::Truncated
    ));
    assert_eq!(undecodable.evidence().integrity_observation_count(), 1);
    assert!(matches!(
        undecodable.evidence().integrity_observations(),
        [observation]
            if observation.scope().artifact_family()
                == PhysicalIntegrityArtifactFamily::RootRoutingBlock
                && matches!(
                    observation.outcome(),
                    PhysicalRecoveryIntegrityObservationOutcome::Rejected(
                        PhysicalRecoveryIntegrityRejection::Integrity(
                            PhysicalIntegrityRejection::Damaged(localization)
                        )
                    ) if localization.cause() == PhysicalDamageCause::Truncated
                )
    ));
}

fn manifest_denial(
    blocked: &worth_store_recovery_runtime::PhysicalRecoveryBlock,
) -> &PhysicalManifestObservationDenial {
    blocked
        .evidence()
        .source_denials
        .iter()
        .find_map(|denial| match denial {
            PhysicalRecoverySourceDenial::ManifestObservation(denial) => Some(denial),
            _ => None,
        })
        .expect("the later typed manifest denial must remain present")
}

fn manifest_case(
    name: &str,
    mutate: impl FnOnce(&std::path::Path),
) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join(name);
    let store = initialize_store(&root);
    publish_synthetic_nonempty_genesis(&root, store);
    mutate(
        &root
            .join("families")
            .join("records")
            .join("roots")
            .join("root-0000000000000001-block-0000000000000001.manifest"),
    );
    expect_blocked(
        admitted_recovery(&root)
            .discover()
            .unwrap()
            .select()
            .err()
            .expect("manifest observation denial must block"),
    )
}
