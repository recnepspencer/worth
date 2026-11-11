//! What a typed admission brand was minted for, checked against its core
//! and against the installed mutation binding it re-enters.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::MutationHandlerExecutionDenial;
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_operation::ApplicationMutationBindingDescriptor;

use super::WorthQueryAdmittedOperation;

/// Copied at mint from the installed operation; type identities serve
/// equality only.
pub(in crate::domain_computation) struct BrandTag {
    pub(super) runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub(super) operation_authority_identity_bytes: [u8; 32],
    pub(super) operation_definition_identity: [u8; 32],
    pub(super) operation_type: EqualityTypeId,
    pub(super) input_type: EqualityTypeId,
}

impl BrandTag {
    pub(super) fn new(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        operation_authority_identity_bytes: [u8; 32],
        operation_definition_identity: [u8; 32],
        operation_type: std::any::TypeId,
        input_type: std::any::TypeId,
    ) -> Self {
        Self {
            runtime_authority,
            operation_authority_identity_bytes,
            operation_definition_identity,
            operation_type: operation_type.into(),
            input_type: input_type.into(),
        }
    }

    /// The core carries the same runtime and sealed operation this brand was
    /// minted for.
    pub(super) fn admits(
        &self,
        core: &WorthQueryAdmittedOperation,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let refused = |kind| {
            Err(WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::Refused(kind),
                &core.data.operation,
            ))
        };
        if self.runtime_authority != core.data.runtime_authority {
            return refused(AuthorizationRefusal::ForeignRuntime);
        }
        if self.operation_authority_identity_bytes != core.data.operation_authority_identity_bytes
            || self.operation_definition_identity != core.data.operation_definition_identity
        {
            return refused(AuthorizationRefusal::StaleInstalledOperation);
        }
        Ok(())
    }

    /// The installed binding a mutation re-enters names this brand's
    /// operation and input types and the core's operation.
    pub(super) fn admits_mutation_binding(
        &self,
        core: &WorthQueryAdmittedOperation,
        descriptor: Option<&ApplicationMutationBindingDescriptor>,
    ) -> Result<(), MutationHandlerExecutionDenial> {
        let Some(descriptor) = descriptor else {
            return Err(MutationHandlerExecutionDenial::HandlerNotInstalled);
        };
        if self.operation_type != EqualityTypeId::from(descriptor.operation_type())
            || core.data.operation != descriptor.operation_name()
        {
            return Err(MutationHandlerExecutionDenial::Projection(
                WorthQueryOperationAuthorizationDenial::new(
                    WorthQueryOperationAuthorizationDenialKind::Refused(
                        AuthorizationRefusal::StaleInstalledOperation,
                    ),
                    &core.data.operation,
                )
                .into(),
            ));
        }
        if self.input_type != EqualityTypeId::from(descriptor.input_type()) {
            return Err(MutationHandlerExecutionDenial::InputNotAdmitted);
        }
        Ok(())
    }
}
