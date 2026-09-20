use crate::capability::CapabilitySnapshot;
use crate::source::{
    WorthUiArtifactInputResolver, WorthUiBindingSemanticsLowerer,
    WorthUiCanonicalArtifactAssembler, WorthUiIdentitySeedLowerer,
    WorthUiStructuralLegalityLowerer,
};
use worth_ui_dsl::WorthUiSealedSemanticPackage;

use super::{
    prepare_declaration_material, service_declaration_admission::admit_service_declarations,
    WorthUiPreparedSemanticHandoffMaterial, WorthUiSemanticHandoffEvidence,
    WorthUiSemanticHandoffPreparationDenial, WorthUiSemanticHandoffPreparationStop,
};

pub(in crate::runtime::source_ingress) fn prepare_semantic_handoff(
    package: WorthUiSealedSemanticPackage,
    snapshot: &CapabilitySnapshot,
) -> Result<WorthUiPreparedSemanticHandoffMaterial, WorthUiSemanticHandoffPreparationDenial> {
    let mut evidence = WorthUiSemanticHandoffEvidence::from_package(&package, snapshot);
    if !package.protocol().is_current() {
        return Err(denial(
            evidence,
            WorthUiSemanticHandoffPreparationStop::UnsupportedProtocol,
        ));
    }
    let refrozen_roles = snapshot
        .refreeze_authored_appearance_roles(
            package
                .appearance_role_declarations()
                .map(|declaration| declaration.role()),
        )
        .map_err(|cause| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::AppearanceRoleRegistration(cause),
            )
        })?;
    let scroll_region_clauses = evidence.authored_scroll_region_clauses().map_err(|cause| {
        denial(
            evidence.clone(),
            WorthUiSemanticHandoffPreparationStop::AuthoredScrollRegion(cause),
        )
    })?;
    let refrozen_regions = refrozen_roles
        .as_ref()
        .unwrap_or(snapshot)
        .refreeze_authored_scroll_regions(&scroll_region_clauses)
        .map_err(|cause| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::AuthoredScrollRegion(cause),
            )
        })?;
    let successor_snapshot = refrozen_regions.or(refrozen_roles).map(std::rc::Rc::new);
    evidence.successor_snapshot = successor_snapshot.clone();
    let snapshot = successor_snapshot.as_deref().unwrap_or(snapshot);
    let intent_material =
        crate::declaration::prepare_authored_intent_material(&package).map_err(|_| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::IntentDeclaration,
            )
        })?;
    evidence.admit_intent_material(intent_material);
    admit_service_declarations(&evidence, snapshot)
        .map_err(|stop| denial(evidence.clone(), stop))?;
    let resolved = WorthUiArtifactInputResolver::resolve(&package, snapshot).map_err(|_| {
        denial(
            evidence.clone(),
            WorthUiSemanticHandoffPreparationStop::CapabilityResolution,
        )
    })?;
    let structured =
        WorthUiStructuralLegalityLowerer::lower(&resolved, snapshot).map_err(|_| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::RuntimeStructuralAdmission,
            )
        })?;
    let mut declaration_material =
        prepare_declaration_material(&package, &structured).map_err(|_| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::DeclarationProjection,
            )
        })?;
    declaration_material
        .admit_authored_component_references(snapshot)
        .map_err(|(declaration_index, cause)| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::ComponentReference {
                    declaration_index,
                    cause,
                },
            )
        })?;
    declaration_material
        .admit_authored_appearance_attachments(snapshot)
        .map_err(|(declaration_index, cause)| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::AppearanceRoleAttachment {
                    declaration_index,
                    cause,
                },
            )
        })?;
    let bound = WorthUiBindingSemanticsLowerer::lower(&structured, snapshot).map_err(|_| {
        denial(
            evidence.clone(),
            WorthUiSemanticHandoffPreparationStop::BindingAdmission,
        )
    })?;
    let identity_seeded = WorthUiIdentitySeedLowerer::lower(&bound)
        .map_err(|_| {
            denial(
                evidence.clone(),
                WorthUiSemanticHandoffPreparationStop::IdentitySeeding,
            )
        })?
        .0;
    let artifact = WorthUiCanonicalArtifactAssembler::assemble(&identity_seeded).map_err(|_| {
        denial(
            evidence.clone(),
            WorthUiSemanticHandoffPreparationStop::CanonicalAssembly,
        )
    })?;
    Ok(WorthUiPreparedSemanticHandoffMaterial::new(
        artifact,
        declaration_material,
        evidence,
    ))
}

fn denial(
    evidence: WorthUiSemanticHandoffEvidence,
    stop: WorthUiSemanticHandoffPreparationStop,
) -> WorthUiSemanticHandoffPreparationDenial {
    WorthUiSemanticHandoffPreparationDenial::new(evidence, stop)
}
