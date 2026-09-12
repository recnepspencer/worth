use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
    ApplicationInvariantRef, ApplicationSchema, ApplicationSchemaMember,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationContribution, WorthQueryInstalledApplicationSchema,
};

use super::super::application_entry::mutation::OperationHandler;
use super::super::application_invariant::WorthQueryApplicationInvariantRule;
use super::super::handler::PendingMutationHandlerRegistry;
use super::super::{
    WorthQueryApplicationInvariantFactories, WorthQueryApplicationInvariantSchemaResolver,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

/// Configuration access restricted to one contribution in the exact installed schema.
pub struct WorthQueryApplicationContributionSetup<'a, Schema> {
    installed_schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
    contribution: WorthQueryInstalledApplicationContribution<'a>,
    handlers: &'a mut PendingMutationHandlerRegistry<Schema>,
    factories: &'a mut WorthQueryApplicationInvariantFactories<Schema>,
}

impl<'a, Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'a, Schema> {
    pub(super) fn new(
        installed_schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
        contribution: WorthQueryInstalledApplicationContribution<'a>,
        handlers: &'a mut PendingMutationHandlerRegistry<Schema>,
        factories: &'a mut WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Self {
        Self {
            installed_schema,
            contribution,
            handlers,
            factories,
        }
    }

    pub fn handler<Binding, Handler>(
        &mut self,
        handler: Handler,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Handler: OperationHandler<Schema, Binding>,
    {
        let installed = self
            .installed_schema
            .installed_mutation_binding::<Binding>()
            .map_err(|_| member_denial(Binding::IDENTITY))?;
        let owned = self.contribution.members().any(|member| {
            matches!(member,
                ApplicationSchemaMember::ApplicationMutation { description }
                    if description.binding_identity().as_str() == installed.identity()
            )
        });
        if !owned {
            return Err(member_denial(Binding::IDENTITY));
        }
        self.handlers.register(
            &self.installed_schema.binding_identity(),
            &installed,
            handler,
        )
    }

    pub fn invariant<Invariant, Rule>(
        &mut self,
        reference: ApplicationInvariantRef<Schema, Invariant>,
        execution_point: ApplicationInvariantExecutionPoint,
        factory: impl for<'resolver> FnOnce(
                &WorthQueryApplicationInvariantSchemaResolver<'resolver, Schema>,
            ) -> Result<Rule, String>
            + Send
            + 'static,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Invariant: ApplicationInvariantMarkerIdentity<Schema>,
        Rule: WorthQueryApplicationInvariantRule,
    {
        let installed = self
            .installed_schema
            .installed_invariant(reference, execution_point)
            .ok_or_else(|| member_denial(reference.identifier()))?;
        let owned = self.contribution.members().any(|member| matches!(member,
            ApplicationSchemaMember::ApplicationInvariant { invariant, major, minor, execution_point: point, .. }
                if invariant == reference.identifier() && *major == reference.major()
                    && *minor == reference.minor() && *point == execution_point
        ));
        if !owned {
            return Err(member_denial(reference.identifier()));
        }
        self.factories.bind(&installed, factory)
    }
}

fn member_denial(subject: &str) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::ContributionMemberMismatch,
        subject,
    )
}
