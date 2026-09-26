use worth_ui_dsl::{UiDslSemanticArtifact, UiDslSemanticFamily};

use crate::declaration::family::contracts::{
    UiControlDeclarationFamily, UiDeclarationIntentProjectionRole,
    UiDeclarationQueryBindingProjectionRole, UiDiagnosticSurfaceDeclarationFamily,
    UiIntentDeclarationFamily, UiLocalCompositionDeclarationFamily, UiMosaicDeclarationFamily,
    UiPageDeclarationFamily, UiPageSetDeclarationFamily, UiQueryBindingDeclarationFamily,
    UiRegionDeclarationFamily,
};
use crate::declaration::family::{
    UiDeclarationFamily, UiDeclarationFamilyAdmissionDenial, UiDeclarationFamilyKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiDeclarationFamilyAdmission {
    Admitted(UiDeclarationFamily),
    Denied(UiDeclarationFamilyAdmissionDenial),
}

impl UiDeclarationFamilyAdmission {
    pub const fn admitted_family(
        &self,
    ) -> Result<&UiDeclarationFamily, &UiDeclarationFamilyAdmissionDenial> {
        match self {
            Self::Admitted(family) => Ok(family),
            Self::Denied(denial) => Err(denial),
        }
    }
}

pub(crate) fn admit_declaration_family(
    semantic_artifact: &UiDslSemanticArtifact,
) -> UiDeclarationFamilyAdmission {
    let structural_tokens = semantic_artifact
        .structural_tokens()
        .iter()
        .map(|token| token.as_str())
        .collect::<Vec<_>>();
    let posture_tokens = semantic_artifact
        .posture_tokens()
        .iter()
        .map(|token| token.as_str())
        .collect::<Vec<_>>();

    match semantic_artifact.family() {
        UiDslSemanticFamily::Page => admit_structural_family(
            UiDeclarationFamilyKind::Page,
            "page:",
            &structural_tokens,
            &posture_tokens,
            |query_role, intent_role| {
                UiDeclarationFamily::Page(UiPageDeclarationFamily::new(query_role, intent_role))
            },
        ),
        UiDslSemanticFamily::PageSet => admit_structural_family(
            UiDeclarationFamilyKind::PageSet,
            "page-set:",
            &structural_tokens,
            &posture_tokens,
            |query_role, intent_role| {
                UiDeclarationFamily::PageSet(UiPageSetDeclarationFamily::new(
                    query_role,
                    intent_role,
                ))
            },
        ),
        UiDslSemanticFamily::Region => admit_structural_family(
            UiDeclarationFamilyKind::Region,
            "region:",
            &structural_tokens,
            &posture_tokens,
            |query_role, intent_role| {
                UiDeclarationFamily::Region(UiRegionDeclarationFamily::new(query_role, intent_role))
            },
        ),
        UiDslSemanticFamily::Mosaic => admit_structural_family(
            UiDeclarationFamilyKind::Mosaic,
            "mosaic:",
            &structural_tokens,
            &posture_tokens,
            |query_role, intent_role| {
                UiDeclarationFamily::Mosaic(UiMosaicDeclarationFamily::new(query_role, intent_role))
            },
        ),
        UiDslSemanticFamily::LocalComposition => admit_structural_family(
            UiDeclarationFamilyKind::LocalComposition,
            "local-composition:",
            &structural_tokens,
            &posture_tokens,
            |query_role, intent_role| {
                UiDeclarationFamily::LocalComposition(UiLocalCompositionDeclarationFamily::new(
                    query_role,
                    intent_role,
                ))
            },
        ),
        UiDslSemanticFamily::Control => admit_structural_family(
            UiDeclarationFamilyKind::Control,
            "control:",
            &structural_tokens,
            &posture_tokens,
            |query_role, intent_role| {
                UiDeclarationFamily::Control(UiControlDeclarationFamily::new(
                    query_role,
                    intent_role,
                ))
            },
        ),
        UiDslSemanticFamily::QueryBinding => admit_standalone_family(
            UiDeclarationFamilyKind::QueryBinding,
            "query-binding:standalone",
            "query-binding:",
            &structural_tokens,
            &posture_tokens,
            UiDeclarationFamily::QueryBinding(UiQueryBindingDeclarationFamily::new()),
        ),
        UiDslSemanticFamily::Intent => admit_standalone_family(
            UiDeclarationFamilyKind::Intent,
            "intent:standalone",
            "intent:",
            &structural_tokens,
            &posture_tokens,
            UiDeclarationFamily::Intent(UiIntentDeclarationFamily::new()),
        ),
        UiDslSemanticFamily::DiagnosticSurface => {
            let known_structural_claims = known_structural_family_claims(&structural_tokens);
            let matching =
                matching_structural_family_claims(&known_structural_claims, "diagnostic-surface:");
            if matching.is_empty() && known_structural_claims.is_empty() {
                return UiDeclarationFamilyAdmission::Denied(
                    UiDeclarationFamilyAdmissionDenial::MissingStructuralClaim {
                        family: UiDeclarationFamilyKind::DiagnosticSurface,
                        expected_prefix: "diagnostic-surface:",
                    },
                );
            }
            if known_structural_claims.len() != 1 || matching.len() != 1 {
                return UiDeclarationFamilyAdmission::Denied(
                    UiDeclarationFamilyAdmissionDenial::ContradictoryStructuralClaims {
                        family: UiDeclarationFamilyKind::DiagnosticSurface,
                        observed: known_structural_claims
                            .iter()
                            .map(|token| (*token).to_owned())
                            .collect(),
                    },
                );
            }
            if has_standalone_posture_tokens(&posture_tokens) {
                return UiDeclarationFamilyAdmission::Denied(
                    UiDeclarationFamilyAdmissionDenial::StructuralFamilyCannotClaimStandaloneRole {
                        family: UiDeclarationFamilyKind::DiagnosticSurface,
                        observed: posture_tokens
                            .iter()
                            .map(|token| (*token).to_owned())
                            .collect(),
                    },
                );
            }
            UiDeclarationFamilyAdmission::Admitted(UiDeclarationFamily::DiagnosticSurface(
                UiDiagnosticSurfaceDeclarationFamily::new(),
            ))
        }
        UiDslSemanticFamily::RuntimeService => {
            unreachable!("sealed runtime-service declarations use service handoff admission")
        }
    }
}

fn admit_structural_family<F>(
    family: UiDeclarationFamilyKind,
    expected_prefix: &'static str,
    structural_tokens: &[&str],
    posture_tokens: &[&str],
    build: F,
) -> UiDeclarationFamilyAdmission
where
    F: FnOnce(
        UiDeclarationQueryBindingProjectionRole,
        UiDeclarationIntentProjectionRole,
    ) -> UiDeclarationFamily,
{
    let known_structural_claims = known_structural_family_claims(structural_tokens);
    let matching = matching_structural_family_claims(&known_structural_claims, expected_prefix);

    if matching.is_empty() && known_structural_claims.is_empty() {
        return UiDeclarationFamilyAdmission::Denied(
            UiDeclarationFamilyAdmissionDenial::MissingStructuralClaim {
                family,
                expected_prefix,
            },
        );
    }

    if known_structural_claims.len() != 1 || matching.len() != 1 {
        return UiDeclarationFamilyAdmission::Denied(
            UiDeclarationFamilyAdmissionDenial::ContradictoryStructuralClaims {
                family,
                observed: known_structural_claims
                    .iter()
                    .map(|token| (*token).to_owned())
                    .collect(),
            },
        );
    }

    let query_role = admit_attached_projection_role(
        posture_tokens,
        family,
        "query-binding:",
        "query-binding:standalone",
        "query-binding:attached:",
        UiDeclarationQueryBindingProjectionRole::Absent,
        UiDeclarationQueryBindingProjectionRole::Attached,
    );
    let intent_role = admit_attached_projection_role(
        posture_tokens,
        family,
        "intent:",
        "intent:standalone",
        "intent:attached:",
        UiDeclarationIntentProjectionRole::Absent,
        UiDeclarationIntentProjectionRole::Attached,
    );

    match (query_role, intent_role) {
        (Err(denial), _) | (_, Err(denial)) => UiDeclarationFamilyAdmission::Denied(denial),
        (Ok(query_role), Ok(intent_role)) => {
            UiDeclarationFamilyAdmission::Admitted(build(query_role, intent_role))
        }
    }
}

fn admit_standalone_family(
    family: UiDeclarationFamilyKind,
    standalone_token: &'static str,
    token_prefix: &'static str,
    structural_tokens: &[&str],
    posture_tokens: &[&str],
    admitted_family: UiDeclarationFamily,
) -> UiDeclarationFamilyAdmission {
    if !structural_tokens.is_empty() {
        return UiDeclarationFamilyAdmission::Denied(
            UiDeclarationFamilyAdmissionDenial::StandaloneFamilyCannotCarryStructuralClaims {
                family,
                observed: structural_tokens
                    .iter()
                    .map(|token| (*token).to_owned())
                    .collect(),
            },
        );
    }

    let family_tokens = posture_tokens
        .iter()
        .copied()
        .filter(|token| token.starts_with(token_prefix))
        .collect::<Vec<_>>();
    let standalone_count = family_tokens
        .iter()
        .filter(|token| **token == standalone_token)
        .count();

    if standalone_count != 1 {
        return UiDeclarationFamilyAdmission::Denied(
            UiDeclarationFamilyAdmissionDenial::StandaloneFamilyRequiresStandalonePosture {
                family,
                expected_token: standalone_token,
            },
        );
    }

    if family_tokens.len() != 1 || has_foreign_role_tokens(posture_tokens, family) {
        return UiDeclarationFamilyAdmission::Denied(
            UiDeclarationFamilyAdmissionDenial::StandaloneFamilyCannotProjectAttachedRole {
                family,
                observed: posture_tokens
                    .iter()
                    .map(|token| (*token).to_owned())
                    .collect(),
            },
        );
    }

    UiDeclarationFamilyAdmission::Admitted(admitted_family)
}

fn admit_attached_projection_role<T>(
    posture_tokens: &[&str],
    family: UiDeclarationFamilyKind,
    prefix: &'static str,
    standalone_token: &'static str,
    attached_prefix: &'static str,
    absent: T,
    attached: T,
) -> Result<T, UiDeclarationFamilyAdmissionDenial>
where
    T: Copy,
{
    let matching = posture_tokens
        .iter()
        .copied()
        .filter(|token| token.starts_with(prefix))
        .collect::<Vec<_>>();

    if matching.is_empty() {
        return Ok(absent);
    }

    if matching.contains(&standalone_token) {
        return Err(
            UiDeclarationFamilyAdmissionDenial::StructuralFamilyCannotClaimStandaloneRole {
                family,
                observed: matching.iter().map(|token| (*token).to_owned()).collect(),
            },
        );
    }

    if matching.len() > 1 {
        return Err(
            UiDeclarationFamilyAdmissionDenial::ContradictoryAttachedRoleClaims {
                family,
                observed: matching.iter().map(|token| (*token).to_owned()).collect(),
            },
        );
    }

    let attached_claim = matching[0];
    if !attached_claim.starts_with(attached_prefix) || attached_claim.len() == attached_prefix.len()
    {
        return Err(
            UiDeclarationFamilyAdmissionDenial::InvalidAttachedRoleClaim {
                family,
                expected_prefix: attached_prefix,
                observed: matching.iter().map(|token| (*token).to_owned()).collect(),
            },
        );
    }

    Ok(attached)
}

fn has_standalone_posture_tokens(posture_tokens: &[&str]) -> bool {
    posture_tokens
        .iter()
        .any(|token| *token == "query-binding:standalone" || *token == "intent:standalone")
}

fn known_structural_family_claims<'a>(structural_tokens: &'a [&str]) -> Vec<&'a str> {
    structural_tokens
        .iter()
        .copied()
        .filter(|token| known_structural_family_prefix(token).is_some())
        .collect()
}

fn matching_structural_family_claims<'a>(
    structural_claims: &'a [&str],
    expected_prefix: &'static str,
) -> Vec<&'a str> {
    structural_claims
        .iter()
        .copied()
        .filter(|token| token.starts_with(expected_prefix))
        .collect()
}

fn known_structural_family_prefix(token: &str) -> Option<&'static str> {
    [
        "page:",
        "page-set:",
        "region:",
        "mosaic:",
        "local-composition:",
        "control:",
        "diagnostic-surface:",
    ]
    .into_iter()
    .find(|prefix| token.starts_with(prefix))
}

fn has_foreign_role_tokens(posture_tokens: &[&str], family: UiDeclarationFamilyKind) -> bool {
    match family {
        UiDeclarationFamilyKind::QueryBinding => posture_tokens
            .iter()
            .any(|token| token.starts_with("intent:")),
        UiDeclarationFamilyKind::Intent => posture_tokens
            .iter()
            .any(|token| token.starts_with("query-binding:")),
        UiDeclarationFamilyKind::Page
        | UiDeclarationFamilyKind::PageSet
        | UiDeclarationFamilyKind::Region
        | UiDeclarationFamilyKind::Mosaic
        | UiDeclarationFamilyKind::LocalComposition
        | UiDeclarationFamilyKind::Control
        | UiDeclarationFamilyKind::DiagnosticSurface => false,
    }
}
