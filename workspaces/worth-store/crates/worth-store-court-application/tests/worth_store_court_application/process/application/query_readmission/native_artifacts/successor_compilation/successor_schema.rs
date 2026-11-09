//! A genuinely distinct declared B schema and program, never A with edited meaning.
use worth_query_host::facade::declaration::{
    application_program::*,
    application_schema::{
        fragment::{self, Fragment, NoBindings},
        *,
    },
};
pub(super) struct SuccessorSchema;
pub(super) struct Account;
pub(super) struct LaterRecord;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = ();
    type Relations = ();
}
impl fragment::Entity for LaterRecord {
    const NAME: &'static str = "LaterRecord";
    type Aspects = ();
    type Relations = ();
}
impl Fragment for SuccessorSchema {
    const NAME: &'static str = "SuccessorSchema";
    type Bindings = NoBindings;
    type Members = (SuccessorSurface,);
}
impl ApplicationSchema for SuccessorSchema {
    const OWNER: &'static str = "host_home_successor_test";
    const NAME: &'static str = "SuccessorSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for SuccessorSchema {
    type Contributions = (SuccessorSurface,);
}
struct NativeFeature;
impl ApplicationFeature<SuccessorSchema> for NativeFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.home-successor.native-feature.v1";
}
pub(super) struct SuccessorProgram;
impl ApplicationProgramDefinition<SuccessorSchema> for SuccessorProgram {
    type Contributions = (SuccessorSurface,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home-successor.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<SuccessorSchema, NativeFeature>().finish()]
    }
}

// Real root-owned contribution membership and its matching entry configuration.
pub(super) struct SuccessorSurface;
impl ApplicationSchemaContribution<SuccessorSchema> for SuccessorSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.home-successor.surface.v1");
    type Members = (Account, LaterRecord);
}
impl
    worth_query_host::facade::application_contribution::WorthQueryApplicationContribution<
        SuccessorSchema,
    > for SuccessorSurface
{
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, SuccessorSchema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        Ok(())
    }
}
