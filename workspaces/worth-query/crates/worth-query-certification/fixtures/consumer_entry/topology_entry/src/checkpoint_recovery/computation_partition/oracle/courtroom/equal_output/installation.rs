//! The equal-output courtroom installs ordinary producers without computation.
use super::*;

mod graph;
pub(super) use graph::{DependentRoot, Root};
pub(super) mod authentication_fixture;
mod seed;

pub(super) struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("courtroom-equal-output-schema");
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        let builder = indexed_decision::declare_edit(facts::declare(builder));
        let builder = crate::source_adjustment::declare_planar_source_adjustment(builder);
        dependent_publication::declare(publication::declare(builder))
            .entity(Body::reference())
            .unit(Metre::reference())
            .aspect(Body::reference(), Geometry::reference())
            .field(Body::reference(), Length::reference())
            .aspect(Body::reference(), PlanarPosition::reference())
            .field(Body::reference(), BodyKey::reference())
            .field(Body::reference(), PositionX::reference())
            .field(Body::reference(), PositionY::reference())
            .relation(
                PlanarSuccessor::reference(),
                Body::reference(),
                Body::reference(),
            )
            .entity(ExternalPrincipalMapping::reference())
            .entity(Principal::reference())
            .aspect(
                ExternalPrincipalMapping::reference(),
                ExternalIdentity::reference(),
            )
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .field(
                ExternalPrincipalMapping::reference(),
                ExternalIdentityField::reference(),
            )
            .field(
                ExternalPrincipalMapping::reference(),
                MappingStatusField::reference(),
            )
            .field(Principal::reference(), PrincipalIdentity::reference())
            .relation(
                MappingTarget::reference(),
                ExternalPrincipalMapping::reference(),
                Principal::reference(),
            )
            .principal_binding(ConsumerPrincipalBinding::reference())
            .application_query(crate::planar_query_definition::<Schema>())
            .application_query_binding::<PlanarReadBinding<Schema>>()
            .application_query(counted_source::counted_query_definition::<Schema>())
            .application_query_binding::<counted_source::CountedReadBinding<Schema>>()
            .application_query(crate::planar_output_query_definition::<Schema>())
            .application_query_binding::<PlanarOutputReadBinding<Schema>>()
            .application_query(source::successor_query_definition::<Schema>())
            .application_query_binding::<source::SuccessorReadBinding<Schema>>()
    }
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
                .mutation::<PlanarSourceAdjustmentBinding<Schema>>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .conditional_operation::<publication::PublishRoot>()
                .mutation::<PlanarEditBinding<Schema>>()
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
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .unwrap();
    let declaration = Schema::declaration().unwrap();

    let limits = support::limits_with_room(
        32,
        16,
        64,
        support::invalidation(128 * 1024 * 1024, 1_000_000, 4),
        support::candidates(),
    );
    application_installation::in_memory_program(program, declaration, ((witnesses, indexed_selection),), limits, |graph, installed| {
        let binding = installed.principal_binding(ConsumerPrincipalBinding::reference::<Schema>()).unwrap();
        let external = worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new("https://checkpoint.invalid/local", "model-owner").unwrap();
        graph.bind_principal(&binding, primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(), 1_u64, external, worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
        seed::seed_cycle(graph); Ok(())
    }).unwrap()
}
