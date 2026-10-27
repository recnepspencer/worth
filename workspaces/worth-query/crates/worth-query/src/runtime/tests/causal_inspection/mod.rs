use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::inspection::CausalEvidenceReferenceDigest;
use worth_runtime_bridge::facade::{
    bridge_truth_digest_identity_evidence_from_external_token,
    bridge_truth_external_identity_token, bridge_truth_projection_identity_from_external_token,
    BridgeCausalInspectionAdmissionSummary, BridgeIdentityEvidence, TruthCommitIdentity,
};

mod admission;
mod adversarial;
mod anchor_reference;
pub(in crate::runtime::tests) mod certification;
mod dx;
mod materialization;
mod reference_index;

pub(in crate::runtime::tests) fn causal_test_reference_digest(
    reference_label: impl AsRef<str>,
) -> CausalEvidenceReferenceDigest {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
        .field_value(
            WorthQueryEvidenceTag::new("fixture_reference"),
            reference_label.as_ref(),
        )
        .seal()
        .into()
}

pub(in crate::runtime::tests::causal_inspection) fn bridge_external_evidence(
    value: impl AsRef<str>,
) -> BridgeIdentityEvidence {
    BridgeIdentityEvidence::from_external_authority(bridge_truth_external_identity_token(
        value.as_ref().to_owned(),
    ))
}

pub(in crate::runtime::tests::causal_inspection) fn causal_truth_commit_identity(
    evidence: impl AsRef<str>,
) -> TruthCommitIdentity {
    TruthCommitIdentity::from_relational_commit_id(stable_causal_position(
        "causal-truth-commit",
        evidence,
    ))
}

fn stable_causal_position(namespace: impl AsRef<str>, evidence: impl AsRef<str>) -> u64 {
    let mut acc = 14_695_981_039_346_656_037_u64;
    for byte in namespace.as_ref().bytes().chain(evidence.as_ref().bytes()) {
        acc ^= u64::from(byte);
        acc = acc.wrapping_mul(1_099_511_628_211_u64);
    }
    acc
}

pub(in crate::runtime::tests::causal_inspection) fn bridge_query_evidence(
    scope: &str,
    token: &str,
) -> BridgeIdentityEvidence {
    let external_token = bridge_truth_external_identity_token(token.to_owned());
    let projection = bridge_truth_projection_identity_from_external_token(
        external_token.clone(),
        scope.to_owned(),
    );
    let digest_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
            .field_shape(WorthQueryEvidenceTag::new("bridge_scope"), scope)
            .field_value(WorthQueryEvidenceTag::new("fixture_token"), token)
            .seal();
    let digest_evidence = bridge_truth_digest_identity_evidence_from_external_token(
        external_token,
        digest_identity.canonical_digest().clone(),
    );

    BridgeIdentityEvidence::from_query_evidence_identity(projection, digest_evidence)
}

pub(in crate::runtime::tests::causal_inspection) fn bridge_admitted_summary(
    admitted: &crate::runtime::AdmittedCausalInspection,
) -> BridgeCausalInspectionAdmissionSummary {
    BridgeCausalInspectionAdmissionSummary::admitted(
        bridge_query_evidence(
            "causal-inspection-outcome",
            admitted.admitted_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            admitted.subject().anchor_for_reporting(),
        ),
    )
    .expect("query admission summary should be valid")
}

pub(in crate::runtime::tests::causal_inspection) fn bridge_advisory_summary(
    advisory: &crate::runtime::AdvisoryCausalInspection,
) -> BridgeCausalInspectionAdmissionSummary {
    BridgeCausalInspectionAdmissionSummary::advisory(
        bridge_query_evidence(
            "causal-inspection-outcome",
            advisory.advisory_inspection_digest(),
        ),
        bridge_query_evidence(
            "causal-observation-anchor",
            advisory.subject().anchor_for_reporting(),
        ),
    )
    .expect("query advisory summary should be valid")
}

// These fixture adapters exercise only the historical diagnostic protocol.
pub(in crate::runtime::tests) fn inspect_fixture(
    receipt: crate::runtime::QueryObservationReceipt,
) -> crate::runtime::CausalInspection {
    let basis = receipt.inspection_basis_for_test();
    crate::runtime::CausalInspection::for_observation(receipt, basis)
}
