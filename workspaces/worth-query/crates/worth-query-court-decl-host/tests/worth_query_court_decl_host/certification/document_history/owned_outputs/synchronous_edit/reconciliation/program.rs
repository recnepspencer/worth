//! Both public entries use the same installed discovery and two producer levels.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::program as base;
use super::*;
pub struct Root;
pub struct Input;
pub struct Output;
pub struct Child;
pub struct ChildInput;
pub struct ChildOutputPort;
impl ApplicationFeature<Schema> for Child {
    type Inputs = ApplicationFeatureInputList<ChildInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "reconciliation.child";
}
impl ApplicationInputPort<Schema, Child> for ChildInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "reconciliation.child-input";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Child> for ChildOutputPort {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "reconciliation.child-output";
}
impl ApplicationFeature<Schema> for Root {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "reconciliation.root";
}
impl ApplicationInputPort<Schema, Root> for Input {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "reconciliation.input";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Root> for Output {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "reconciliation.output";
}
pub struct RootLocality;
impl ApplicationLocalityScope for RootLocality {
    const IDENTITY: &'static str = "reconciliation.root-locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Schema, Root> for Artifact {
    type Output = Output;
    type Locality = RootLocality;
    const IDENTITY: &'static str = "reconciliation.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Reconstructive;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Replace;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = PROFILE;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <base::Entry as ApplicationFeature<Schema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "reconciliation.exact-source";
    const STOPPED_OUTCOME: &'static str = "reconciliation.refused";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Root>()
        .derived_artifact::<Artifact>()
        .required_pair::<Roles>()
        .finish()
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "reconciliation.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, Root> for Start {}
pub type Connection =
    ApplicationConnectionRef<Schema, base::Entry, base::EntryOutput, Root, Input, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "reconciliation.discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Root, Child> for Discover {}
pub type ChildConnection =
    ApplicationConnectionRef<Schema, Root, Output, Child, ChildInput, Discover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = Demand;
    type Discovery = discovery::Read;
    type Demand = child::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<discovery::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(discovery::Read(root.0.clone()))
    }
    fn demands_from_discovery(
        snapshot: &discovery::SourcesFound,
    ) -> Result<Vec<child::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(snapshot
            .sources
            .iter()
            .map(|source| child::Demand(source.key.clone()))
            .collect())
    }
}
type Edges = ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>;
pub type LazyTree = ApplicationDemandOutputGraph<Connection, Edges>;
pub type Tree = ApplicationOutputGraph<Connection, Edges>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "reconciliation.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct ChildArtifact;
impl ApplicationDerivedArtifact<Schema, Child> for ChildArtifact {
    type Output = ChildOutputPort;
    type Locality = RootLocality;
    const IDENTITY: &'static str = "reconciliation.child-artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Reconstructive;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Replace;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = "reconciled-child";
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            "reconciliation.artifact",
        )];
    const REUSE_RULE: &'static str = "reconciliation.child-exact-source";
    const STOPPED_OUTCOME: &'static str = "reconciliation.child-refused";
}
fn specs(entry: ApplicationFeatureSpec) -> Vec<ApplicationFeatureSpec> {
    let mut specs = hierarchy::Program::feature_specs();
    specs[0] = entry;
    specs
}
pub fn child_feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Child>()
        .derived_artifact::<ChildArtifact>()
        .required_pair::<child::Roles>()
        .finish()
}
pub struct Change;
impl ApplicationChangeShape for Change {
    const IDENTITY: &'static str = "reconciliation.count-change";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Replace;
}
fn entry<D: ApplicationActionDelivery<Schema, edit::Binding>>() -> ApplicationFeatureSpec {
    base::with_envelope_completion_actions(ApplicationFeatureSpec::root::<Schema, base::Entry>())
        .provides::<base::EntryOutput>()
        .action::<predecessor::synchronous::Binding, Completion>()
        .finish_action()
        .action::<super::super::new_source::Binding, Completion>()
        .finish_action()
        .action::<super::super::new_source::restricted::Binding, Completion>()
        .finish_action()
        .action::<super::super::Binding, Completion>()
        .finish_action()
        .action::<edit::Binding, D>()
        .locality_and_change::<RootLocality, Change>()
        .finish_action()
        .action::<cap_edit::Binding, Completion>()
        .finish_action()
        .action::<member_edit::Binding, Completion>()
        .finish_action()
        .action::<authored_member::retire::Binding, Completion>()
        .finish_action()
        .action::<authored_member::restore::Binding, Completion>()
        .finish_action()
        .action::<member_delete::Binding, Completion>()
        .finish_action()
        .finish()
}
pub struct Lazy;
impl ApplicationProgramDefinition<Schema> for Lazy {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<LazyTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("reconciliation.lazy");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        specs(entry::<Completion>())
    }
}
pub struct Synchronous;
impl ApplicationProgramDefinition<Schema> for Synchronous {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("reconciliation.synchronous");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        specs(entry::<Delivery>())
    }
}
