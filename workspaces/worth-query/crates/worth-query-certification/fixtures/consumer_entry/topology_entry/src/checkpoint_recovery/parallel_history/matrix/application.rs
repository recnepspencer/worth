//! The neutral matrix's single schema and installed producer/computation.
use super::{computation, control::Control, edit, facts, operation, producer, readiness, source};
use crate::principal::*;
use std::sync::Arc;
use worth_query_decl::facade::{
    application_program::*, application_schema::*, worth_query_application,
    worth_query_application_contribution, worth_query_aspect, worth_query_entity,
    worth_query_field,
};
use worth_query_host::facade::{
    application_contribution::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub trait SchemaBinding: crate::TopologySchemaBinding {}
worth_query_entity!(pub Node for Schema: SchemaBinding);
worth_query_aspect!(pub NodeFacts for Schema: SchemaBinding, Node; identity=AspectIdentity(0x9176_78b1), revision=AspectContractRevision(1),);
worth_query_field!(pub NodeKey for Schema: SchemaBinding, Node, NodeFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Revision for Schema: SchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Value for Schema: SchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Applied for Schema: SchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_application_contribution! { pub contribution Contribution for Schema: SchemaBinding {
identity:"worth.certification.required-matrix.v1", members:|schema| {
edit::declare(operation::declare(facts::declare(source::declare(schema))))
.entity(Node::reference::<Schema>()).aspect(Node::reference(),NodeFacts::reference())
.field(Node::reference(),NodeKey::reference()).field(Node::reference(),Revision::reference())
.field(Node::reference(),Value::reference()).field(Node::reference(),Applied::reference())
.entity(ExternalPrincipalMapping::reference::<Schema>()).entity(Principal::reference::<Schema>())
.aspect(ExternalPrincipalMapping::reference(),ExternalIdentity::reference()).aspect(Principal::reference(),PrincipalFacts::reference())
.field(ExternalPrincipalMapping::reference(),ExternalIdentityField::reference()).field(ExternalPrincipalMapping::reference(),MappingStatusField::reference())
.field(Principal::reference(),PrincipalIdentity::reference())
.relation(MappingTarget::reference(),ExternalPrincipalMapping::reference(),Principal::reference())
.principal_binding(ConsumerPrincipalBinding::reference()) } } }
worth_query_application! { pub Schema {owner:"worth.certification.required-matrix",version:(1,0),contributions:[Contribution],} }
impl crate::TopologySchemaBinding for Schema {}
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
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(Some(8192), Some(16384));
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
