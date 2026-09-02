pub(crate) struct AppearanceRoleRegistry {
    roles: Vec<worth_ui_dsl::UiAppearanceRoleDeclaration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_enforces_role_capacity_before_mutation() {
        let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
            [worth_ui_dsl::UiAppearanceAspect::Background],
            [],
        )
        .unwrap();
        let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
            .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new("capacity.slot").unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ))
            .compile(worth_ui_dsl::UiAppearanceAspect::Background)
            .unwrap();
        let mut registry = AppearanceRoleRegistry::empty();
        for index in 0..worth_ui_dsl::UI_APPEARANCE_ROLE_CAPACITY {
            let role = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
                worth_ui_dsl::UiAppearanceRoleIdentity::new(format!("capacity.role.{index}"))
                    .unwrap(),
                worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
                worth_ui_dsl::UiAppearanceRoleApplicability::Component(
                    worth_ui_dsl::UiDslComponentReference::new("test.component").unwrap(),
                ),
                &contract,
                [(
                    worth_ui_dsl::UiAppearanceAspect::Background,
                    partition.clone(),
                )],
            )
            .unwrap();
            registry.push(role).unwrap();
        }
        let overflow = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
            worth_ui_dsl::UiAppearanceRoleIdentity::new("capacity.role.overflow").unwrap(),
            worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
            worth_ui_dsl::UiAppearanceRoleApplicability::Component(
                worth_ui_dsl::UiDslComponentReference::new("test.component").unwrap(),
            ),
            &contract,
            [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
        )
        .unwrap();
        assert_eq!(
            registry.push(overflow),
            Err(AppearanceRoleRegistrationDenial::CapacityExceeded)
        );
    }

    #[test]
    fn registry_rejects_duplicate_identity_before_mutation() {
        let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
            [worth_ui_dsl::UiAppearanceAspect::Background],
            [],
        )
        .unwrap();
        let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
            .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new("duplicate.slot").unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ))
            .compile(worth_ui_dsl::UiAppearanceAspect::Background)
            .unwrap();
        let role = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
            worth_ui_dsl::UiAppearanceRoleIdentity::new("duplicate.role").unwrap(),
            worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
            worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
            &contract,
            [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
        )
        .unwrap();
        let mut registry = AppearanceRoleRegistry::empty();
        registry.push(role.clone()).unwrap();
        assert_eq!(
            registry.push(role),
            Err(AppearanceRoleRegistrationDenial::DuplicateIdentity)
        );
        assert_eq!(registry.roles.len(), 1);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppearanceRoleRegistrationDenial {
    DuplicateIdentity,
    CapacityExceeded,
}

impl AppearanceRoleRegistry {
    pub(crate) const fn empty() -> Self {
        Self { roles: Vec::new() }
    }

    pub(crate) fn push(
        &mut self,
        role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    ) -> Result<crate::capability::RegistrationCandidate, AppearanceRoleRegistrationDenial> {
        if self
            .roles
            .iter()
            .any(|registered| registered.role() == role.role())
        {
            return Err(AppearanceRoleRegistrationDenial::DuplicateIdentity);
        }
        if self.roles.len() >= worth_ui_dsl::UI_APPEARANCE_ROLE_CAPACITY {
            return Err(AppearanceRoleRegistrationDenial::CapacityExceeded);
        }
        let candidate = super::descriptor::registration_candidate(&role);
        self.roles.push(role);
        Ok(candidate)
    }

    pub(crate) fn freeze(
        self,
        accepted: &super::AppearanceRoleAcceptedRegistrationProof,
    ) -> super::FrozenAppearanceRoleCapabilities {
        super::FrozenAppearanceRoleCapabilities::from_accepted(self.roles, accepted)
    }

    pub(crate) fn get(
        &self,
        identity: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> Option<&worth_ui_dsl::UiAppearanceRoleDeclaration> {
        self.roles.iter().find(|role| role.role() == identity)
    }
}
