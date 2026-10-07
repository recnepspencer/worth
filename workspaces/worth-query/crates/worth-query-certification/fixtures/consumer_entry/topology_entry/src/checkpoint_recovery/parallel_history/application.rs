//! Neutral schema, declarations and installed contracts; no reference arithmetic.
use super::{computation, operation, read};
use crate::principal::*;
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
worth_query_aspect!(pub NodeFacts for Schema: SchemaBinding, Node;
    identity = AspectIdentity(0x9176_7801), revision = AspectContractRevision(1),);
worth_query_field!(pub NodeKey for Schema: SchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub Value for Schema: SchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub Applied for Schema: SchemaBinding, Node, NodeFacts: u64 => U64ApplicationValueBinding, read_write, equality);

worth_query_application_contribution! {
    pub contribution Contribution for Schema: SchemaBinding {
        identity: "worth.certification.neutral-history.v1",
        members: |schema| {
            operation::declare(read::declare(schema))
                .entity(Node::reference::<Schema>())
                .aspect(Node::reference(),NodeFacts::reference())
                .field(Node::reference(),NodeKey::reference())
                .field(Node::reference(),Value::reference())
                .field(Node::reference(),Applied::reference())
                .entity(ExternalPrincipalMapping::reference::<Schema>())
                .entity(Principal::reference::<Schema>())
                .aspect(ExternalPrincipalMapping::reference(),ExternalIdentity::reference())
                .aspect(Principal::reference(),PrincipalFacts::reference())
                .field(ExternalPrincipalMapping::reference(),ExternalIdentityField::reference())
                .field(ExternalPrincipalMapping::reference(),MappingStatusField::reference())
                .field(Principal::reference(),PrincipalIdentity::reference())
                .relation(MappingTarget::reference(),ExternalPrincipalMapping::reference(),Principal::reference())
                .principal_binding(ConsumerPrincipalBinding::reference())
        }
    }
}
worth_query_application! { pub Schema {owner:"worth.certification.neutral-history", version:(1,0), contributions:[Contribution],} }
impl crate::TopologySchemaBinding for Schema {}
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
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(8192, 16384);
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
            .mutation::<operation::Binding<Schema>>()
            .finish()]
    }
}
