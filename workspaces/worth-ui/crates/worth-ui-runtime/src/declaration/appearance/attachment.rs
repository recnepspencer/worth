#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceRoleAttachmentDenial {
    MissingComponentReference,
    UnknownRole,
    StaleRoleRevision,
    RoleTargetMismatch,
    AspectContractMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceRoleAttachment {
    target: crate::capability::ComponentId,
    role: worth_ui_dsl::UiAppearanceRoleIdentity,
    revision: worth_ui_dsl::UiAppearanceRoleRevision,
    aspect_contract: worth_ui_dsl::UiAppearanceAspectContract,
}

impl UiAppearanceRoleAttachment {
    pub(crate) fn admit(
        declaration: &worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration,
        target: Option<&crate::capability::ComponentId>,
        snapshot: &crate::capability::CapabilitySnapshot,
    ) -> Result<Self, UiAppearanceRoleAttachmentDenial> {
        let target = target
            .ok_or(UiAppearanceRoleAttachmentDenial::MissingComponentReference)?
            .clone();
        let component = snapshot
            .components()
            .get(&target)
            .expect("component reference was admitted against this frozen snapshot");
        let role = snapshot
            .appearance_roles()
            .get(declaration.role())
            .ok_or(UiAppearanceRoleAttachmentDenial::UnknownRole)?;
        if role.revision() != declaration.revision() {
            return Err(UiAppearanceRoleAttachmentDenial::StaleRoleRevision);
        }
        admit_role_target(role.applicability(), &target)?;
        let target_contract = component
            .appearance_aspect_contract()
            .ok_or(UiAppearanceRoleAttachmentDenial::AspectContractMismatch)?;
        if target_contract != role.aspect_contract() {
            return Err(UiAppearanceRoleAttachmentDenial::AspectContractMismatch);
        }
        Ok(Self {
            target,
            role: role.role().clone(),
            revision: role.revision(),
            aspect_contract: target_contract.clone(),
        })
    }

    pub(crate) const fn target(&self) -> &crate::capability::ComponentId {
        &self.target
    }

    pub(crate) const fn role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.role
    }

    pub(crate) const fn revision(&self) -> worth_ui_dsl::UiAppearanceRoleRevision {
        self.revision
    }

    pub(crate) const fn aspect_contract(&self) -> &worth_ui_dsl::UiAppearanceAspectContract {
        &self.aspect_contract
    }
}

fn admit_role_target(
    applicability: &worth_ui_dsl::UiAppearanceRoleApplicability,
    target: &crate::capability::ComponentId,
) -> Result<(), UiAppearanceRoleAttachmentDenial> {
    match applicability {
        worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent => Ok(()),
        worth_ui_dsl::UiAppearanceRoleApplicability::Component(applies_to)
            if applies_to.as_str() == target.as_str() =>
        {
            Ok(())
        }
        _ => Err(UiAppearanceRoleAttachmentDenial::RoleTargetMismatch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconstrained_roles_reuse_and_exact_constraints_deny_other_components() {
        let first = crate::capability::ComponentId::new("component.first").unwrap();
        let second = crate::capability::ComponentId::new("component.second").unwrap();
        assert!(admit_role_target(
            &worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
            &first,
        )
        .is_ok());
        assert!(admit_role_target(
            &worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
            &second,
        )
        .is_ok());

        let constrained = worth_ui_dsl::UiAppearanceRoleApplicability::Component(
            worth_ui_dsl::UiDslComponentReference::new("component.first").unwrap(),
        );
        assert!(admit_role_target(&constrained, &first).is_ok());
        assert_eq!(
            admit_role_target(&constrained, &second),
            Err(UiAppearanceRoleAttachmentDenial::RoleTargetMismatch)
        );
        assert_eq!(
            admit_role_target(
                &worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop,
                &first,
            ),
            Err(UiAppearanceRoleAttachmentDenial::RoleTargetMismatch)
        );
    }
}
