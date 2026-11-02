use super::*;
pub struct Entry;
pub struct Parent;
impl ApplicationFeature<Schema> for Entry {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "source-subgraph.entry.v1";
}
impl ApplicationFeature<Schema> for Parent {
    type Inputs = ApplicationFeatureInputList<ParentInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "source-subgraph.parent.v1";
}
pub struct EntryOutput;
impl ApplicationOutputPort<Schema, Entry> for EntryOutput {
    type Value = graph_read::RowBinding;
    const IDENTITY: &'static str = "source-subgraph.entry-output.v1";
}
pub struct ParentInput;
impl ApplicationInputPort<Schema, Parent> for ParentInput {
    type Value = graph_read::RowBinding;
    const IDENTITY: &'static str = "source-subgraph.parent-input.v1";
    const REQUIRED: bool = false;
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "source-subgraph.start.v1";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Parent> for Start {}
pub type Connection =
    ApplicationConnectionRef<Schema, Entry, EntryOutput, Parent, ParentInput, Start>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = generation::Binding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &generation::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand)
    }
}
pub type Graph = ApplicationDemandOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Graph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("source-subgraph.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, Entry>()
                .provides::<EntryOutput>()
                .native_action::<edit::source::Binding>()
                .finish_action()
                .native_action::<edit::generated::Binding>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Parent>()
                .conditional_operation::<generation::Generate>()
                .finish(),
        ]
    }
}
