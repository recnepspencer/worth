//! Earlier C9 rejection is checked by named cause, not flattened to legacy invalidity.
use worth_store_physical_integrity::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause as Cause,
    PhysicalFormatField as Field, PhysicalIntegrityRejection,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryPlanningDenial as Planning, PhysicalRecoveryRootProtocolDenial as Root,
    PhysicalRecoverySuccessorCandidateDenial as Candidate,
};

pub(super) fn require(actual: Option<Planning>, expected: Planning, hostile: &str) {
    if matches!(hostile, "inflated" | "selected-routing-root") {
        assert_eq!(actual, Some(expected), "{hostile}");
        return;
    }
    let Planning::SuccessorCandidate(Candidate::InvalidArtifact {
        artifact: expected_artifact,
        generation: expected_generation,
    }) = expected
    else {
        panic!("named expected successor artifact")
    };
    let Some(Planning::SuccessorCandidate(Candidate::RootProtocol {
        artifact,
        generation,
        denial: Root::Integrity(PhysicalIntegrityRejection::Damaged(localization)),
    })) = actual
    else {
        panic!("expected early C9 successor rejection for {hostile}: {actual:?}")
    };
    assert_eq!(artifact, expected_artifact);
    assert_eq!(generation, expected_generation);
    let (cause, range, field, blast) = if hostile == "conflicting" {
        (
            Cause::ArtifactIdentityMismatch,
            PhysicalByteRange::new(48, 8).unwrap(),
            Field::TreeIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else {
        assert!(matches!(
            hostile,
            "root-routing-child" | "segment-membership-child" | "free-space-child"
        ));
        (
            Cause::WrongMagic,
            PhysicalByteRange::new(0, 8).unwrap(),
            Field::Magic,
            PhysicalBlastRadius::CompleteArtifact,
        )
    };
    assert_eq!(localization.cause(), cause, "{hostile}");
    assert_eq!(localization.damaged_range(), range, "{hostile}");
    assert_eq!(localization.field(), Some(field), "{hostile}");
    assert_eq!(localization.blast_radius(), blast, "{hostile}");
}
