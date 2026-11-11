use super::*;

#[test]
fn runtime_surface_evidence_identities_resist_joined_string_folklore_collisions() {
    let authority = crate::runtime::WorthQueryRuntimeEvidenceAuthority::new();
    let left = crate::runtime::WorthQueryPreviewBasisAdmission::new(
        &authority,
        test_session_label("preview|basis"),
        WorthQueryEffectPolicy::SandboxedWriteIntent,
        crate::runtime::WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
            "alpha",
            "beta|gamma",
        ])
        .expect("fixture evidence rows must fit"),
    )
    .expect("fixture digest must fit");
    let right = crate::runtime::WorthQueryPreviewBasisAdmission::new(
        &authority,
        test_session_label("preview"),
        WorthQueryEffectPolicy::SandboxedWriteIntent,
        crate::runtime::WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
            "basis|alpha",
            "beta|gamma",
        ])
        .expect("fixture evidence rows must fit"),
    )
    .expect("fixture digest must fit");
    let branch = crate::runtime::WorthQueryBranchBasisAdmission::new(
        &authority,
        test_session_label("preview|basis"),
        WorthQueryEffectPolicy::SandboxedWriteIntent,
        crate::runtime::WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
            "alpha",
            "beta|gamma",
        ])
        .expect("fixture evidence rows must fit"),
    )
    .expect("fixture digest must fit");

    assert_ne!(left.admission_identity(), right.admission_identity());
    assert_ne!(left.admission_identity(), branch.admission_identity());
}
