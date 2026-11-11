//! The equal-output courtroom installs ordinary producers without computation.
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

mod graph;
pub(super) use crate::checkpoint_recovery::support::authentication_fixture;
pub(super) use graph::{DependentRoot, Root};
mod seed;

pub(super) struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("courtroom-equal-output-schema");
    type Members = (
        crate::checkpoint_recovery::output_availability::members::AvailabilityRecords,
        crate::CheckpointRecords,
        crate::PrincipalRecords,
        crate::document_history::HistoryGrantRecords,
        ContributionContracts,
    );
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = (usize, bool);
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<Producer>()?;
        contracts.producer::<counted_producer::Producer>()?;
        contracts.conditional::<readiness::Readiness>()?;
        contracts.conditional::<readiness::root::Readiness>()?;
        Ok(())
    }
    fn configure(
        configuration: (usize, bool),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<dependent_publication::Binding, _>(dependent_publication::Handler {
            witnesses: configuration.0,
            indexed_selection: configuration.1,
        })?;
        setup.handler::<publication::Binding, _>(publication::Handler)?;
        setup.handler::<PlanarEditBinding<Schema>, _>(crate::PlanarHandler)?;
        setup.handler::<PlanarSourceAdjustmentBinding<Schema>, _>(PlanarSourceAdjustmentHandler)?;
        setup.producer::<Producer>(Provider)?;
        setup.producer::<counted_producer::Producer>(counted_producer::Provider)?;
        setup.conditional::<readiness::Readiness>(())?;
        setup.conditional::<readiness::root::Readiness>(())?;
        Ok(())
    }
}
pub(super) struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Root, DependentRoot)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("courtroom-equal-output-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarSourceAdjustmentBinding<Schema>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .conditional_operation::<publication::PublishRoot>()
                .native_action::<PlanarEditBinding<Schema>>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, graph::DependentFeature>()
                .provides::<graph::DependentOutput>()
                .conditional_operation::<dependent_publication::PublishDependent>()
                .finish(),
        ]
    }
}
pub(super) fn install(
) -> application_installation::WorthQueryProgramApplicationRuntime<Schema, Program> {
    install_with_witnesses(0)
}
pub(super) fn install_with_witnesses(
    witnesses: usize,
) -> application_installation::WorthQueryProgramApplicationRuntime<Schema, Program> {
    install_configured(witnesses, false)
}

pub(super) fn install_with_indexed_decision(
) -> application_installation::WorthQueryProgramApplicationRuntime<Schema, Program> {
    install_configured(0, true)
}

fn install_configured(
    witnesses: usize,
    indexed_selection: bool,
) -> application_installation::WorthQueryProgramApplicationRuntime<Schema, Program> {
    assert!(witnesses <= 2);
    let declaration =
        <Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration()
            .unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();

    let limits = support::limits_with_room(32, 16, 64, support::invalidation());
    application_installation::in_memory_program(program, declaration, ((witnesses, indexed_selection),), ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(), limits, |_bootstrap_phase, graph, installed| {
        let binding = installed.principal_binding(ConsumerPrincipalBinding::reference::<Schema>()).unwrap();
        let external = worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new("https://checkpoint.invalid/local", "model-owner").unwrap();
        graph.bind_principal(&binding, primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(), 1_u64, external, worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
        seed::seed_cycle(graph); Ok(())
    }).unwrap()
}

pub struct ContributionContracts;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema>
    for ContributionContracts
{
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok({
            let builder = indexed_decision::declare_edit(builder);
            let builder = <crate::PlanarSourceFeatureDefinition<Schema> as worth_query_decl::facade::application_program::ApplicationFeatureDefinition<Schema>>::declare_members(builder);
            dependent_publication::declare(publication::declare(builder))
                .principal_binding(ConsumerPrincipalBinding::reference())
                .application_query(crate::planar_query_definition::<Schema>())
                .application_query_binding::<PlanarReadBinding<Schema>>()
                .application_query(counted_source::counted_query_definition::<Schema>())
                .application_query_binding::<counted_source::CountedReadBinding<Schema>>()
                .application_query(crate::planar_output_query_definition::<Schema>())
                .application_query_binding::<PlanarOutputReadBinding<Schema>>()
                .application_query(source::successor_query_definition::<Schema>())
                .application_query_binding::<source::SuccessorReadBinding<Schema>>()
        })
    }
}
