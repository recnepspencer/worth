//! Capability-keyed eligibility: "who may exercise this capability?" read
//! from the installed capability contract itself, so an application never
//! re-declares a second policy beside its admission contract.
//!
//! The question carries the capability, its operation and the request
//! projection admission would derive. The verdict runs admission's own
//! capability observation (grant selection, currentness, validity at the one
//! trusted instant, composition rules and delegation lineage) and keeps only
//! its outcome: no decision fact is retained, no grant reserved, nothing
//! committed. A verdict is not authority; acting still admits the actor.

mod observation;
pub(super) mod prepared;
mod source;

use std::marker::PhantomData;

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRef, ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationOperationRef, ApplicationSchema,
};

use crate::domain_computation::authorization::WorthQueryApplicationAuthorizationExplanationCause;
use crate::domain_computation::primary_graph::resolution::WorthQueryCandidatePrincipalRefusal;

/// Whether one principal may exercise one installed capability on the
/// request a question projects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCapabilityEligibility {
    /// Admission would admit this principal's capability access.
    Eligible,
    /// Admission would refuse it for this policy cause: no current grant
    /// (missing, inactive, outside its validity or under a revoked extent), a
    /// failed composition rule, or a scope or purpose mismatch.
    Denied(WorthQueryApplicationAuthorizationExplanationCause),
    /// No mapping resolves this identity to a principal.
    Unmapped,
    /// The identity's mapping is disabled.
    Disabled,
    /// The identity resolves to more than one principal.
    Ambiguous,
}

impl WorthQueryCapabilityEligibility {
    pub const fn is_eligible(self) -> bool {
        matches!(self, Self::Eligible)
    }

    /// The policy cause of a denial, when the policy refused.
    pub const fn denial_cause(self) -> Option<WorthQueryApplicationAuthorizationExplanationCause> {
        match self {
            Self::Denied(cause) => Some(cause),
            _ => None,
        }
    }

    pub(crate) const fn of_refusal(refusal: WorthQueryCandidatePrincipalRefusal) -> Self {
        match refusal {
            WorthQueryCandidatePrincipalRefusal::Unmapped => Self::Unmapped,
            WorthQueryCandidatePrincipalRefusal::Disabled => Self::Disabled,
            WorthQueryCandidatePrincipalRefusal::Ambiguous => Self::Ambiguous,
        }
    }
}

/// One installed capability on its operation, by schema identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCapabilityKey {
    capability: &'static str,
    operation: &'static str,
}

impl WorthQueryCapabilityKey {
    pub const fn capability(&self) -> &'static str {
        self.capability
    }

    pub const fn operation(&self) -> &'static str {
        self.operation
    }
}

type Projection<Schema, Capability, Input> = ApplicationCapabilityRequestProjection<
    Schema,
    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
>;

/// One installed capability, exercised through its operation on one request
/// projection, asked through the principal binding a
/// `worth_query_principal_binding!` context names.
pub struct WorthQueryCapabilityEligibilityQuestion<Schema, Context, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    capability: ApplicationCapabilityRef<Schema, Capability>,
    operation: ApplicationOperationRef<Schema, Operation, Input>,
    projection: Projection<Schema, Capability, Input>,
    _context: PhantomData<fn() -> Context>,
}

impl<Schema, Context, Capability, Operation, Input>
    WorthQueryCapabilityEligibilityQuestion<Schema, Context, Capability, Operation, Input>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    /// Asks about `projection` exactly as given.
    pub fn new(
        _context: Context,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        projection: Projection<Schema, Capability, Input>,
    ) -> Self {
        Self {
            capability,
            operation,
            projection,
            _context: PhantomData,
        }
    }

    /// Asks about the projection `input` derives: the very request admission
    /// derives when the operation runs with that input.
    pub fn of_request(
        context: Context,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: &Input,
    ) -> Result<Self, ApplicationCapabilityRequestProjectionDenial> {
        Ok(Self::new(
            context,
            capability,
            operation,
            input.capability_request()?,
        ))
    }

    /// The installed capability this question asks about.
    pub const fn capability(&self) -> WorthQueryCapabilityKey {
        WorthQueryCapabilityKey {
            capability: self.capability.name(),
            operation: self.operation.name(),
        }
    }

    pub(crate) const fn projection(&self) -> &Projection<Schema, Capability, Input> {
        &self.projection
    }

    pub(crate) const fn references(
        &self,
    ) -> (
        ApplicationCapabilityRef<Schema, Capability>,
        ApplicationOperationRef<Schema, Operation, Input>,
    ) {
        (self.capability, self.operation)
    }
}
