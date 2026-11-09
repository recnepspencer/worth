//! The neutral matrix's single schema and installed producer/computation.
#[path = "application/records.rs"]
mod storage_model;
use super::{computation, control::Control, edit, facts, operation, producer, readiness, source};
use crate::principal::*;
use std::sync::Arc;
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
            "worth.certification.required-matrix.v1",
        );
    type Members = (
        HistoryRecords,
        crate::MinimalPrincipalRecords,
        crate::PrincipalMappingRecords,
        ContributionContracts,
    );
}
impl Contribution {
    pub fn reference<Schema: SchemaBinding>(
    ) -> worth_query_decl::facade::application_schema::ApplicationSchemaContributionRef<Schema, Self>
    {
        worth_query_decl::facade::application_schema::ApplicationSchemaContributionRef::from_contribution()
    }
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
        Ok(
            edit::declare(operation::declare(facts::declare(source::declare(schema))))
                .principal_binding(ConsumerPrincipalBinding::reference()),
        )
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
    const OWNER: &'static str = "worth.certification.required-matrix";
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
    type Configuration = Arc<Control>;
    fn contracts(
        c: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        c.producer::<producer::Producer>()?;
        c.conditional::<readiness::Readiness>()?;
        Ok(())
    }
    fn configure(
        control: Arc<Control>,
        s: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let installed = s.partitioned_computation::<Feature, computation::Computation, _>(
            computation::Owner(control.clone()),
        )?;
        s.handler::<operation::Binding<Schema>, _>(operation::Handler {
            installed,
            control: control.clone(),
        })?;
        s.handler::<edit::Binding<Schema>, _>(edit::Handler)?;
        s.producer::<producer::Producer>(producer::Provider(control))?;
        s.conditional::<readiness::Readiness>(())
    }
}
pub struct SourceFeature;
pub struct Feature;
pub struct SourcePort;
pub struct InputPort;
pub struct OutputPort;
pub struct Connection;
impl ApplicationFeature<Schema> for SourceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "matrix-source";
}
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputList<InputPort, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "matrix-output";
}
impl ApplicationOutputPort<Schema, SourceFeature> for SourcePort {
    type Value = source::ResultBinding;
    const IDENTITY: &'static str = "source";
}
impl ApplicationInputPort<Schema, Feature> for InputPort {
    type Value = source::ResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<Schema, Feature> for OutputPort {
    type Value = facts::ResultBinding;
    const IDENTITY: &'static str = "value";
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "matrix-source-output";
}
impl ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, Feature> for Connection {}
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<Schema, SourceFeature, SourcePort, Feature, InputPort, Connection>,
    ApplicationOutputLeaf,
>;
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "matrix-node";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
struct SourceChange;
impl ApplicationChangeShape for SourceChange {
    const IDENTITY: &'static str = "matrix-source-reconstruction";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Reconstruct;
}
impl ApplicationDerivedArtifact<Schema, Feature> for Artifact {
    type Output = OutputPort;
    type Locality = Locality;
    const IDENTITY: &'static str = "matrix-value";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Recompute;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "matrix-producer-family";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <SourceFeature as ApplicationFeature<Schema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "matrix-exact";

    const STOPPED_OUTCOME: &'static str = "matrix-stop";
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("required-neutral-matrix");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, SourceFeature>()
                .provides::<SourcePort>()
                .native_action::<edit::Binding<Schema>>()
                .locality_and_change::<Locality, SourceChange>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Feature>()
                .derived_artifact::<Artifact>()
                .managed_computation::<computation::Computation>()
                .conditional_operation::<operation::Apply>()
                .finish(),
        ]
    }
}
