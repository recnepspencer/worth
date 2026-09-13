use crate::application_schema::{ApplicationFieldBindingRecipe, ApplicationPrincipalBindingRef};
use worth_foundational::facade::ScalarAspectType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMutationPrincipalBindingContract {
    name: String,
    mapping_entity: String,
    identity_aspect: String,
    identity_field: String,
    status_aspect: String,
    status_field: String,
    target_relation: String,
    principal_entity: String,
    principal_identity_aspect: String,
    principal_identity_field: String,
    principal_identity_scalar_family: ScalarAspectType,
    principal_identity_value_type: String,
    principal_identity_binding: ApplicationFieldBindingRecipe,
}

impl ApplicationMutationPrincipalBindingContract {
    pub(crate) fn from_reference<Schema, Binding, Mapping, Principal, Identity, IdentityBinding>(
        reference: ApplicationPrincipalBindingRef<
            Schema,
            Binding,
            Mapping,
            Principal,
            Identity,
            IdentityBinding,
        >,
    ) -> Self {
        Self {
            name: reference.name().to_owned(),
            mapping_entity: reference.mapping_entity().to_owned(),
            identity_aspect: reference.identity_aspect().to_owned(),
            identity_field: reference.identity_field().to_owned(),
            status_aspect: reference.status_aspect().to_owned(),
            status_field: reference.status_field().to_owned(),
            target_relation: reference.target_relation().to_owned(),
            principal_entity: reference.principal_entity().to_owned(),
            principal_identity_aspect: reference.principal_identity_aspect().to_owned(),
            principal_identity_field: reference.principal_identity_field().to_owned(),
            principal_identity_scalar_family: reference.principal_identity_scalar_family(),
            principal_identity_value_type: reference.principal_identity_value_type().to_owned(),
            principal_identity_binding: reference.principal_identity_binding_recipe().clone(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mapping_entity(&self) -> &str {
        &self.mapping_entity
    }

    pub fn identity_field(&self) -> &str {
        &self.identity_field
    }

    pub fn identity_aspect(&self) -> &str {
        &self.identity_aspect
    }

    pub fn status_field(&self) -> &str {
        &self.status_field
    }

    pub fn status_aspect(&self) -> &str {
        &self.status_aspect
    }

    pub fn target_relation(&self) -> &str {
        &self.target_relation
    }

    pub fn principal_entity(&self) -> &str {
        &self.principal_entity
    }

    pub fn principal_identity_field(&self) -> &str {
        &self.principal_identity_field
    }

    pub fn principal_identity_aspect(&self) -> &str {
        &self.principal_identity_aspect
    }

    pub const fn principal_identity_scalar_family(&self) -> ScalarAspectType {
        self.principal_identity_scalar_family
    }

    pub fn principal_identity_value_type(&self) -> &str {
        &self.principal_identity_value_type
    }

    pub fn principal_identity_binding(&self) -> &ApplicationFieldBindingRecipe {
        &self.principal_identity_binding
    }
}
