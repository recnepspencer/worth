#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenAppearanceRoleCapabilities {
    roles: Vec<worth_ui_dsl::UiAppearanceRoleDeclaration>,
}

impl FrozenAppearanceRoleCapabilities {
    pub(crate) fn prepare_authored_succession<'a>(
        &self,
        declarations: impl IntoIterator<Item = &'a worth_ui_dsl::UiAppearanceRoleDeclaration>,
    ) -> Result<Option<Self>, super::AppearanceRoleRegistrationDenial> {
        let mut successor = None;
        for declaration in declarations {
            let index = self
                .roles
                .binary_search_by(|role| role.role().cmp(declaration.role()))
                .map_err(|_| super::AppearanceRoleRegistrationDenial::UnregisteredIdentity)?;
            if self.roles[index] != *declaration {
                let roles = successor.get_or_insert_with(|| self.roles.clone());
                roles[index] = declaration.clone();
            }
        }
        Ok(successor.map(|roles| Self { roles }))
    }

    #[cfg(test)]
    pub(crate) const fn empty() -> Self {
        Self { roles: Vec::new() }
    }

    pub(crate) fn from_accepted(
        mut roles: Vec<worth_ui_dsl::UiAppearanceRoleDeclaration>,
        accepted: &super::AppearanceRoleAcceptedRegistrationProof,
    ) -> Self {
        roles.retain(|role| accepted.admits(role));
        roles.sort_by(|left, right| left.role().cmp(right.role()));
        Self { roles }
    }

    pub fn len(&self) -> usize {
        self.roles.len()
    }

    pub fn get(
        &self,
        identity: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> Option<&worth_ui_dsl::UiAppearanceRoleDeclaration> {
        self.roles
            .binary_search_by(|role| role.role().cmp(identity))
            .ok()
            .map(|index| &self.roles[index])
    }

    pub(crate) fn digest_basis(&self) -> u64 {
        self.roles
            .iter()
            .fold(0x6170_7065_6172_616e, |digest, role| {
                let bytes = role.canonical_bytes();
                let digest =
                    super::semantic_digest::fold_semantic_digest(digest, bytes.len() as u64);
                bytes.into_iter().fold(digest, |digest, byte| {
                    super::semantic_digest::fold_semantic_digest(digest, u64::from(byte))
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn role(
        applicability: worth_ui_dsl::UiAppearanceRoleApplicability,
    ) -> worth_ui_dsl::UiAppearanceRoleDeclaration {
        let contract = worth_ui_dsl::UiAppearanceAspectContract::component(
            [worth_ui_dsl::UiAppearanceAspect::Background],
            [],
        )
        .unwrap();
        let partition = worth_ui_dsl::UiAppearancePartitionAuthoring::new([])
            .with_cell(worth_ui_dsl::UiAppearanceCell::when([]).uses_slot(
                worth_ui_dsl::UiThemeSlotIdentity::new("test.slot").unwrap(),
                worth_ui_dsl::UiThemeValueKind::Color,
            ))
            .compile(worth_ui_dsl::UiAppearanceAspect::Background)
            .unwrap();
        worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
            worth_ui_dsl::UiAppearanceRoleIdentity::new("test.role").unwrap(),
            worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
            applicability,
            &contract,
            [(worth_ui_dsl::UiAppearanceAspect::Background, partition)],
        )
        .unwrap()
    }

    #[test]
    fn applicability_changes_the_frozen_semantic_digest() {
        let any = FrozenAppearanceRoleCapabilities {
            roles: vec![role(
                worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent,
            )],
        };
        let constrained = FrozenAppearanceRoleCapabilities {
            roles: vec![role(
                worth_ui_dsl::UiAppearanceRoleApplicability::Component(
                    worth_ui_dsl::UiDslComponentReference::new("test.component").unwrap(),
                ),
            )],
        };
        assert_ne!(any.digest_basis(), constrained.digest_basis());
    }

    #[test]
    fn authored_succession_preserves_registration_and_reuses_equal_meaning() {
        let original = role(worth_ui_dsl::UiAppearanceRoleApplicability::AnyComponent);
        let registered = FrozenAppearanceRoleCapabilities {
            roles: vec![original.clone()],
        };
        assert_eq!(
            registered.prepare_authored_succession([&original]),
            Ok(None)
        );
        let changed = role(worth_ui_dsl::UiAppearanceRoleApplicability::Component(
            worth_ui_dsl::UiDslComponentReference::new("test.component").unwrap(),
        ));
        let successor = registered
            .prepare_authored_succession([&changed])
            .unwrap()
            .unwrap();
        assert_eq!(registered.get(original.role()), Some(&original));
        assert_eq!(successor.get(changed.role()), Some(&changed));
        assert_ne!(registered.digest_basis(), successor.digest_basis());
        let unknown = worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
            worth_ui_dsl::UiAppearanceRoleIdentity::new("unregistered.role").unwrap(),
            original.revision(),
            original.applicability().clone(),
            original.aspect_contract(),
            original.partitions().iter().cloned(),
        )
        .unwrap();
        assert_eq!(
            registered.prepare_authored_succession([&unknown]),
            Err(super::super::AppearanceRoleRegistrationDenial::UnregisteredIdentity)
        );
        assert_eq!(registered.len(), 1);
    }
}
