//! Minting, rewrapping and typed custody of an admission brand over its core.

use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBindingDescriptor, ApplicationSharedInput,
};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::brand_tag::BrandTag;
use super::original_input::ErasedOriginalInput;
use super::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdmittedOperation,
    WorthQueryOperationAdmissionIdentity,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryDelegationActivationBinding,
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::MutationHandlerExecutionDenial;

type Denial = WorthQueryOperationAuthorizationDenial;

fn inconsistent(core: &WorthQueryAdmittedOperation) -> Denial {
    Denial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InconsistentDecision,
        ),
        &core.operation,
    )
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    /// Brands a freshly admitted core. A conventional admission carries no
    /// typed capability input; every capability mode carries exactly one.
    pub(super) fn mint(
        core: WorthQueryAdmittedOperation,
        tag: BrandTag,
        capability_input: Option<ApplicationSharedInput<Input>>,
    ) -> Result<Self, Denial> {
        if core.authorization_basis.carries_capability_input() != capability_input.is_some() {
            return Err(inconsistent(&core));
        }
        tag.admits(&core)?;
        Ok(Self {
            core,
            tag,
            capability_input,
            original_input: None,
            _marker: PhantomData,
        })
    }

    /// Carries a successor core only when it continues the predecessor's
    /// admission and still matches this brand's tag.
    pub(super) fn rewrap(
        predecessor: WorthQueryOperationAdmissionIdentity,
        successor: WorthQueryAdmittedOperation,
        tag: BrandTag,
        capability_input: Option<ApplicationSharedInput<Input>>,
        original_input: Option<ApplicationSharedInput<Input>>,
    ) -> Result<Self, (Denial, WorthQueryAdmittedOperation)> {
        if successor.admission_identity != predecessor {
            return Err((inconsistent(&successor), successor));
        }
        if let Err(denial) = tag.admits(&successor) {
            return Err((denial, successor));
        }
        Ok(Self {
            core: successor,
            tag,
            capability_input,
            original_input,
            _marker: PhantomData,
        })
    }

    /// Ownership precedes lapse and every read; a brand whose core is not
    /// the one it was minted for is refused in the same slot.
    pub fn validate_runtime_ownership(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), Denial> {
        self.core
            .validate_runtime_ownership(runtime_authority, binding_identity)?;
        self.tag.admits(&self.core)
    }

    pub fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.core.belongs_to(runtime_authority, binding_identity)
            && self.tag.admits(&self.core).is_ok()
    }

    pub(in crate::domain_computation) fn validate_projection_authority(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), Denial> {
        self.validate_runtime_ownership(runtime_authority, binding_identity)?;
        self.core.validate_current_authority()
    }

    pub fn validate_installed_mutation_binding(
        &self,
        descriptor: Option<&ApplicationMutationBindingDescriptor>,
    ) -> Result<(), MutationHandlerExecutionDenial> {
        self.tag.admits_mutation_binding(&self.core, descriptor)
    }

    #[inline]
    pub fn capability_input(&self) -> Option<&Input> {
        self.capability_input.as_deref()
    }

    #[inline]
    pub(in crate::domain_computation) fn shared_capability_input(
        &self,
    ) -> Option<&ApplicationSharedInput<Input>> {
        self.capability_input.as_ref()
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
    pub(in crate::domain_computation) fn retain_original_input<Binding>(
        &mut self,
        input: ApplicationEncodedInput<Binding::InputBinding>,
        correction: Option<
            std::sync::Arc<
                crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure,
            >,
        >,
    ) where
        Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
            Operation = Operation,
            Input = Input,
        >,
        Input: Send + Sync + 'static,
    {
        // The caller has checked the sealed handler's binding and input identity.
        self.core.original_input = Some(ErasedOriginalInput {
            encoded: std::sync::Arc::new(input.clone()),
            program: None,
            identity: *input.identity(),
            binding_type: std::any::TypeId::of::<Binding>(),
            correction,
        });
        self.original_input = Some(input.into_shared_input());
    }

    pub(in crate::domain_computation) fn original_input(
        &self,
    ) -> Option<(&ApplicationSharedInput<Input>, [u8; 32])> {
        match (&self.original_input, self.core.original_input_identity()) {
            (Some(input), Some(identity)) => Some((input, identity)),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn bind_delegation_activation(
        self,
        binding: WorthQueryDelegationActivationBinding,
    ) -> Result<Self, Denial> {
        let Self {
            core,
            tag,
            capability_input,
            original_input,
            ..
        } = self;
        let predecessor = core.admission_identity;
        let successor = core.bind_delegation_activation(binding)?;
        Self::rewrap(
            predecessor,
            successor,
            tag,
            capability_input,
            original_input,
        )
        .map_err(|(denial, _)| denial)
    }

    pub(in crate::domain_computation) fn bind_capability_revocation(
        self,
        binding: WorthQueryCapabilityRevocationBinding,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) -> Result<Self, Denial> {
        let Self {
            core,
            tag,
            capability_input,
            original_input,
            ..
        } = self;
        let predecessor = core.admission_identity;
        let successor = core.bind_capability_revocation(binding, work)?;
        Self::rewrap(
            predecessor,
            successor,
            tag,
            capability_input,
            original_input,
        )
        .map_err(|(denial, _)| denial)
    }

    pub(in crate::domain_computation) fn bind_elevation_request(
        self,
        binding: WorthQueryElevationRequestBinding,
    ) -> Result<Self, Denial> {
        let Self {
            core,
            tag,
            capability_input,
            original_input,
            ..
        } = self;
        let predecessor = core.admission_identity;
        let successor = core.bind_elevation_request(binding)?;
        Self::rewrap(
            predecessor,
            successor,
            tag,
            capability_input,
            original_input,
        )
        .map_err(|(denial, _)| denial)
    }

    pub(in crate::domain_computation) fn bind_elevation_approval(
        self,
        binding: WorthQueryElevationApprovalBinding,
    ) -> Result<Self, (Denial, WorthQueryElevationApprovalBinding)> {
        let Self {
            core,
            tag,
            capability_input,
            original_input,
            ..
        } = self;
        let predecessor = core.admission_identity;
        let successor = core.bind_elevation_approval(binding)?;
        Self::rewrap(
            predecessor,
            successor,
            tag,
            capability_input,
            original_input,
        )
        .map_err(|(denial, mut successor)| {
            let binding = successor
                .take_elevation_approval_binding()
                .expect("a refused successor returns the binding it was just given");
            (denial, binding)
        })
    }

    pub(in crate::domain_computation) fn bind_elevation_close(
        self,
        binding: WorthQueryElevationCloseBinding,
    ) -> Result<Self, (Denial, WorthQueryElevationCloseBinding)> {
        let Self {
            core,
            tag,
            capability_input,
            original_input,
            ..
        } = self;
        let predecessor = core.admission_identity;
        let successor = core.bind_elevation_close(binding)?;
        Self::rewrap(
            predecessor,
            successor,
            tag,
            capability_input,
            original_input,
        )
        .map_err(|(denial, mut successor)| {
            let binding = successor
                .take_elevation_close_binding()
                .expect("a refused successor returns the binding it was just given");
            (denial, binding)
        })
    }

    pub(in crate::domain_computation) fn bind_mandatory_review(
        self,
        binding: WorthQueryMandatoryReviewBinding,
    ) -> Result<Self, (Denial, WorthQueryMandatoryReviewBinding)> {
        let Self {
            core,
            tag,
            capability_input,
            original_input,
            ..
        } = self;
        let predecessor = core.admission_identity;
        let successor = core.bind_mandatory_review(binding)?;
        Self::rewrap(
            predecessor,
            successor,
            tag,
            capability_input,
            original_input,
        )
        .map_err(|(denial, mut successor)| {
            let binding = successor
                .take_mandatory_review_binding()
                .expect("a refused successor returns the binding it was just given");
            (denial, binding)
        })
    }
}
