use worth_runtime_bridge::facade::{
    BridgeAggregateMutationEvidenceDigest, BridgeAuthoritativeMutationEvidenceCloseout,
    BridgeAuthoritativeMutationEvidenceSupport, BridgeAuthorityEvidenceDeferredBoundary,
    BridgeMutationEvidenceCarryForwardSection, BridgeMutationEvidenceContinuityFamily,
    BridgeMutationEvidenceExistingTruthBindingFamily, BridgeMutationEvidenceNamingFamily,
    BridgeMutationEvidenceSymbolicTargetReferenceFamily,
};

use super::WorthQueryAuthoritativeMutationEvidenceSupport;

pub(super) fn assert_bridge_support_alignment(
    _query_support: &WorthQueryAuthoritativeMutationEvidenceSupport,
    bridge_support: &BridgeAuthoritativeMutationEvidenceSupport,
    bridge_closeout: &BridgeAuthoritativeMutationEvidenceCloseout,
) {
    let mut failures = Vec::new();

    for section in [
        BridgeMutationEvidenceCarryForwardSection::DeclaredResolvedTargetEvidence,
        BridgeMutationEvidenceCarryForwardSection::BatchSessionCausalityProvenance,
        BridgeMutationEvidenceCarryForwardSection::ExistingTruthBinding,
        BridgeMutationEvidenceCarryForwardSection::SameBatchSymbolicTargetReference,
        BridgeMutationEvidenceCarryForwardSection::NamingMutationEvidence,
        BridgeMutationEvidenceCarryForwardSection::ContinuityMutationEvidence,
        BridgeMutationEvidenceCarryForwardSection::ReplaySafeRequestReceiptDigests,
    ] {
        if !bridge_support
            .carry_forward_sections()
            .iter()
            .any(|bridge_section| bridge_section == &section)
        {
            failures.push(format!("missing carry-forward section `{section:?}`"));
        }
    }

    for family in [
        BridgeMutationEvidenceExistingTruthBindingFamily::DirectEntityIdentity,
        BridgeMutationEvidenceExistingTruthBindingFamily::DirectRelationIdentity,
    ] {
        if !bridge_support
            .existing_truth_binding_families()
            .iter()
            .any(|bridge_family| bridge_family == &family)
        {
            failures.push(format!(
                "missing existing-truth binding family `{family:?}`"
            ));
        }
    }
    {
        let family = BridgeMutationEvidenceSymbolicTargetReferenceFamily::SameBatchDeclaredTarget;
        if !bridge_support
            .symbolic_target_reference_families()
            .iter()
            .any(|bridge_family| bridge_family == &family)
        {
            failures.push(format!("missing symbolic target family `{family:?}`"));
        }
    }
    for family in [
        BridgeMutationEvidenceNamingFamily::AttachNewTarget,
        BridgeMutationEvidenceNamingFamily::AttachExistingTarget,
        BridgeMutationEvidenceNamingFamily::RebindTarget,
        BridgeMutationEvidenceNamingFamily::Remove,
    ] {
        if !bridge_support
            .naming_mutation_families()
            .iter()
            .any(|bridge_family| bridge_family == &family)
        {
            failures.push(format!("missing naming family `{family:?}`"));
        }
    }
    for family in [
        BridgeMutationEvidenceContinuityFamily::RebindExistingTarget,
        BridgeMutationEvidenceContinuityFamily::SplitExistingTarget,
    ] {
        if !bridge_support
            .continuity_mutation_families()
            .iter()
            .any(|bridge_family| bridge_family == &family)
        {
            failures.push(format!("missing continuity family `{family:?}`"));
        }
    }

    for section in [
        BridgeAggregateMutationEvidenceDigest::ExistingTruthBinding,
        BridgeAggregateMutationEvidenceDigest::SymbolicTargetReference,
        BridgeAggregateMutationEvidenceDigest::NamingMutation,
        BridgeAggregateMutationEvidenceDigest::ContinuityMutation,
        BridgeAggregateMutationEvidenceDigest::Causality,
        BridgeAggregateMutationEvidenceDigest::Provenance,
    ] {
        if !bridge_support
            .aggregate_evidence_digests()
            .iter()
            .any(|bridge_section| bridge_section == &section)
        {
            failures.push(format!("missing aggregate evidence digest `{section:?}`"));
        }
    }

    if !bridge_closeout
        .deferred_boundaries()
        .iter()
        .any(|boundary| {
            boundary == &BridgeAuthorityEvidenceDeferredBoundary::UnsupportedMutationFamiliesRemainFailClosed
        })
    {
        failures.push(
            "bridge closeout does not fail-close unsupported existing-truth binding families"
                .to_string(),
        );
    }

    assert!(
        failures.is_empty(),
        "bridge/query authoritative mutation evidence drifted: {}",
        failures.join(", ")
    );
}
