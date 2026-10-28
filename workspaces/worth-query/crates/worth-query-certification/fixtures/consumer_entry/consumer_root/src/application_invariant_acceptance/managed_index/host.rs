//! Genuine application contribution, authenticated principal and committed source.
use super::super::{authentication, resources};
use super::{prefix, schema::*};
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
        vec![ApplicationFeatureSpec::root::<Catalog, Feature>().finish()]
    }
}
impl WorthQueryApplicationContribution<Catalog> for CatalogContribution {
    type Configuration = Arc<prefix::Calls>;
    fn configure(
        calls: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.managed_index::<prefix::PrefixIndex, _>(prefix::Provider(calls))
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Catalog, Program>;
pub fn install(calls: Arc<prefix::Calls>) -> Host {
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Catalog, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (calls,),
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
            for directory in ["catalog", "empty"] {
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
                    .field(ItemEnabled::reference(), enabled),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Contains::reference(),
                    identity,
                    directory_key("catalog"),
                    key,
                ))?;
            }
            Ok(())
        },
    )
    .expect("the actual public managed source and provider install")
}
fn directory_key(identity: &str) -> WorthQueryApplicationEntityKey<Catalog, Directory> {
    WorthQueryApplicationEntityKey::new(identity).unwrap()
}
