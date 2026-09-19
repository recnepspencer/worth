use crate::{
    UiDslAspectName, UiDslPostureToken, UiDslSemanticFamily, UiDslSemanticKey,
    UiDslStructuralToken, UiDslSupportToken,
};

/// Authored semantic meaning that must pass through the DSL compiler before it
/// can participate in a sealed package.
///
/// This value carries no provenance, seal, protocol, or runtime authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticArtifactDeclaration {
    key: UiDslSemanticKey,
    family: UiDslSemanticFamily,
    published_aspects: Vec<UiDslAspectName>,
    consumed_aspects: Vec<UiDslAspectName>,
    structural_tokens: Vec<UiDslStructuralToken>,
    posture_tokens: Vec<UiDslPostureToken>,
    support_tokens: Vec<UiDslSupportToken>,
    component_reference: Option<crate::UiDslComponentReference>,
    appearance_role_attachment: Option<crate::UiAppearanceRoleAttachmentDeclaration>,
    intent: Option<crate::WorthUiIntentDeclarationMeaning>,
    service: Option<crate::WorthUiServiceDeclarationMeaning>,
}

impl WorthUiSemanticArtifactDeclaration {
    pub fn new(key: UiDslSemanticKey, family: UiDslSemanticFamily) -> Self {
        Self {
            key,
            family,
            published_aspects: Vec::new(),
            consumed_aspects: Vec::new(),
            structural_tokens: Vec::new(),
            posture_tokens: Vec::new(),
            support_tokens: Vec::new(),
            component_reference: None,
            appearance_role_attachment: None,
            intent: None,
            service: None,
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

    pub fn key(&self) -> &UiDslSemanticKey {
        &self.key
    }

    pub fn family(&self) -> UiDslSemanticFamily {
        self.family
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

    pub fn intent_declaration(&self) -> Option<&crate::WorthUiIntentDeclarationMeaning> {
        self.intent.as_ref()
    }

    pub fn service_declaration(&self) -> Option<&crate::WorthUiServiceDeclarationMeaning> {
        self.service.as_ref()
    }

    pub(crate) fn with_service_declaration(
        mut self,
        service: crate::WorthUiServiceDeclarationMeaning,
    ) -> Self {
        self.service = Some(service);
        self
    }

    pub(crate) fn with_intent_declaration(
        mut self,
        intent: crate::WorthUiIntentDeclarationMeaning,
    ) -> Self {
        self.intent = Some(intent);
        self
    }

    pub(crate) fn canonicalize(mut self) -> Self {
        canonicalize_set(&mut self.published_aspects);
        canonicalize_set(&mut self.consumed_aspects);
        canonicalize_set(&mut self.structural_tokens);
        canonicalize_set(&mut self.posture_tokens);
        canonicalize_set(&mut self.support_tokens);
        if let Some(intent) = &mut self.intent {
            intent.canonicalize();
        }
        self
    }

    pub(crate) fn fold_source_revision(&self, digest: &mut u64) {
        fold_text(digest, self.key.as_str());
        fold_text(digest, self.family.as_str());
        fold_values(digest, &self.published_aspects);
        fold_values(digest, &self.consumed_aspects);
        fold_values(digest, &self.structural_tokens);
        fold_values(digest, &self.posture_tokens);
        fold_values(digest, &self.support_tokens);
        match &self.component_reference {
            Some(component) => {
                fold_u64(digest, 1);
                component.fold_source_revision(digest);
            }
            None => fold_u64(digest, 0),
        }
        match &self.appearance_role_attachment {
            Some(attachment) => {
                fold_u64(digest, 1);
                attachment.fold_source_revision(digest);
            }
            None => fold_u64(digest, 0),
        }
        if let Some(intent) = &self.intent {
            intent.fold_source_revision(digest);
        }
        if let Some(service) = &self.service {
            fold_text(digest, &service.canonical_text());
        }
    }
}

fn canonicalize_set<T: Ord>(values: &mut Vec<T>) {
    values.sort();
    values.dedup();
}

fn fold_values<T: SemanticText>(digest: &mut u64, values: &[T]) {
    fold_u64(digest, values.len() as u64);
    for value in values {
        fold_text(digest, value.semantic_text());
    }
}

fn fold_text(digest: &mut u64, text: &str) {
    fold_u64(digest, text.len() as u64);
    for byte in text.as_bytes() {
        *digest ^= u64::from(*byte);
        *digest = digest.wrapping_mul(0x100_0000_01b3);
    }
}

fn fold_u64(digest: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *digest ^= u64::from(byte);
        *digest = digest.wrapping_mul(0x100_0000_01b3);
    }
}

trait SemanticText {
    fn semantic_text(&self) -> &str;
}

macro_rules! semantic_text {
    ($type:ty) => {
        impl SemanticText for $type {
            fn semantic_text(&self) -> &str {
                self.as_str()
            }
        }
    };
}

semantic_text!(UiDslAspectName);
semantic_text!(UiDslStructuralToken);
semantic_text!(UiDslPostureToken);
semantic_text!(UiDslSupportToken);
