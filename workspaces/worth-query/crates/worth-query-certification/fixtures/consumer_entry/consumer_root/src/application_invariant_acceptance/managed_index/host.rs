//! Genuine application contribution, authenticated principal and committed source.
use super::super::{authentication, resources};
use super::{boxes, directory, mutation, prefix, schema::*};
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
        match configuration.prefix {
            PrefixExecutable::Current(provider) => {
                setup.managed_index::<prefix::PrefixIndex, _>(provider)?
            }
            PrefixExecutable::Incompatible(provider) => {
                setup.managed_index::<prefix::PrefixIndex, _>(provider)?
            }
        }
        setup.handler::<mutation::Binding, _>(mutation::Handler)?;
        setup.handler::<directory::Binding, _>(directory::Handler)?;
        boxes::configure(configuration.boxes, configuration.incompatible, setup)
    }
}
pub struct Configuration {
    pub prefix: PrefixExecutable,
    pub boxes: Arc<boxes::Calls>,
    pub incompatible: bool,
}
pub enum PrefixExecutable {
    Current(prefix::Provider),
    Incompatible(prefix::IncompatibleVersion),
}
pub type Host = WorthQueryProgramApplicationRuntime<Catalog, Program>;
pub fn install(calls: Arc<prefix::Calls>) -> Host {
    install_boxes(calls, Arc::new(boxes::Calls::default()))
}
pub fn install_boxes(prefix: Arc<prefix::Calls>, boxes: Arc<boxes::Calls>) -> Host {
    try_install(prefix, boxes, false)
        .expect("the actual public managed sources and both providers install")
}
pub fn try_install(
    prefix: Arc<prefix::Calls>,
    boxes: Arc<boxes::Calls>,
    incompatible: bool,
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    install_configuration(Configuration {
        prefix: PrefixExecutable::Current(prefix::Provider(prefix)),
        boxes,
        incompatible,
    })
}
pub fn try_install_prefix_version(
    prefix: Arc<prefix::Calls>,
    boxes: Arc<boxes::Calls>,
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    install_configuration(Configuration {
        prefix: PrefixExecutable::Incompatible(prefix::IncompatibleVersion(prefix::Provider(
            prefix,
        ))),
        boxes,
        incompatible: false,
    })
}
fn install_configuration(
    configuration: Configuration,
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Catalog, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (configuration,),
        ApplicationBuildIdentity::parse("19191919-1919-1919-1919-191919191919").unwrap(),
        resources::application_limits(),
        |_, graph, installed| {
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
                    .field(DirectoryIdentity::reference(), directory.to_owned()),
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
                    .field(ItemIdentity::reference(), identity.to_owned())
                    .field(ItemName::reference(), name.to_owned())
                    .field(ItemEnabled::reference(), enabled)
                    // Name-only rows still populate every required ItemFacts field.
                    .field(
                        boxes::ItemBounds::reference(),
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
        },
    )
}
fn directory_key(identity: &str) -> WorthQueryApplicationEntityKey<Catalog, Directory> {
    WorthQueryApplicationEntityKey::new(identity).unwrap()
}
