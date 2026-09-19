use crate::semantic::{
    UiDslAspectName, UiDslPostureToken, UiDslSemanticArtifact, UiDslSemanticFamily,
    UiDslSemanticKey, UiDslSourceProvenance, UiDslStructuralToken, UiDslSupportToken,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDslSemanticArtifactSpec {
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

impl UiDslSemanticArtifactSpec {
    pub fn new(
        key: UiDslSemanticKey,
        family: UiDslSemanticFamily,
        provenance: UiDslSourceProvenance,
    ) -> Self {
        Self {
            key,
            family,
            provenance,
            published_aspects: Vec::new(),
            consumed_aspects: Vec::new(),
            structural_tokens: Vec::new(),
            posture_tokens: Vec::new(),
            support_tokens: Vec::new(),
            component_reference: None,
            appearance_role_attachment: None,
            authored_comments: Vec::new(),
            formatting_profile: None,
            parser_local_id: None,
            diagnostic_label: None,
            renderer_label: None,
        }
    }

    pub fn with_published_aspect(mut self, aspect: UiDslAspectName) -> Self {
        self.published_aspects.push(aspect);
        self
    }

    pub fn with_consumed_aspect(mut self, aspect: UiDslAspectName) -> Self {
        self.consumed_aspects.push(aspect);
        self
    }

    pub fn with_structural_token(mut self, token: UiDslStructuralToken) -> Self {
        self.structural_tokens.push(token);
        self
    }

    pub fn with_posture_token(mut self, token: UiDslPostureToken) -> Self {
        self.posture_tokens.push(token);
        self
    }

    pub fn with_support_token(mut self, token: UiDslSupportToken) -> Self {
        self.support_tokens.push(token);
        self
    }

    pub fn with_component_reference(
        mut self,
        component: crate::UiDslComponentReference,
    ) -> Result<Self, crate::UiDslComponentReferenceDenial> {
        if self.component_reference.is_some() {
            return Err(crate::UiDslComponentReferenceDenial::DuplicateReference);
        }
        self.component_reference = Some(component);
        Ok(self)
    }

    pub fn with_appearance_role_attachment(
        mut self,
        attachment: crate::UiAppearanceRoleAttachmentDeclaration,
    ) -> Result<Self, crate::UiAppearanceRoleAttachmentDeclarationDenial> {
        if self.appearance_role_attachment.is_some() {
            return Err(crate::UiAppearanceRoleAttachmentDeclarationDenial::DuplicateAttachment);
        }
        self.appearance_role_attachment = Some(attachment);
        Ok(self)
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.authored_comments.push(comment.into());
        self
    }

    pub fn with_formatting_profile(mut self, formatting_profile: impl Into<String>) -> Self {
        self.formatting_profile = Some(formatting_profile.into());
        self
    }

    pub fn with_parser_local_id(mut self, parser_local_id: impl Into<String>) -> Self {
        self.parser_local_id = Some(parser_local_id.into());
        self
    }

    pub fn with_diagnostic_label(mut self, diagnostic_label: impl Into<String>) -> Self {
        self.diagnostic_label = Some(diagnostic_label.into());
        self
    }

    pub fn with_renderer_label(mut self, renderer_label: impl Into<String>) -> Self {
        self.renderer_label = Some(renderer_label.into());
        self
    }

    pub(crate) fn into_artifact(self) -> UiDslSemanticArtifact {
        UiDslSemanticArtifact::new(super::UiDslSemanticArtifactInput {
            key: self.key,
            family: self.family,
            provenance: self.provenance,
            published_aspects: self.published_aspects,
            consumed_aspects: self.consumed_aspects,
            structural_tokens: self.structural_tokens,
            posture_tokens: self.posture_tokens,
            support_tokens: self.support_tokens,
            component_reference: self.component_reference,
            appearance_role_attachment: self.appearance_role_attachment,
            authored_comments: self.authored_comments,
            formatting_profile: self.formatting_profile,
            parser_local_id: self.parser_local_id,
            diagnostic_label: self.diagnostic_label,
            renderer_label: self.renderer_label,
        })
    }

    /// Materializes semantic meaning without minting compiler or runtime
    /// authority. Governed consumers still require a compiler lowering receipt.
    pub fn into_semantic_artifact(self) -> UiDslSemanticArtifact {
        self.into_artifact()
    }
}
