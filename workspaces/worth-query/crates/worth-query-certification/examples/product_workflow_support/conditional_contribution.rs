use std::sync::Arc;

use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationConditionalBinding, WorthQueryApplicationConditionalPackageContract,
        WorthQueryApplicationConditionalProducerAccess, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
    },
    declaration::{
        application_query::ApplicationQueryParameterSet,
        application_schema::{
            ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
        },
    },
    domain, primary_graph,
};

use super::adapters::{
    ClockSource, ExampleClock, IntentProjector, Invoker, Predicate, PrincipalSource,
};
use super::application::admit_identity_adapter;
use super::contract::{self, TemporalReadyNode};
use super::integrity;
use super::schema::{
    ExecuteTemporal, IntentIdentityField, IntentLifecycleField, IntentRevisionField,
    TemporalHostContribution, TemporalHostSchema, TemporalIntegrity, TemporalIntentQuery,
    TemporalPrincipalBinding,
};

pub struct TemporalContributionConfiguration {
    pub clock_source: ClockSource,
}

pub struct InstalledTemporalConditional {
    pub clock: primary_graph::WorthQueryConditionalClockHandle<
        TemporalHostSchema,
        TemporalReadyNode,
        ExampleClock,
    >,
    pub invariant:
        Arc<primary_graph::WorthQueryApplicationInvariantProjectionAuthority<TemporalHostSchema>>,
}

pub struct TemporalConditional;

impl WorthQueryApplicationConditionalBinding<TemporalHostSchema> for TemporalConditional {
    type Configuration = TemporalContributionConfiguration;
    type Installed = InstalledTemporalConditional;
    type Operation = ExecuteTemporal;

    const IDENTITY: &'static str = "worth.query.example.temporal-conditional.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[];

    fn package_contract() -> WorthQueryApplicationConditionalPackageContract {
        WorthQueryApplicationConditionalPackageContract::new(
            contract::operation_definition().into_portable(),
            contract::conditional_binding().portable().clone(),
            TemporalReadyNode::reference().node_identity(),
        )
    }

    fn install(
        configuration: Self::Configuration,
        _: &WorthQueryApplicationConditionalProducerAccess<'_, TemporalHostSchema>,
        installation: &mut primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<
            TemporalHostSchema,
        >,
    ) -> Result<Self::Installed, primary_graph::WorthQueryConditionalRuntimeInstallationDenial>
    {
        let schema = installation.installed_schema();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the temporal principal binding is declared");
        let authentication = admit_identity_adapter(schema);
        let operation = schema
            .installed_operation(ExecuteTemporal::reference())
            .expect("the temporal operation is declared");
        let query = schema
            .certification_query(TemporalIntentQuery::reference())
            .expect("the temporal query is declared");
        let conditional = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &contract::conditional_binding())
            .expect("the conditional operation is declared")
            .bind_node(TemporalReadyNode::reference())
            .expect("the conditional node is declared")
            .bind_host_predicate_provider(Predicate)
            .expect("the temporal predicate is installed")
            .bind_named_clock::<ExampleClock, _>(configuration.clock_source)
            .expect("the temporal clock is installed")
            .bind_temporal_intent_projection(
                query,
                ApplicationQueryParameterSet::new(),
                IntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 16, 8)
                    .expect("valid temporal bounds"),
            )
            .expect("the temporal query projection is installed");
        let invariant = Arc::new(installation.retain_invariant_projection_authority());
        let execution = primary_graph::WorthQueryTemporalOperationExecution::with_authorization(
            Arc::clone(&invariant),
            Invoker,
            IntentIdentityField::reference(),
            IntentRevisionField::reference(),
            IntentLifecycleField::reference(),
            "active".to_owned(),
            "completed".to_owned(),
            primary_graph::WorthQueryPublicTemporalOperationAuthorization,
        )
        .expect("the temporal invocation is installed");
        let reconstruction = primary_graph::WorthQueryTemporalReconstructionAccess::new(
            principal_binding,
            PrincipalSource::new(authentication),
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
        )
        .expect("the temporal reconstruction source is installed");
        let clock = installation.bind_temporal_operation(conditional, execution, reconstruction)?;
        Ok(InstalledTemporalConditional { clock, invariant })
    }
}

impl WorthQueryApplicationContribution<TemporalHostSchema> for TemporalHostContribution {
    type Configuration = TemporalContributionConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.conditional::<TemporalConditional>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            TemporalIntegrity::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            integrity::resolve_rule,
        )?;
        setup.handler::<super::application_entry::AmendTemporalBinding, _>(
            super::application_entry::AmendTemporalHandler,
        )?;
        setup.conditional::<TemporalConditional>(configuration)
    }
}
