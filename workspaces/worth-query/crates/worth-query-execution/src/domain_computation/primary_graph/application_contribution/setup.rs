use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryMarkerIdentity,
};
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
use super::conditional::PendingConditionalRegistry;
use super::producer::PendingProducerRegistry;
use super::{WorthQueryApplicationConditionalBinding, WorthQueryApplicationProducerBinding};

/// Configuration access restricted to one contribution in the exact installed schema.
pub struct WorthQueryApplicationContributionSetup<'a, Schema>
where
    Schema: ApplicationSchema,
{
    installed_schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
    contribution: WorthQueryInstalledApplicationContribution<'a>,
    handlers: &'a mut PendingMutationHandlerRegistry<Schema>,
    factories: &'a mut WorthQueryApplicationInvariantFactories<Schema>,
    producers: &'a mut PendingProducerRegistry<Schema>,
    conditionals: &'a mut PendingConditionalRegistry<Schema>,
}

impl<'a, Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'a, Schema> {
    pub(super) fn new(
        installed_schema: &'a WorthQueryInstalledApplicationSchema<Schema>,
        contribution: WorthQueryInstalledApplicationContribution<'a>,
        handlers: &'a mut PendingMutationHandlerRegistry<Schema>,
        factories: &'a mut WorthQueryApplicationInvariantFactories<Schema>,
        producers: &'a mut PendingProducerRegistry<Schema>,
        conditionals: &'a mut PendingConditionalRegistry<Schema>,
    ) -> Self {
        Self {
            installed_schema,
            contribution,
            handlers,
            factories,
            producers,
            conditionals,
        }
    }

    pub fn conditional<Binding>(
        &mut self,
        configuration: Binding::Configuration,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: 'static,
        Binding: WorthQueryApplicationConditionalBinding<Schema>,
    {
        self.conditionals
            .register::<Binding>(self.contribution.identity().as_str(), configuration)
    }

    pub fn computation<Feature, Computation, Owner>(
        &mut self,
        owner: Owner,
    ) -> Result<
        super::WorthQueryInstalledManagedComputation<Schema, Feature, Computation, Owner>,
        WorthQueryPrimaryGraphInstallationDenial,
    >
    where
        Feature: worth_query_declaration::facade::application_program::ApplicationFeature<Schema>,
        Computation:
            worth_query_declaration::facade::application_program::ApplicationManagedComputation<
                Schema,
                Feature,
            >,
        Owner: super::WorthQueryManagedComputationOwner<Schema, Feature, Computation>,
    {
        self.handlers
            .install_computation::<Feature, Computation, Owner>(owner)
    }

    pub fn producer<Binding>(
        &mut self,
        provider: Binding::Provider,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.require_owned_query::<<Binding::OutputFamily as super::WorthQueryProducerOutputFamily<Schema>>::Source>()?;
        self.require_owned_mutation::<Binding::Operation>()?;
        for requirement in Binding::REQUIRED_INVARIANTS {
            self.require_owned_invariant(*requirement)?;
        }
        self.producers
            .register::<Binding>(self.contribution.identity().as_str(), provider)
    }

    fn require_owned_mutation<Binding>(
        &self,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let installed = self
            .installed_schema
            .installed_mutation_binding::<Binding>()
            .map_err(|_| member_denial(Binding::IDENTITY))?;
        let operation_owned = self.contribution.members().any(|member| {
            matches!(member,
                ApplicationSchemaMember::ApplicationMutation { description }
                    if description.binding_identity().as_str() == installed.identity()
            )
        });
        if !operation_owned {
            return Err(member_denial(Binding::IDENTITY));
        }
        Ok(())
    }

    fn require_owned_query<Binding>(&self) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        self.installed_schema
            .installed_query_binding::<Binding>()
            .map_err(|_| member_denial(Binding::IDENTITY))?;
        let query_owned = self.contribution.members().any(|member| {
            matches!(member,
                ApplicationSchemaMember::ApplicationQuery { definition }
                    if definition.query_type() == Binding::Query::QUERY_TYPE_NAME
            )
        });
        if !query_owned {
            return Err(member_denial(Binding::IDENTITY));
        }
        Ok(())
    }

    fn require_owned_invariant(
        &self,
        requirement: super::WorthQueryProducerInvariantRequirement,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let installed = self
            .installed_schema
            .invariants()
            .descriptors()
            .any(|descriptor| {
                descriptor.identifier() == requirement.identifier()
                    && descriptor.major() == requirement.major()
                    && descriptor.minor() == requirement.minor()
                    && descriptor.execution_point() == requirement.execution_point()
            });
        let owned = self.contribution.members().any(|member| {
            matches!(member,
                ApplicationSchemaMember::ApplicationInvariant {
                    invariant,
                    major,
                    minor,
                    execution_point,
                    ..
                } if invariant == requirement.identifier()
                    && *major == requirement.major()
                    && *minor == requirement.minor()
                    && *execution_point == requirement.execution_point()
            )
        });
        if !installed || !owned {
            return Err(member_denial(requirement.identifier()));
        }
        Ok(())
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
        Rule: WorthQueryApplicationInvariantRule<Schema>,
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
