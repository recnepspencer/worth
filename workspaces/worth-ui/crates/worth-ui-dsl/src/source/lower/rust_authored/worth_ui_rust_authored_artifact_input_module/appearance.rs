use super::WorthUiRustAuthoredDeclaration;

impl super::WorthUiRustAuthoredArtifactInputModule {
    pub fn with_appearance_role(mut self, role: crate::UiAppearanceRoleDeclaration) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::AppearanceRole(role));
        self
    }

    pub fn with_backdrop(mut self, declaration: crate::UiBackdropDeclaration) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Backdrop(declaration));
        self
    }

    pub fn with_component_appearance_role(
        mut self,
        component_name: impl Into<String>,
        attachment: crate::UiAppearanceRoleAttachmentDeclaration,
    ) -> Result<Self, crate::UiAppearanceRoleAttachmentDeclarationDenial> {
        let component_name = component_name.into();
        let matching_components = self
            .declarations
            .iter()
            .filter(|declaration| {
                matches!(
                    declaration,
                    WorthUiRustAuthoredDeclaration::Component { name_text, .. }
                        if name_text == &component_name
                )
            })
            .count();
        if matching_components > 1 {
            return Err(crate::UiAppearanceRoleAttachmentDeclarationDenial::DuplicateAttachment);
        }
        if matching_components == 1 {
            let declaration = self
                .declarations
                .iter_mut()
                .find(|declaration| {
                    matches!(
                        declaration,
                        WorthUiRustAuthoredDeclaration::Component { name_text, .. }
                            if name_text == &component_name
                    )
                })
                .expect("the single matching component must be present");
            if let WorthUiRustAuthoredDeclaration::Component {
                appearance_role_attachment,
                ..
            } = declaration
            {
                if appearance_role_attachment.is_some() {
                    return Err(
                        crate::UiAppearanceRoleAttachmentDeclarationDenial::DuplicateAttachment,
                    );
                }
                *appearance_role_attachment = Some(attachment);
            }
            return Ok(self);
        }
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: component_name,
                authored_identity: None,
                body_atoms: Vec::new(),
                appearance_role_attachment: Some(attachment),
            });
        Ok(self)
    }
}
