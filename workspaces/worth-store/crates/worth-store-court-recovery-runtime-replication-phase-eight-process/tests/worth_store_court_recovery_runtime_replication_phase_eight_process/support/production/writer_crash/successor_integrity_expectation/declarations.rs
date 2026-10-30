//! Earlier C9 rejection is checked by named cause, not flattened to legacy invalidity.
pub(crate) use {
    worth_store_physical_integrity::PhysicalBlastRadius,
    worth_store_physical_integrity::PhysicalByteRange,
    worth_store_physical_integrity::PhysicalDamageCause as Cause,
    worth_store_physical_integrity::PhysicalFormatField as Field,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial as Planning,
    worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial as Root,
    worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial as Candidate,
};

pub(crate) fn require(actual: Option<Planning>, expected: Planning, hostile: &str) {
    if matches!(hostile, "inflated" | "selected-routing-root") {
        assert_eq!(actual, Some(expected), "{hostile}");
        return;
    }
    let worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
        worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::InvalidArtifact {
            artifact: expected_artifact,
            generation: expected_generation,
        },
    ) = expected
    else {
        panic!("named expected successor artifact")
    };
    let Some(worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(
        worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::RootProtocol {
            artifact,
            generation,
            denial:
                worth_store_recovery_runtime::PhysicalRecoveryRootProtocolDenial::Integrity(
                    worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
                        localization,
                    ),
                ),
        },
    )) = actual
    else {
        panic!("expected early C9 successor rejection for {hostile}: {actual:?}")
    };
    assert_eq!(artifact, expected_artifact);
    assert_eq!(generation, expected_generation);
    let (cause, range, field, blast) = if hostile == "conflicting" {
        (
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalByteRange::new(48, 8).unwrap(),
            worth_store_physical_integrity::PhysicalFormatField::TreeIdentity,
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        )
    } else {
        assert!(matches!(
            hostile,
            "root-routing-child" | "segment-membership-child" | "free-space-child"
        ));
        (
            worth_store_physical_integrity::PhysicalDamageCause::WrongMagic,
            worth_store_physical_integrity::PhysicalByteRange::new(0, 8).unwrap(),
            worth_store_physical_integrity::PhysicalFormatField::Magic,
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        )
    };
    assert_eq!(localization.cause(), cause, "{hostile}");
    assert_eq!(localization.damaged_range(), range, "{hostile}");
    assert_eq!(localization.field(), Some(field), "{hostile}");
    assert_eq!(localization.blast_radius(), blast, "{hostile}");
}
