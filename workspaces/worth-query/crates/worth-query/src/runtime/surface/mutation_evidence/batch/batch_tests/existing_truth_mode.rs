use super::{relational_entity, retained_assertion_identity};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryExistingTruthAssertionEvidence, WorthQueryMutationFamily,
};
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};

#[test]
fn summary_digest_changes_with_mutation_family() -> Result<(), IdentityDerivationDenial> {
    let backend_verified = backend_verified_assertion()?;

    let update = super::super::summarize_existing_truth_modes(
        &[WorthQueryMutationFamily::Update],
        &[Some(backend_verified.clone())],
    );
    let delete = super::super::summarize_existing_truth_modes(
        &[WorthQueryMutationFamily::Delete],
        &[Some(backend_verified)],
    );

    assert_ne!(update.4, delete.4);

    Ok(())
}

#[test]
fn summary_digest_changes_with_assertion_mode() -> Result<(), IdentityDerivationDenial> {
    let retained = WorthQueryExistingTruthAssertionEvidence::retained_assertion(
        1,
        retained_assertion_identity("retained-assertion")?,
    );
    let backend_verified = backend_verified_assertion()?;

    let retained_summary = super::super::summarize_existing_truth_modes(
        &[WorthQueryMutationFamily::Assertion],
        &[Some(retained)],
    );
    let verified_summary = super::super::summarize_existing_truth_modes(
        &[WorthQueryMutationFamily::Assertion],
        &[Some(backend_verified)],
    );

    assert_ne!(retained_summary.4, verified_summary.4);

    Ok(())
}

#[test]
#[should_panic(expected = "invalid existing-truth assertion mode")]
fn summary_panics_on_invalid_family_mode_pair() -> Result<(), IdentityDerivationDenial> {
    let retained = WorthQueryExistingTruthAssertionEvidence::retained_assertion(
        1,
        retained_assertion_identity("retained-assertion")?,
    );

    let _ = super::super::summarize_existing_truth_modes(
        &[WorthQueryMutationFamily::Update],
        &[Some(retained)],
    );

    Ok(())
}

fn backend_verified_assertion(
) -> Result<WorthQueryExistingTruthAssertionEvidence, IdentityDerivationDenial> {
    Ok(WorthQueryExistingTruthAssertionEvidence::backend_verified(
        &crate::runtime::WorthQueryVerifiedExistingTruthAssertion::new(
            &crate::runtime::WorthQueryExistingTruthTargetBinding::direct_entity(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
                    crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new(
                        "authority:left",
                    )
                    .expect("existing-truth authority label"),
                )
                .expect("existing-truth authority identity"),
                relational_entity(1, 1, 0),
            )
            .expect("binding should build"),
            &[crate::runtime::WorthQueryAuthoredAspectMutation::new(
                title_value_touch(),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                    "Seed title",
                ),
            )
            .expect("aspect should build")],
            crate::memory_workspace::admit_external_snapshot_label("snapshot:test")?,
        )
        .expect("verified assertion should build"),
    ))
}

fn title_value_touch() -> WorthQueryAspectTouch {
    let aspect_key = AspectKey::new("title").expect("batch evidence test aspect key should admit");
    let field_key = FieldKey::new("value").expect("batch evidence test field key should admit");
    let field_path =
        CanonicalFieldPath::new([field_key]).expect("batch evidence test field path should admit");
    WorthQueryAspectTouch::aspect_field_path(aspect_key, field_path)
}
