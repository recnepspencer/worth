use crate::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceIdentityComparisonError,
    WorthQueryEvidenceIdentityScheme, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[test]
fn intent_receipt_identity_scopes_fail_cross_scheme_typed() {
    for scope in [
        WorthQueryEvidenceScope::AuthoritativeIntentReceipt,
        WorthQueryEvidenceScope::EffectIntentReceipt,
        WorthQueryEvidenceScope::PreviewIntentReceipt,
        WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis,
        WorthQueryEvidenceScope::PreviewIntentReceiptInspection,
    ] {
        let v1 = probe_identity(scope, WorthQueryEvidenceIdentityScheme::V1);
        let v1_same = probe_identity(scope, WorthQueryEvidenceIdentityScheme::V1);
        let v2 = probe_identity(scope, WorthQueryEvidenceIdentityScheme::V2);

        assert_eq!(v1.eq_same_scheme(&v1_same), Ok(true));
        assert_eq!(
            v1.eq_same_scheme(&v2),
            Err(WorthQueryEvidenceIdentityComparisonError::SchemeMismatch {
                left: WorthQueryEvidenceIdentityScheme::V1,
                right: WorthQueryEvidenceIdentityScheme::V2,
            }),
            "{scope:?} must reject cross-scheme comparison as a typed mismatch"
        );
    }
}

fn probe_identity(
    scope: WorthQueryEvidenceScope,
    scheme: WorthQueryEvidenceIdentityScheme,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose_with_scheme(scope, scheme)
        .field_shape(WorthQueryEvidenceTag::new("probe_shape"), "intent-receipt")
        .field_value(
            WorthQueryEvidenceTag::new("probe_identity"),
            "identity|with:delimiter",
        )
        .seal()
        .expect("fixture digest must fit")
}
