//! Neutral schema, declarations and installed contracts; no reference arithmetic.
#[path = "application/records.rs"]
mod storage_model;
use super::{computation, operation, read};
use crate::principal::*;
pub use storage_model::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    application_contribution::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

pub trait SchemaBinding: crate::TopologySchemaBinding {}

pub struct Contribution;
impl<Schema: SchemaBinding>
    worth_query_decl::facade::application_schema::ApplicationSchemaContribution<Schema>
    for Contribution
{
    const IDENTITY:
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity =
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionIdentity::new(
            "worth.certification.neutral-history.v1",
        );
    type Members = (
        HistoryRecords,
        crate::MinimalPrincipalRecords,
        crate::PrincipalMappingRecords,
        ContributionContracts,
    );
}
pub struct ContributionContracts;
impl<Schema: SchemaBinding>
    worth_query_decl::facade::application_schema::fragment::Registration<Schema>
    for ContributionContracts
{
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
            Schema,
        >,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(operation::declare(read::declare(schema))
            .principal_binding(ConsumerPrincipalBinding::reference()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Schema;
impl worth_query_decl::facade::application_schema::fragment::Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (Contribution,);
}
impl worth_query_decl::facade::application_schema::ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.certification.neutral-history";
    const NAME: &'static str = "Schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for Schema {
    type Contributions = (Contribution,);
}

impl crate::TopologySchemaBinding for Schema {
    type PrincipalFragment = crate::PrincipalRecords;
}
impl SchemaBinding for Schema {}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn contracts(
        _: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
    }
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let installed = setup
            .partitioned_computation::<Feature, computation::Computation, _>(computation::Owner)?;
        setup.handler::<operation::Binding<Schema>, _>(operation::Handler(installed))?;
        Ok(())
    }
}
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "neutral-history";
}
pub struct Port;
impl ApplicationOutputPort<Schema, Feature> for Port {
    type Value = operation::ResultBinding;
    const IDENTITY: &'static str = "value";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "member";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Schema, Feature> for Artifact {
    type Output = Port;
    type Locality = Locality;
    const IDENTITY: &'static str = "neutral-value";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Disposable;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = false;
    const PRODUCER_FAMILY: &'static str = "neutral-value";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] = &[];
    const REUSE_RULE: &'static str = "exact-input";

    const STOPPED_OUTCOME: &'static str = "refused";
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new("neutral-history");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .derived_artifact::<Artifact>()
            .managed_computation::<computation::Computation>()
            .native_action::<operation::Binding<Schema>>()
            .finish_action()
            .finish()]
    }
}
