use super::super::{ApplicationAbilityRef, ApplicationAuthorizationPath, ApplicationPolicyRef};
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
