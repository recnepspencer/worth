//! A governed read's capability, admitted for this request's own principal.
//!
//! Inspection and history reads consume move-only admitted accesses. This
//! request admits one from its declared capability, operation and input at
//! the request branch's current head, resolving the principal through the
//! named read binding exactly as that binding's own query admission does.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::ApplicationQueryBinding,
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    primary_graph::{
        WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryPrincipalResolutionMode,
    },
};

use super::super::WorthQueryApplicationRequestQueryDenial as Refused;

/// One capability a governed read admits for its principal: the declared
/// capability, its operation and the request input.
pub struct WorthQueryGovernedReadAccess<Schema, Capability, Operation, Input> {
    capability: ApplicationCapabilityRef<Schema, Capability>,
    operation: ApplicationOperationRef<Schema, Operation, Input>,
    input: Input,
}

impl<Schema, Capability, Operation, Input>
    WorthQueryGovernedReadAccess<Schema, Capability, Operation, Input>
{
    pub(in crate::application_entry) fn capability(
        &self,
    ) -> ApplicationCapabilityRef<Schema, Capability> {
        self.capability
    }

    pub(in crate::application_entry) fn operation(
        &self,
    ) -> ApplicationOperationRef<Schema, Operation, Input> {
        self.operation
    }

    pub(in crate::application_entry) fn input(&self) -> Input
    where
        Input: Clone,
    {
        self.input.clone()
    }

    /// The declared capability, operation and input, for a governed read
    /// that admits them itself (`query_governed`, `execute_governed`).
    pub fn into_parts(
        self,
    ) -> (
        ApplicationCapabilityRef<Schema, Capability>,
        ApplicationOperationRef<Schema, Operation, Input>,
        Input,
    ) {
        (self.capability, self.operation, self.input)
    }

    pub const fn new(
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Self {
        Self {
            capability,
            operation,
            input,
        }
    }
}

impl<'a, 'p, 'r, Schema: ApplicationSchema> WorthQueryApplicationRequest<'a, 'p, 'r, Schema> {
    /// Admits `access` for this request's principal, resolved through
    /// `Binding`'s principal binding, at the request branch's current head.
    /// The admitted access is consumed by the one inspection or history step
    /// it is passed to; a later step admits again.
    pub fn admit_read_access_in_advancement<Binding, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryGovernedReadAccess<Schema, Capability, Operation, Input>,
    ) -> Result<
        WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        Refused,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + Clone + 'static,
    {
        let schema = self.application.installed_schema();
        let binding = schema
            .installed_query_binding::<Binding>()
            .map_err(Refused::BindingInstallation)?;
        let capability = schema
            .capability(access.capability, access.operation)
            .map_err(Refused::CapabilityInstallation)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select(phase)
            .map_err(Refused::ProductSelection)?;
        let principal = selected
            .resolve_authenticated_principal(
                phase,
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(Refused::PrincipalResolution)?;
        selected
            .admit_capability_access(
                phase,
                &principal,
                &capability,
                access.input.clone(),
                self.scope,
            )
            .map_err(Refused::CapabilityAdmission)
    }
}
