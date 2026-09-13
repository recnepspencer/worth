use std::any::TypeId;

use worth_foundational::facade::ScalarAspectType;

use crate::{
    application_schema::{
        ApplicationFieldBindingRecipe, ApplicationPrincipalBindingRef,
        ApplicationStructuredValueBinding,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

use super::ApplicationQueryBindingLimits;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationQueryScopeResolutionMode {
    InputField,
    PrincipalIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationQueryScopeContract {
    mode: ApplicationQueryScopeResolutionMode,
    field: ApplicationFieldBindingRecipe,
    writable: bool,
}

impl ApplicationQueryScopeContract {
    pub(crate) fn new(
        mode: ApplicationQueryScopeResolutionMode,
        field: ApplicationFieldBindingRecipe,
        writable: bool,
    ) -> Self {
        Self {
            mode,
            field,
            writable,
        }
    }

    pub const fn mode(&self) -> ApplicationQueryScopeResolutionMode {
        self.mode
    }

    pub fn field(&self) -> &ApplicationFieldBindingRecipe {
        &self.field
    }

    pub const fn writable(&self) -> bool {
        self.writable
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationQueryPrincipalBindingContract {
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

impl ApplicationQueryPrincipalBindingContract {
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
    pub fn identity_aspect(&self) -> &str {
        &self.identity_aspect
    }
    pub fn identity_field(&self) -> &str {
        &self.identity_field
    }
    pub fn status_aspect(&self) -> &str {
        &self.status_aspect
    }
    pub fn status_field(&self) -> &str {
        &self.status_field
    }
    pub fn target_relation(&self) -> &str {
        &self.target_relation
    }
    pub fn principal_entity(&self) -> &str {
        &self.principal_entity
    }
    pub fn principal_identity_aspect(&self) -> &str {
        &self.principal_identity_aspect
    }
    pub fn principal_identity_field(&self) -> &str {
        &self.principal_identity_field
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationQueryBindingDescriptor {
    identity: String,
    input_identity: WorthQueryPortableTypeIdentity,
    input_binding_type: TypeId,
    input_type: TypeId,
    query_name: String,
    query_identity: WorthQueryPortableTypeIdentity,
    query_type: TypeId,
    parameter_identity: WorthQueryPortableTypeIdentity,
    result_identity: WorthQueryPortableTypeIdentity,
    scope_identity: WorthQueryPortableTypeIdentity,
    scope: ApplicationQueryScopeContract,
    principal: ApplicationQueryPrincipalBindingContract,
    limits: ApplicationQueryBindingLimits,
}

impl ApplicationQueryBindingDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new<InputBinding, Query>(
        identity: &'static str,
        query_name: &'static str,
        query_identity: WorthQueryPortableTypeIdentity,
        parameter_identity: WorthQueryPortableTypeIdentity,
        result_identity: WorthQueryPortableTypeIdentity,
        scope_identity: WorthQueryPortableTypeIdentity,
        scope: ApplicationQueryScopeContract,
        principal: ApplicationQueryPrincipalBindingContract,
        limits: ApplicationQueryBindingLimits,
    ) -> Self
    where
        InputBinding: ApplicationStructuredValueBinding,
        Query: 'static,
    {
        Self {
            identity: identity.to_owned(),
            input_identity: InputBinding::IDENTITY,
            input_binding_type: TypeId::of::<InputBinding>(),
            input_type: TypeId::of::<InputBinding::Value>(),
            query_name: query_name.to_owned(),
            query_identity,
            query_type: TypeId::of::<Query>(),
            parameter_identity,
            result_identity,
            scope_identity,
            scope,
            principal,
            limits,
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn input_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.input_identity
    }
    pub fn input_binding_type(&self) -> TypeId {
        self.input_binding_type
    }
    pub fn input_type(&self) -> TypeId {
        self.input_type
    }
    pub fn query_name(&self) -> &str {
        &self.query_name
    }
    pub fn query_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.query_identity
    }
    pub fn query_type(&self) -> TypeId {
        self.query_type
    }
    pub fn parameter_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parameter_identity
    }
    pub fn result_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.result_identity
    }
    pub fn scope_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.scope_identity
    }
    pub fn scope(&self) -> &ApplicationQueryScopeContract {
        &self.scope
    }
    pub fn principal(&self) -> &ApplicationQueryPrincipalBindingContract {
        &self.principal
    }
    pub const fn limits(&self) -> ApplicationQueryBindingLimits {
        self.limits
    }
}
