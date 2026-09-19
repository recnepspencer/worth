impl super::UiDeclarationArtifact {
    pub(crate) fn admit_appearance_role_attachment(
        &mut self,
        snapshot: &crate::capability::CapabilitySnapshot,
    ) -> Result<(), crate::declaration::UiAppearanceRoleAttachmentDenial> {
        let Some(authored) = &self.authored_appearance_role_attachment else {
            return Ok(());
        };
        self.appearance_role_attachment =
            Some(crate::declaration::UiAppearanceRoleAttachment::admit(
                authored,
                self.component_reference.as_ref(),
                snapshot,
            )?);
        Ok(())
    }

    pub(crate) fn authored_appearance_role_attachment(
        &self,
    ) -> Option<&worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration> {
        self.authored_appearance_role_attachment.as_ref()
    }

    pub(crate) fn admit_component_reference(
        &mut self,
        snapshot: &crate::capability::CapabilitySnapshot,
    ) -> Result<(), crate::declaration::UiDeclarationComponentReferenceDenial> {
        let Some(authored) = &self.authored_component_reference else {
            return Ok(());
        };
        self.component_reference = Some(crate::declaration::admit_component_reference(
            authored, snapshot,
        )?);
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "Gate 0 retains the admitted attachment target for future lowering"
    )]
    pub(crate) const fn component_reference(&self) -> Option<&crate::capability::ComponentId> {
        self.component_reference.as_ref()
    }
}
