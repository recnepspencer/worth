//! One installed workflow vocabulary, borrowed for a single request.
//!
//! A workflow runtime carries the vocabulary installed against its initial
//! program and, after adoption, vocabularies installed against programs the
//! host rostered. Each request names exactly one of them. The view borrows the
//! host, the installed spec and the signing owner together, so a request can
//! never pair one program's vocabulary with another host's runtime.

use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowSpec, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::WorthQueryWorkflowApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// The workflow vocabulary one request prepares against.
///
/// Its revision is the installed program's revision; whether that program is
/// the one a branch activated is decided when the request prepares, so a view
/// for a program the branch does not run is refused before any effect.
pub struct WorthQueryWorkflowVocabulary<'vocabulary, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    runtime: &'vocabulary WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    workflow: &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
    authentication: &'vocabulary WorthQueryAuthenticationEventSigningOwner<Schema>,
}

impl<'vocabulary, Schema, Spec, Program>
    WorthQueryWorkflowVocabulary<'vocabulary, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) const fn new(
        runtime: &'vocabulary WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        workflow: &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>,
        authentication: &'vocabulary WorthQueryAuthenticationEventSigningOwner<Schema>,
    ) -> Self {
        Self {
            runtime,
            workflow,
            authentication,
        }
    }

    /// The host this vocabulary was installed on.
    pub const fn runtime(&self) -> &'vocabulary WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime
    }

    pub const fn workflow_spec(
        &self,
    ) -> &'vocabulary WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program> {
        self.workflow
    }

    pub const fn authentication_owner(
        &self,
    ) -> &'vocabulary WorthQueryAuthenticationEventSigningOwner<Schema> {
        self.authentication
    }
}

impl<Schema, Spec, Program> Clone for WorthQueryWorkflowVocabulary<'_, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Spec, Program> Copy for WorthQueryWorkflowVocabulary<'_, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
}

impl<'vocabulary, Schema, Spec, Program>
    From<&'vocabulary WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>>
    for WorthQueryWorkflowVocabulary<'vocabulary, Schema, Spec, Program>
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
