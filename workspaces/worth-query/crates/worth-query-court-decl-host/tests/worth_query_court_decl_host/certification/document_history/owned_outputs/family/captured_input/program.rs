//! Two complete captured parent ports share the existing First producer feature.
use super::super::super::synchronous_edit::{envelope_fixture as fixture, interleaved};
use super::super::super::{hierarchy::program as hierarchy, program as entry};
use super::*;
use interleaved::component_carriage::outgoing;

pub struct Feature;
pub struct RootInput;
pub struct SeedInput;
pub struct AnchorInput;
pub struct Output;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputList<
        RootInput,
        ApplicationFeatureInputList<
            SeedInput,
            ApplicationFeatureInputList<AnchorInput, ApplicationFeatureInputLeaf>,
        >,
    >;
    const IDENTITY: &'static str = "owned-outputs.captured-input.feature";
}
macro_rules! input {
    ($name:ident, $identity:literal) => {
        impl ApplicationInputPort<Schema, Feature> for $name {
            type Value = query::RowBinding;
            const IDENTITY: &'static str = $identity;
            const REQUIRED: bool = false;
        }
    };
}
input!(RootInput, "captured-input.root");
input!(SeedInput, "captured-input.seed");
input!(AnchorInput, "captured-input.anchor");
impl ApplicationOutputPort<Schema, Feature> for Output {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "captured-input.output";
}
pub struct Locality;
impl ApplicationLocalityScope for Locality {
    const IDENTITY: &'static str = "captured-input.locality";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
pub struct Artifact;
impl ApplicationDerivedArtifact<Schema, Feature> for Artifact {
    type Output = Output;
    type Locality = Locality;
    const IDENTITY: &'static str = "captured-input.artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Reconstructive;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Replace;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str = PROFILE;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <entry::Entry as ApplicationFeature<Schema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "captured-input.exact-data";
    const STOPPED_OUTCOME: &'static str = "captured-input.refused";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Feature>()
        .derived_artifact::<Artifact>()
        .required_pair::<Roles>()
        .finish()
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "captured-input.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, Feature> for Start {}
pub type Root =
    ApplicationConnectionRef<Schema, entry::Entry, entry::EntryOutput, Feature, RootInput, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = fixture::model::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for fixture::model::Binding {
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for fixture::member::Binding {
    fn demand_from_source(
        _: &fixture::member::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand("capture:child".into()))
    }
}
macro_rules! parent {
    ($binding:ident, $connection:ident, $input:ident, $identity:literal) => {
        pub struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, Feature>
            for $binding
        {
        }
        pub type $connection = ApplicationConnectionRef<
            Schema,
            hierarchy::Parent,
            hierarchy::Output,
            Feature,
            $input,
            $binding,
        >;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $binding {
            type RootDemand = first::Demand;
            type Demand = Demand;
            type Discovery = outgoing::Read;
            const IDENTITY: &'static str = $identity;
            fn discovery_from_root(
                root: &first::Demand,
            ) -> Result<outgoing::Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(outgoing::Read(root.0.clone()))
            }
            fn demands_from_discovery(
                row: &outgoing::Dependents,
            ) -> Result<Vec<Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(row.children.iter().cloned().map(Demand).collect())
            }
        }
    };
}
parent!(
    Seed,
    SeedConnection,
    SeedInput,
    "captured-input.seed-parent"
);
parent!(
    Anchor,
    AnchorConnection,
    AnchorInput,
    "captured-input.anchor-parent"
);
pub struct Component;
impl ApplicationOccurrenceComponent<Schema> for Component {
    type Connections = (
        ApplicationCapturedInputConnection<SeedConnection, Capture>,
        ApplicationCapturedInputConnection<AnchorConnection, Capture>,
    );
}
pub type Tree = ApplicationOutputGraph<Root, ApplicationOutputOccurrences<Component>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        feature_specs::<fixture::Delivery<Tree>>()
    }
}
pub(super) fn feature_specs<Delivery>() -> Vec<ApplicationFeatureSpec>
where
    Delivery: ApplicationActionDelivery<Schema, fixture::model::Binding>
        + ApplicationActionDelivery<Schema, fixture::member::Binding>,
{
    let mut features = hierarchy::Program::feature_specs();
    features[0] = entry::envelope_actions::<
        Delivery,
        Delivery,
        Completion,
        Completion,
        Completion,
        Completion,
        Completion,
    >(entry::entry_builder::<Completion, Completion, Completion>())
    .action::<interleaved::created::Binding, Completion>()
    .finish_action()
    .action::<member_retarget::operation::Binding, Completion>()
    .finish_action()
    .finish();
    features.push(feature());
    features
}

pub(in super::super::super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new("capture:child").unwrap(),
        )
        .field(ModelKey::at("ModelKey"), "capture:child".to_owned())
        .field(ModelValue::at("ModelValue"), 3),
    )?;
    for (key, parent, weight) in [
        ("capture:seed-a", "a", 1),
        ("capture:seed-late", "carriage:late", 1),
        ("capture:anchor-d", "carriage:d", 2),
    ] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Member::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(MemberKey::at("MemberKey"), key.to_owned())
            .field(MemberValue::at("MemberValue"), weight),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:{key}"),
            WorthQueryApplicationEntityKey::new("capture:child").unwrap(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            UpstreamOf::reference(),
            format!("uses:{key}"),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
            WorthQueryApplicationEntityKey::new(parent).unwrap(),
        ))?;
    }
    Ok(())
}
