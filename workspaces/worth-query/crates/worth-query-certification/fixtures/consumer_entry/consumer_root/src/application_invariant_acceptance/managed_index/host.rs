//! Genuine application contribution, authenticated principal and committed source.
use super::super::{authentication, resources};
use super::{boxes, directory, mutation, prefix, schema::*};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::Arc;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*,
    declaration::authentication::WorthQueryPrincipalMappingStatus, primary_graph::*,
};

pub struct Program;
pub struct Feature;
impl ApplicationFeature<Catalog> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "managed-catalog.feature.v1";
}
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (CatalogContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("managed-catalog.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Catalog, Feature>()
            .native_action::<directory::Binding>()
            .finish_action()
            .native_action::<mutation::Binding>()
            .finish_action()
            .native_action::<boxes::status::Binding>()
            .finish_action()
            .operation::<boxes::access::InspectBoxes>()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Catalog> for CatalogContribution {
    type Configuration = Configuration;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        (configuration.configure_prefix)(configuration.prefix, setup)?;
        setup.handler::<mutation::Binding, _>(mutation::Handler)?;
        setup.handler::<directory::Binding, _>(directory::Handler)?;
        boxes::configure(configuration.boxes, configuration.incompatible, setup)
    }
}
pub struct Configuration {
    pub prefix: Arc<prefix_observation::Calls>,
    pub configure_prefix: fn(
        Arc<prefix_observation::Calls>,
        &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    pub boxes: Arc<boxes::Calls>,
    pub incompatible: bool,
}
/// Own the actual runtime installed through this fixture's one public factory.
pub struct Host<S = Catalog, P = Program> {
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
}
impl<S, P> std::ops::Deref for Host<S, P> {
    type Target = WorthQueryProgramApplicationRuntime<S, P>;
    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
impl<S, P> Host<S, P>
where
    S: worth_query_decl::facade::application_schema::ApplicationSchemaComposition,
    P: ApplicationProgramDefinition<S> + 'static,
    P::Outputs: ApplicationProgramOutputsShape<S> + WorthQueryApplicationProgramRoots<S>,
    P::Contributions: WorthQueryApplicationContributionTuple<S>,
{
    pub fn install(
        declaration: worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<S>,
        configuration: <P::Contributions as WorthQueryApplicationContributionTuple<S>>::Configuration,
        initial_state: impl FnOnce(
            &WorthQueryBootstrapAdvancementPhase<'_>,
            &mut WorthQueryPrimaryGraphBootstrap<S>,
            &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<S>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    ) -> Result<Self, WorthQueryApplicationPreparationDenial> {
        let program = ApplicationProgramAuthoring::<S, P>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap();
        in_memory_program(
            program,
            declaration,
            configuration,
            ApplicationBuildIdentity::parse("19191919-1919-1919-1919-191919191919").unwrap(),
            resources::application_limits(),
            initial_state,
        )
        .map(|runtime| Self { runtime })
    }
}
pub fn install(calls: Arc<prefix_observation::Calls>) -> Host {
    install_boxes(calls, Arc::new(boxes::Calls::default()))
}
pub fn install_boxes(prefix: Arc<prefix_observation::Calls>, boxes: Arc<boxes::Calls>) -> Host {
    try_install(prefix, boxes, false)
        .expect("the actual public managed sources and both providers install")
}
pub fn try_install(
    prefix: Arc<prefix_observation::Calls>,
    boxes: Arc<boxes::Calls>,
    incompatible: bool,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    install_configuration(Configuration {
        prefix,
        configure_prefix: |calls, setup| {
            setup.managed_index::<prefix::PrefixIndex, _>(prefix::Provider(calls))
        },
        boxes,
        incompatible,
    })
}
pub(super) fn install_configuration(
    configuration: Configuration,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    let declaration =
        <Catalog as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration()
            .unwrap();
    Host::install(declaration, (configuration,), |_, graph, installed| {
        graph.bind_principal(
            &installed
                .principal_binding(CatalogPrincipal::reference())
                .unwrap(),
            WorthQueryApplicationPrincipalKey::new("catalog-reader").unwrap(),
            1u64,
            authentication::external_identity(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )?;
        for directory in [
            "catalog",
            "empty",
            "boxes",
            "empty-boxes",
            "invalid-boxes",
            "other-boxes",
        ] {
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Directory::reference(),
                    directory_key(directory),
                )
                .field(
                    DirectoryIdentity::at("DirectoryIdentity"),
                    directory.to_owned(),
                ),
            )?;
        }
        for (identity, name, enabled) in [
            ("item-a", "beta", true),
            ("item-b", "alpha", true),
            ("item-c", "alpha", true),
            ("item-d", "alpine", false),
            ("item-e", "", true),
            ("item-f", "álamo", true),
            ("item-g", "alphabet", true),
        ] {
            let key = WorthQueryApplicationEntityKey::new(identity).unwrap();
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Item::reference(),
                    WorthQueryApplicationEntityKey::new(identity).unwrap(),
                )
                .field(ItemIdentity::at("ItemIdentity"), identity.to_owned())
                .field(ItemName::at("ItemName"), name.to_owned().into())
                .field(ItemEnabled::at("ItemEnabled"), enabled)
                // Name-only rows still populate every required ItemFacts field.
                .field(
                    boxes::ItemBounds::at("ItemBounds"),
                    boxes::Coordinates::new(0, 0, 0, 0),
                ),
            )?;
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Contains::reference(),
                identity,
                directory_key("catalog"),
                key,
            ))?;
        }
        boxes::seed::populate(graph)?;
        Ok(())
    })
}
fn directory_key(identity: &str) -> WorthQueryApplicationEntityKey<Catalog, Directory> {
    WorthQueryApplicationEntityKey::new(identity).unwrap()
}
