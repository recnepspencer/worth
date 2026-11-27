use super::super::{
    ApplicationAbilityRef, ApplicationAbilityVocabulary, ApplicationAuthorizationPath,
    ApplicationAuthorizationRule, ApplicationAuthorizationRuleDenial, ApplicationPolicyRef,
    ApplicationRoleMatrix,
};
use super::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn policy<Policy>(mut self, policy: ApplicationPolicyRef<Schema, Policy>) -> Self {
        self.members.push(ApplicationSchemaMember::Policy {
            policy: policy.name().to_string(),
        });
        self
    }

    pub fn ability<Ability, Scope>(
        mut self,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
    ) -> Self {
        self.members.push(ApplicationSchemaMember::Ability {
            ability: ability.name().to_string(),
            scope_entity: ability.scope().to_string(),
        });
        self
    }

    pub fn operation_requires_ability<Operation, Input, Ability, Scope>(
        mut self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
    ) -> Self
    where
        Ability: super::super::capabilities::OperationRequiresAbility<Operation>,
    {
        self.members
            .push(ApplicationSchemaMember::OperationAbility {
                operation: operation.name().to_string(),
                ability: ability.name().to_string(),
                scope_entity: ability.scope().to_string(),
            });
        self
    }

    pub fn ability_policy<Ability, Scope, Policy>(
        mut self,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
        policy: ApplicationPolicyRef<Schema, Policy>,
        paths: impl IntoIterator<Item = ApplicationAuthorizationPath>,
    ) -> Self {
        let mut paths = paths.into_iter().collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        self.members.push(ApplicationSchemaMember::AbilityPolicy {
            ability: ability.name().to_string(),
            scope_entity: ability.scope().to_string(),
            policy: policy.name().to_string(),
            paths,
        });
        self
    }
}

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Declares `ability` and installs `rule` as its policy paths. A rule that
    /// cannot lower is kept as the schema's refusal and reported by `build`.
    pub fn ability_rule<Ability, Scope, Policy, Principal>(
        self,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
        policy: ApplicationPolicyRef<Schema, Policy>,
        rule: impl Into<ApplicationAuthorizationRule<Schema, Principal, Scope>>,
    ) -> Self {
        self.install_ability_rule(
            ability.name(),
            ability.scope(),
            policy.name(),
            rule.into().into_paths(),
        )
    }

    /// Installs every ability of `Ability` from the role matrix: one rule per
    /// ability, built by `rule` over exactly the roles granting it. An ability
    /// no role grants is refused by name rather than installed for nobody.
    pub fn role_abilities<Role, Ability, Policy, Principal>(
        mut self,
        policy: ApplicationPolicyRef<Schema, Policy>,
        rule: impl Fn(
            Ability,
            &[Role],
        ) -> ApplicationAuthorizationRule<Schema, Principal, Ability::Scope>,
    ) -> Self
    where
        Role: ApplicationRoleMatrix<Ability>,
        Ability: ApplicationAbilityVocabulary<Schema>,
    {
        for ability in Ability::ALL.iter().copied() {
            let roles = Role::VARIANTS
                .iter()
                .copied()
                .filter(|role| role.abilities().contains(&ability))
                .collect::<Vec<_>>();
            let paths = match roles.is_empty() {
                true => Err(ApplicationAuthorizationRuleDenial::RoleLessAbility {
                    ability: ability.ability_name().to_owned(),
                }),
                false => rule(ability, &roles).into_paths(),
            };
            self = self.install_ability_rule(
                ability.ability_name(),
                Ability::scope_name(),
                policy.name(),
                paths,
            );
        }
        self
    }

    fn install_ability_rule(
        mut self,
        ability: &str,
        scope_entity: &str,
        policy: &str,
        paths: Result<Vec<ApplicationAuthorizationPath>, ApplicationAuthorizationRuleDenial>,
    ) -> Self {
        match paths {
            Ok(paths) => {
                self.members.push(ApplicationSchemaMember::Ability {
                    ability: ability.to_owned(),
                    scope_entity: scope_entity.to_owned(),
                });
                self.members.push(ApplicationSchemaMember::AbilityPolicy {
                    ability: ability.to_owned(),
                    scope_entity: scope_entity.to_owned(),
                    policy: policy.to_owned(),
                    paths,
                });
            }
            Err(denial) => {
                self.refusal.get_or_insert(denial.into());
            }
        }
        self
    }
}
