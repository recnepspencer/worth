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
pub type Tree = ApplicationOutputGraph<
    Root,
    (
        ApplicationOutputOccurrences<All>,
        ApplicationOutputEdge<StaticConnection, ApplicationOutputLeaf>,
    ),
>;
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
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for fixture::after_producer::Binding {
    fn demand_from_source(
        _: &fixture::after_producer::Intent,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            source: "a".into(),
            selector: Some("a".into()),
        })
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for fixture::member::Binding {
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
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.complete-component");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = entry::envelope_actions::<
            fixture::Delivery<Tree>,
            fixture::Delivery<Tree>,
            Completion,
            Completion,
            Completion,
            Completion,
            fixture::Delivery<Tree>,
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
