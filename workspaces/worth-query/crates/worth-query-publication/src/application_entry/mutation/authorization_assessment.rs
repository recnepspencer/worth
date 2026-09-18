use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationMutationRequest;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// A fresh authorization observation. It contains no admission or execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCurrentAuthorizationAssessment {
    private: (),
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
    pub fn assess_current_authorization(
        mut self,
    ) -> Result<WorthQueryCurrentAuthorizationAssessment, WorthQueryApplicationRequestMutationDenial>
    {
        super::authorization::assess(&mut self)?;
        Ok(WorthQueryCurrentAuthorizationAssessment { private: () })
    }
}
