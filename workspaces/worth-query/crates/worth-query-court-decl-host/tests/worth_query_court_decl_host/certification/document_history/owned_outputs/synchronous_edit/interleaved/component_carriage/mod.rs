//! One complete component plus demand-dependent static continuations.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::program as entry;
mod topology;
use super::*;
mod connections;
use connections::*;
#[cfg(test)]
mod courts;

#[derive(Clone)]
pub struct Demand {
    source: String,
    selector: Option<String>,
}
impl WorthQueryApplicationOutputDemand<Schema> for Demand {
    type OutputFamily = first::Family;
    fn source_intent(&self) -> query::producer_model::Read {
        self.source.clone().into()
    }
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "component-carriage.start";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, hierarchy::Parent> for Start {}
type Root = ApplicationConnectionRef<
    Schema,
    entry::Entry,
    entry::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Start,
>;
pub type All = ApplicationOccurrenceGroup<(PreviousConnection,), (ForeignConnection,)>;
type Edges = (
    ApplicationOutputOccurrences<All>,
    ApplicationOutputEdge<StaticConnection, ApplicationOutputLeaf>,
);
pub type Tree = ApplicationOutputGraph<Root, Edges>;
pub struct After;
impl ApplicationConnectionIdentity for After {
    const IDENTITY: &'static str = "component-carriage.after";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, hierarchy::Parent> for After {}
type AfterConnection = ApplicationConnectionRef<
    Schema,
    entry::Entry,
    entry::EntryOutput,
    hierarchy::Parent,
    hierarchy::ContinuationInput,
    After,
>;
pub type AfterTree = ApplicationOutputGraph<AfterConnection, Edges>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for After {
    type Source = fixture::after_producer::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &fixture::after_producer::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        <fixture::after_producer::Binding as WorthQueryApplicationRequiredOutputSource<
            Schema,
            Self,
        >>::demand_from_source(input)
    }
}
pub struct MemberStart;
impl ApplicationConnectionIdentity for MemberStart {
    const IDENTITY: &'static str = "component-carriage.member";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, hierarchy::Parent>
    for MemberStart
{
}
type MemberConnection = ApplicationConnectionRef<
    Schema,
    entry::Entry,
    entry::EntryOutput,
    hierarchy::Parent,
    hierarchy::FirstSeedInput,
    MemberStart,
>;
pub type MemberTree = ApplicationOutputGraph<MemberConnection, Edges>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for MemberStart {
    type Source = fixture::member::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &fixture::member::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        <fixture::member::Binding as WorthQueryApplicationRequiredOutputSource<Schema, Self>>::demand_from_source(input)
    }
}
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = fixture::model::Binding;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: input.key.clone(),
            selector: None,
        })
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for fixture::model::Binding {
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Start::demand_from_source(input)
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, After> for fixture::after_producer::Binding {
    fn demand_from_source(
        _: &fixture::after_producer::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: "a".into(),
            selector: Some("a".into()),
        })
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, MemberStart> for fixture::member::Binding {
    fn demand_from_source(
        _: &fixture::member::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: "a".into(),
            selector: Some("carriage:d".into()),
        })
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, (AfterTree, MemberTree))>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.complete-component");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = entry::envelope_actions::<
            fixture::Delivery<Tree>,
            fixture::Delivery<MemberTree>,
            Completion,
            Completion,
            Completion,
            Completion,
            fixture::Delivery<AfterTree>,
        >(entry::entry_builder::<Completion, Completion, Completion>())
        .finish();
        specs
    }
}
struct Fragmented;
impl ApplicationProgramDefinition<Schema> for Fragmented {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<
        ApplicationOutputGraph<
            Root,
            (
                ApplicationOutputOccurrences<(PreviousConnection,)>,
                ApplicationOutputOccurrences<(ForeignConnection,)>,
            ),
        >,
    >;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.fragmented-component");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Program::feature_specs()
    }
}
struct DuplicateTail;
impl ApplicationProgramDefinition<Schema> for DuplicateTail {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<
        ApplicationOutputGraph<
            Root,
            (
                Edges,
                ApplicationOutputEdge<StaticConnection, ApplicationOutputLeaf>,
            ),
        >,
    >;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.duplicate-tail");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        Program::feature_specs()
    }
}
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    for (key, value) in [("carriage:b", 2), ("carriage:d", 4)] {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(ModelKey::at("ModelKey"), key.to_owned())
            .field(ModelValue::at("ModelValue"), value),
        )?;
    }
    for (role, parent) in [("previous", "a"), ("foreign", "carriage:d")] {
        let descriptor = format!("carriage:b:{role}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Member::reference(),
                WorthQueryApplicationEntityKey::new(&descriptor).unwrap(),
            )
            .field(MemberKey::at("MemberKey"), descriptor.clone())
            .field(MemberValue::at("MemberValue"), 0),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ModelMember::reference(),
            format!("owns:{descriptor}"),
            WorthQueryApplicationEntityKey::new("carriage:b").unwrap(),
            WorthQueryApplicationEntityKey::new(&descriptor).unwrap(),
        ))?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            UpstreamOf::reference(),
            format!("uses:{descriptor}"),
            WorthQueryApplicationEntityKey::new(&descriptor).unwrap(),
            WorthQueryApplicationEntityKey::new(parent).unwrap(),
        ))?;
    }
    Ok(())
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    topology::declare(schema)
}
