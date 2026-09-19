use crate::declaration::{
    authored_source_provenance_digest, UiAspectContract, UiDeclarationFamily,
    UiDeclarationGraphHandoff, UiDeclarationIdentity, UiDeclarationProvenance,
    UiDeclarationStructuralDigest, UiDeclarationStructuralSemantics, UiDeclaredAspectPayload,
    UiDeclaredPostureContract, UiDeclaredPosturePayload, UiStructuralDeclarationPayload,
};

pub(crate) fn derive_declaration_graph_handoff(
    identity: &UiDeclarationIdentity,
    provenance: &UiDeclarationProvenance,
    aspect_contract: &UiAspectContract,
    family: &UiDeclarationFamily,
    structural_digest: UiDeclarationStructuralDigest,
    structural_semantics: &UiDeclarationStructuralSemantics,
    declared_posture: &UiDeclaredPostureContract,
    component_reference: Option<crate::capability::ComponentId>,
    appearance_role_attachment: Option<crate::declaration::UiAppearanceRoleAttachment>,
) -> UiDeclarationGraphHandoff {
    UiDeclarationGraphHandoff::new(
        identity.clone(),
        authored_source_provenance_digest(
            provenance.source_provenance().module_path(),
            provenance.source_provenance().declaration_index(),
        ),
        UiStructuralDeclarationPayload::new(
            family.clone(),
            structural_digest,
            structural_semantics.clone(),
        ),
        UiDeclaredAspectPayload::new(aspect_contract.clone()),
        UiDeclaredPosturePayload::new(declared_posture.clone()),
        component_reference,
        appearance_role_attachment,
    )
}
