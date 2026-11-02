use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationMutationRequest;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// A fresh authorization observation. It contains no admission or execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCurrentAuthorizationAssessment {
    private: (),
}

impl WorthQueryCurrentAuthorizationAssessment {
    pub(in crate::application_entry) fn observed() -> Self {
        Self { private: () }
    }
}

impl<Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Observes conventional operation authorization. Capability mutations use
    /// `assess_current_capability_authorization` instead.
    pub fn assess_current_authorization(
        mut self,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        let application = self.application;
        let scope = self.scope.clone();
        application
            .with_application_advancement(&scope, |phase| {
                super::authorization::assess(&phase, &mut self)?;
                Ok(WorthQueryCurrentAuthorizationAssessment { private: () })
            })
            .map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?
    }

    /// Observes current capability authorization without claiming a key, running
    /// a handler, or retaining execution authority. Execution must authorize again.
    pub fn assess_current_capability_authorization(
        mut self,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    where
        Intent::Binding: ApplicationCapabilityMutationBinding<Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone
            + worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                Scope = <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
{
        let application = self.application;
        let scope = self.scope.clone();
        application
            .with_application_advancement(&scope, |phase| {
                let selected = application
                    .on_branch(self.branch)
                    .select(&phase)
                    .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
                super::authorization::admit_capability_for_assessment(
                    &phase, &mut self, &selected,
                )?;
                Ok(WorthQueryCurrentAuthorizationAssessment { private: () })
            })
            .map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?
    }
}
