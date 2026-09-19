use crate::semantic::{
    UiDslAspectName, UiDslPostureToken, UiDslSemanticFamily, UiDslSemanticKey,
    UiDslSourceProvenance, UiDslStructuralToken, UiDslSupportToken,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDslSemanticArtifact {
    key: UiDslSemanticKey,
    family: UiDslSemanticFamily,
    provenance: UiDslSourceProvenance,
    published_aspects: Vec<UiDslAspectName>,
    consumed_aspects: Vec<UiDslAspectName>,
    structural_tokens: Vec<UiDslStructuralToken>,
    posture_tokens: Vec<UiDslPostureToken>,
    support_tokens: Vec<UiDslSupportToken>,
    component_reference: Option<crate::UiDslComponentReference>,
    appearance_role_attachment: Option<crate::UiAppearanceRoleAttachmentDeclaration>,
    authored_comments: Vec<String>,
    formatting_profile: Option<String>,
    parser_local_id: Option<String>,
    diagnostic_label: Option<String>,
    renderer_label: Option<String>,
}

pub(crate) struct UiDslSemanticArtifactInput {
    pub key: UiDslSemanticKey,
    pub family: UiDslSemanticFamily,
    pub provenance: UiDslSourceProvenance,
    pub published_aspects: Vec<UiDslAspectName>,
    pub consumed_aspects: Vec<UiDslAspectName>,
    pub structural_tokens: Vec<UiDslStructuralToken>,
    pub posture_tokens: Vec<UiDslPostureToken>,
    pub support_tokens: Vec<UiDslSupportToken>,
    pub component_reference: Option<crate::UiDslComponentReference>,
    pub appearance_role_attachment: Option<crate::UiAppearanceRoleAttachmentDeclaration>,
    pub authored_comments: Vec<String>,
    pub formatting_profile: Option<String>,
    pub parser_local_id: Option<String>,
    pub diagnostic_label: Option<String>,
    pub renderer_label: Option<String>,
}

impl UiDslSemanticArtifact {
    pub(crate) fn new(input: UiDslSemanticArtifactInput) -> Self {
        let UiDslSemanticArtifactInput {
            key,
            family,
            provenance,
            published_aspects,
            consumed_aspects,
            structural_tokens,
            posture_tokens,
            support_tokens,
            component_reference,
            appearance_role_attachment,
            authored_comments,
            formatting_profile,
            parser_local_id,
            diagnostic_label,
            renderer_label,
        } = input;
        Self {
            key,
            family,
            provenance,
            published_aspects,
            consumed_aspects,
            structural_tokens,
            posture_tokens,
            support_tokens,
            component_reference,
            appearance_role_attachment,
            authored_comments,
            formatting_profile,
            parser_local_id,
            diagnostic_label,
            renderer_label,
        }
    }

    pub fn key(&self) -> &UiDslSemanticKey {
        &self.key
    }

    pub fn family(&self) -> UiDslSemanticFamily {
        self.family
    }

    pub fn provenance(&self) -> &UiDslSourceProvenance {
        &self.provenance
    }

    pub fn published_aspects(&self) -> &[UiDslAspectName] {
        &self.published_aspects
    }

    pub fn consumed_aspects(&self) -> &[UiDslAspectName] {
        &self.consumed_aspects
    }

    pub fn structural_tokens(&self) -> &[UiDslStructuralToken] {
        &self.structural_tokens
    }

    pub fn posture_tokens(&self) -> &[UiDslPostureToken] {
        &self.posture_tokens
    }

    pub fn support_tokens(&self) -> &[UiDslSupportToken] {
        &self.support_tokens
    }

    pub fn component_reference(&self) -> Option<&crate::UiDslComponentReference> {
        self.component_reference.as_ref()
    }

    pub fn appearance_role_attachment(
        &self,
    ) -> Option<&crate::UiAppearanceRoleAttachmentDeclaration> {
        self.appearance_role_attachment.as_ref()
    }
}
