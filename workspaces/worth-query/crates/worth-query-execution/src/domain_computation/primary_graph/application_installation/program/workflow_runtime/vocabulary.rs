//! The installed workflow vocabularies, borrowed for a single request.
//!
//! A workflow runtime carries the vocabulary installed against its initial
//! program and, after adoption, vocabularies installed against programs the
//! host rostered. A request never names one: it prepares against the one
//! installed for the program its selected branch runs. The view borrows the
//! host, the installed specs and the signing owner together, so a request can
//! never pair one host's vocabulary with another host's runtime.

use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::WorthQueryWorkflowApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

/// The workflow vocabularies one request prepares against.
pub struct WorthQueryWorkflowVocabulary<'vocabulary, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    runtime: &'vocabulary WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    initial: &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    supported: &'vocabulary [WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>],
    authentication: &'vocabulary WorthQueryAuthenticationEventSigningOwner<Schema>,
}

impl<'vocabulary, Schema, Spec> WorthQueryWorkflowVocabulary<'vocabulary, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) const fn new(
        runtime: &'vocabulary WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        initial: &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        supported: &'vocabulary [WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>],
        authentication: &'vocabulary WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Self {
        Self {
            runtime,
            initial,
            supported,
            authentication,
        }
    }

    /// The host this vocabulary was installed on.
    pub const fn runtime(&self) -> &'vocabulary WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime
    }

    /// The spec installed for the program the selected branch runs.
    ///
    /// When the branch's program cannot be inspected or has no installed
    /// vocabulary, this is the initial program's spec, whose revision then
    /// refuses the request when it prepares, before any effect.
    pub fn workflow_spec_for(
        &self,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    ) -> &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec> {
        let Ok(inspection) = selected.inspect_selected_program() else {
            return self.initial;
        };
        self.supported
            .iter()
            .find(|supported| supported.program_revision() == inspection.revision())
            .unwrap_or(self.initial)
    }

    /// [`Self::workflow_spec_for`] the branch's current head, for work that
    /// binds before it selects. Whatever later selects rechecks the revision.
    pub fn workflow_spec_on(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec> {
        match self.runtime.on_branch(branch).select() {
            Ok(selected) => self.workflow_spec_for(&selected),
            Err(_) => self.initial,
        }
    }

    pub const fn authentication_owner(
        &self,
    ) -> &'vocabulary WorthQueryAuthenticationEventSigningOwner<Schema> {
        self.authentication
    }
}

impl<Schema, Spec> Clone for WorthQueryWorkflowVocabulary<'_, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Spec> Copy for WorthQueryWorkflowVocabulary<'_, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
}

impl<'vocabulary, Schema, Spec, Program>
    From<&'vocabulary WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>>
    for WorthQueryWorkflowVocabulary<'vocabulary, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn from(
        runtime: &'vocabulary WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>,
    ) -> Self {
        runtime.vocabulary()
    }
}
