use std::any::TypeId;

use super::{
    ApplicationMutationDescription, ApplicationMutationDescriptionParts,
    ApplicationMutationOutputRoleDescription, ApplicationMutationPrincipalBindingContract,
    ApplicationMutationScopeContract, ApplicationMutationScopeDescription,
};

use crate::{
    application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationOutputContract,
        ApplicationMutationOutputRoleDescriptor,
    },
    application_schema::ApplicationStructuredValueBinding,
    portable_identity::WorthQueryPortableTypeIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMutationHandlerMetadata {
    identity: String,
    decision_type: TypeId,
    denial_type: TypeId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMutationIdempotencyMetadata {
    identity: String,
    key_type: TypeId,
}

impl ApplicationMutationIdempotencyMetadata {
    fn of<Key: 'static>(identity: &'static str) -> Self {
        Self {
            identity: identity.to_owned(),
            key_type: TypeId::of::<Key>(),
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn key_type(&self) -> TypeId {
        self.key_type
    }
}

impl ApplicationMutationHandlerMetadata {
    fn of<Decision: 'static, Denial: 'static>(identity: &'static str) -> Self {
        Self {
            identity: identity.to_owned(),
            decision_type: TypeId::of::<Decision>(),
            denial_type: TypeId::of::<Denial>(),
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn decision_type(&self) -> TypeId {
        self.decision_type
    }

    pub const fn denial_type(&self) -> TypeId {
        self.denial_type
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMutationBindingDescriptor {
    description: ApplicationMutationDescription,
    denial_binding_type: TypeId,
    identity: String,
    input_identity: WorthQueryPortableTypeIdentity,
    input_binding_type: TypeId,
    input_type: TypeId,
    result_identity: WorthQueryPortableTypeIdentity,
    result_binding_type: TypeId,
    result_type: TypeId,
    operation_name: String,
    operation_type: TypeId,
    handler: ApplicationMutationHandlerMetadata,
    output_contract_type: TypeId,
    output_roles: Vec<ApplicationMutationOutputRoleDescriptor>,
    idempotency: ApplicationMutationIdempotencyMetadata,
    scope_entity: String,
    scope: ApplicationMutationScopeContract,
    principal: ApplicationMutationPrincipalBindingContract,
    candidates: ApplicationCandidateRequirements,
}

impl ApplicationMutationBindingDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new<
        InputBinding,
        ResultBinding,
        Operation,
        Decision,
        DenialBinding,
        Output,
        IdempotencyKey,
        Schema,
    >(
        identity: &'static str,
        operation_name: &'static str,
        handler_identity: &'static str,
        idempotency_identity: &'static str,
        scope_entity: &'static str,
        scope: ApplicationMutationScopeContract,
        principal: ApplicationMutationPrincipalBindingContract,
        candidates: ApplicationCandidateRequirements,
    ) -> Self
    where
        InputBinding: ApplicationStructuredValueBinding,
        ResultBinding: ApplicationStructuredValueBinding,
        Operation: 'static,
        Decision: 'static,
        DenialBinding: ApplicationStructuredValueBinding,
        Output: ApplicationMutationOutputContract<Schema>,
        IdempotencyKey: 'static,
        Schema: crate::application_schema::ApplicationSchema,
    {
        let description = ApplicationMutationDescription::from_untrusted_parts(
            ApplicationMutationDescriptionParts {
                binding_identity: WorthQueryPortableTypeIdentity::declared(identity),
                operation: operation_name.to_owned(),
                input_identity: InputBinding::IDENTITY,
                result_identity: ResultBinding::IDENTITY,
                denial_identity: DenialBinding::IDENTITY,
                scope: ApplicationMutationScopeDescription {
                    entity: scope_entity.to_owned(),
                    aspect: scope.field().locus().aspect().to_owned(),
                    field: scope.field().locus().field().to_owned(),
                    resolution: scope.mode(),
                },
                output_roles: Output::ROLES
                    .iter()
                    .map(|role| ApplicationMutationOutputRoleDescription {
                        name: role.name().to_owned(),
                        entity: role.entity().to_owned(),
                        posture: role.posture(),
                    })
                    .collect(),
            },
        );
        Self {
            description,
            denial_binding_type: TypeId::of::<DenialBinding>(),
            identity: identity.to_owned(),
            input_identity: InputBinding::IDENTITY,
            input_binding_type: TypeId::of::<InputBinding>(),
            input_type: TypeId::of::<InputBinding::Value>(),
            result_identity: ResultBinding::IDENTITY,
            result_binding_type: TypeId::of::<ResultBinding>(),
            result_type: TypeId::of::<ResultBinding::Value>(),
            operation_name: operation_name.to_owned(),
            operation_type: TypeId::of::<Operation>(),
            handler: ApplicationMutationHandlerMetadata::of::<Decision, DenialBinding::Value>(
                handler_identity,
            ),
            output_contract_type: TypeId::of::<Output>(),
            output_roles: Output::ROLES.to_vec(),
            idempotency: ApplicationMutationIdempotencyMetadata::of::<IdempotencyKey>(
                idempotency_identity,
            ),
            scope_entity: scope_entity.to_owned(),
            scope,
            principal,
            candidates,
        }
    }

    pub fn description(&self) -> &ApplicationMutationDescription {
        &self.description
    }

    pub fn denial_binding_type(&self) -> TypeId {
        self.denial_binding_type
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn input_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.input_identity
    }

    pub const fn input_binding_type(&self) -> TypeId {
        self.input_binding_type
    }

    pub const fn input_type(&self) -> TypeId {
        self.input_type
    }

    pub fn result_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.result_identity
    }

    pub const fn result_binding_type(&self) -> TypeId {
        self.result_binding_type
    }

    pub const fn result_type(&self) -> TypeId {
        self.result_type
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }

    pub const fn operation_type(&self) -> TypeId {
        self.operation_type
    }

    pub fn handler(&self) -> &ApplicationMutationHandlerMetadata {
        &self.handler
    }

    pub const fn output_contract_type(&self) -> TypeId {
        self.output_contract_type
    }

    pub fn output_roles(&self) -> &[ApplicationMutationOutputRoleDescriptor] {
        &self.output_roles
    }

    pub fn idempotency(&self) -> &ApplicationMutationIdempotencyMetadata {
        &self.idempotency
    }

    pub fn scope_entity(&self) -> &str {
        &self.scope_entity
    }

    pub fn scope(&self) -> &ApplicationMutationScopeContract {
        &self.scope
    }

    pub fn principal(&self) -> &ApplicationMutationPrincipalBindingContract {
        &self.principal
    }

    pub const fn candidates(&self) -> ApplicationCandidateRequirements {
        self.candidates
    }
}
