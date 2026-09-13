//! C8 root-protocol refusals are separate from record/WAL ingress observations.
use crate::c9_integrity_localization::{
    artifact_edit::ArtifactOperator as Op, artifact_inventory::ArtifactGranule,
    process_integrity_projection::project_integrity_scope, process_recovery_observation::*,
};

pub(super) fn require(
    clean: &ProcessRecoveryObservation,
    observed: &ProcessRecoveryObservation,
    target: &ArtifactGranule,
    operator: Op,
) -> bool {
    let expected = project_integrity_scope(target.scope);
    let artifact = match target.family {
        "bootstrap_catalog" => ProcessRootProtocolArtifact::BootstrapCatalog,
        "current_root_selector" => ProcessRootProtocolArtifact::CurrentSelector,
        "previous_root_selector" => ProcessRootProtocolArtifact::PreviousSelector,
        "root_manifest" => ProcessRootProtocolArtifact::CurrentRoot {
            generation: target.scope.root_generation().unwrap(),
        },
        _ => return false,
    };
    let denials = observed
        .root_protocol_denials
        .iter()
        .filter(|denial| denial.artifact == artifact)
        .collect::<Vec<_>>();
    let selected = clean.discovery.map_or(0, |discovery| match target.family {
        "current_root_selector" => discovery.current_selector_integrity_admissions,
        "previous_root_selector" => discovery.previous_selector_integrity_admissions,
        "root_manifest" => discovery.current_root_integrity_admissions,
        // C8 discovers from selectors; bootstrap is not an invented prerequisite.
        "bootstrap_catalog" => 0,
        _ => unreachable!(),
    });
    if selected == 0 {
        assert!(denials.is_empty(), "unselected root source: {artifact:?}");
        return true;
    }
    assert_eq!(
        denials.len(),
        1,
        "selected root source must have one exact refusal: {artifact:?}"
    );
    let ProcessRootProtocolDenialKind::Integrity(ProcessIntegrityRejection::Damaged(damage)) =
        denials[0].denial
    else {
        panic!(
            "selected root source bypassed integrity rejection: {:?}",
            denials[0]
        );
    };
    assert_eq!(damage.scope, expected);
    let (cause, offset, length, field, blast) = if operator == Op::ScopeSubstitution {
        if target.family == "root_manifest" {
            (
                ProcessDamageCause::PhysicalGenerationMismatch,
                28,
                8,
                Some(ProcessFormatField::PhysicalGeneration),
                ProcessBlastRadius::ReachableSubtree,
            )
        } else {
            (
                ProcessDamageCause::StoreIdentityMismatch,
                48,
                16,
                Some(ProcessFormatField::StoreIdentity),
                ProcessBlastRadius::CompleteArtifact,
            )
        }
    } else {
        (
            ProcessDamageCause::ChecksumMismatch,
            0,
            target.length() as u64,
            None,
            ProcessBlastRadius::CanonicalFrame,
        )
    };
    assert_eq!(damage.cause, cause);
    assert_eq!(
        damage.damaged_range,
        ProcessByteRange {
            offset: target.offset() as u64 + offset,
            length
        }
    );
    assert_eq!(damage.field, field);
    assert_eq!(damage.blast_radius, blast);
    let discovery = observed
        .discovery
        .expect("selected root refusal retains discovery counters");
    let interpreted = match target.family {
        "current_root_selector" => discovery.current_selector_interpretations,
        "previous_root_selector" => discovery.previous_selector_interpretations,
        "root_manifest" => discovery.current_root_candidate_interpretations,
        _ => unreachable!(),
    };
    assert_eq!(
        interpreted, 0,
        "poisoned source entered its C8 owner interpreter"
    );
    println!("C9 C8 root source={artifact:?} rejected before interpreter entry");
    true
}
