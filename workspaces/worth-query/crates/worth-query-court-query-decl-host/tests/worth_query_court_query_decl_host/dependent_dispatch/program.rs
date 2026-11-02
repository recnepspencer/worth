use super::*;
pub struct Contribution;
pub struct Configuration {
    pub selections: Selections,
    pub self_supersede: bool,
}
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("dispatch.contribution");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema::declare(schema)
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = Configuration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<parent::initial::Producer>()?;
        contracts.producer::<parent::preserve::Producer>()?;
        contracts.producer::<child::initial::Producer>()?;
        contracts.producer::<child::preserve::Producer>()?;
        contracts.producer::<leaf::initial::Producer>()?;
        contracts.producer::<leaf::preserve::Producer>()?;
        contracts.conditional::<parent::initial::Readiness>()?;
        contracts.conditional::<parent::preserve::Readiness>()?;
        contracts.conditional::<child::initial::Readiness>()?;
        contracts.conditional::<child::preserve::Readiness>()?;
        contracts.conditional::<leaf::initial::Readiness>()?;
        contracts.conditional::<leaf::preserve::Readiness>()?;
        split_producer::parent::contracts(contracts)?;
        split_producer::child::contracts(contracts)?;
        split_producer::leaf::contracts(contracts)
    }
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let Configuration {
            selections,
            self_supersede,
        } = configuration;
        setup.handler::<source_control::Binding, _>(source_control::Handler)?;
        parent::install(setup, selections.clone())?;
        child::install(setup, selections.clone())?;
        leaf::install(setup, selections.clone())?;
        split_producer::parent::install(setup, selections.clone(), self_supersede)?;
        split_producer::child::install(setup, selections.clone(), self_supersede)?;
        split_producer::leaf::install(setup, selections, self_supersede)
    }
}
macro_rules! feature {
    ($feature:ident, $inputs:ty, $name:literal) => {
        pub struct $feature;
        impl ApplicationFeature<Schema> for $feature {
            type Inputs = $inputs;
            const IDENTITY: &'static str = $name;
        }
    };
}
feature!(Entry, ApplicationFeatureInputLeaf, "dispatch.entry");
feature!(Parent, ApplicationFeatureInputList<ParentInput, ApplicationFeatureInputLeaf>, "dispatch.parent");
feature!(Child, ApplicationFeatureInputList<ChildInput, ApplicationFeatureInputList<ChildRootInput, ApplicationFeatureInputLeaf>>, "dispatch.child");
feature!(Leaf, ApplicationFeatureInputList<LeafInput, ApplicationFeatureInputList<LeafRootInput, ApplicationFeatureInputLeaf>>, "dispatch.leaf");
macro_rules! output_port {
    ($port:ident, $feature:ident, $name:literal) => {
        pub struct $port;
        impl ApplicationOutputPort<Schema, $feature> for $port {
            type Value = RowBinding;
            const IDENTITY: &'static str = $name;
        }
    };
}
macro_rules! input_port {
    ($port:ident, $feature:ident, $name:literal) => {
        pub struct $port;
        impl ApplicationInputPort<Schema, $feature> for $port {
            type Value = RowBinding;
            const IDENTITY: &'static str = $name;
            const REQUIRED: bool = false;
        }
    };
}
output_port!(EntryOutput, Entry, "entry-output");
output_port!(ParentOutput, Parent, "parent-output");
output_port!(ChildOutput, Child, "child-output");
input_port!(ParentInput, Parent, "parent-input");
input_port!(ChildInput, Child, "child-input");
input_port!(ChildRootInput, Child, "child-root-input");
input_port!(LeafInput, Leaf, "leaf-input");
input_port!(LeafRootInput, Leaf, "leaf-root-input");
macro_rules! connection {
    ($binding:ident, $alias:ident, $from:ident, $out:ident, $to:ident, $in:ident, $name:literal) => {
        pub struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $name;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, $from, $to> for $binding {}
        pub type $alias = ApplicationConnectionRef<Schema, $from, $out, $to, $in, $binding>;
    };
}
connection!(
    StartParent,
    ParentConnection,
    Entry,
    EntryOutput,
    Parent,
    ParentInput,
    "dispatch.start-parent"
);
connection!(
    StartChild,
    ChildRootConnection,
    Entry,
    EntryOutput,
    Child,
    ChildRootInput,
    "dispatch.start-child"
);
connection!(
    ParentChild,
    ChildConnection,
    Parent,
    ParentOutput,
    Child,
    ChildInput,
    "dispatch.parent-child"
);
connection!(
    ChildLeaf,
    LeafConnection,
    Child,
    ChildOutput,
    Leaf,
    LeafInput,
    "dispatch.child-leaf"
);
connection!(
    ChildRootLeaf,
    DirectLeafConnection,
    Child,
    ChildOutput,
    Leaf,
    LeafRootInput,
    "dispatch.child-root-leaf"
);
macro_rules! root_connection {
    ($binding:ident, $family:ident) => {
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for $binding {
            type Source = $family::initial::Binding;
            type Demand = $family::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn demand_from_source(
                input: &$family::initial::Input,
            ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok($family::Demand(input.0.key.clone()))
            }
        }
    };
}
root_connection!(StartParent, parent);
root_connection!(StartChild, child);
macro_rules! edge {
    ($binding:ident, $parent:ident, $child:ident) => {
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $binding {
            type RootDemand = $parent::Demand;
            type Discovery = Read;
            type Demand = $child::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_root(
                root: &Self::RootDemand,
            ) -> Result<Read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(Read(root.0.clone()))
            }
            fn demands_from_discovery(
                row: &Row,
            ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok((0..row.width)
                    .map(|index| $child::Demand(format!("{}.{}", row.key, index)))
                    .collect())
            }
        }
    };
}
edge!(ParentChild, parent, child);
edge!(ChildLeaf, child, leaf);
edge!(ChildRootLeaf, child, leaf);
pub type FullTree = ApplicationDemandOutputGraph<
    ParentConnection,
    ApplicationOutputEdge<
        ChildConnection,
        ApplicationOutputEdge<LeafConnection, ApplicationOutputLeaf>,
    >,
>;
pub type ChildTree = ApplicationDemandOutputGraph<
    ChildRootConnection,
    ApplicationOutputEdge<DirectLeafConnection, ApplicationOutputLeaf>,
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        (FullTree, ChildTree),
        (split_program::Tree, split_program::ChildTree),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("dispatch.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = vec![
            ApplicationFeatureSpec::root::<Schema, Entry>()
                .provides::<EntryOutput>()
                .native_action::<source_control::Binding>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Parent>()
                .provides::<ParentOutput>()
                .conditional_operation::<parent::initial::Operation>()
                .conditional_operation::<parent::preserve::Operation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Child>()
                .provides::<ChildOutput>()
                .conditional_operation::<child::initial::Operation>()
                .conditional_operation::<child::preserve::Operation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Leaf>()
                .conditional_operation::<leaf::initial::Operation>()
                .conditional_operation::<leaf::preserve::Operation>()
                .finish(),
        ];
        features.extend(split_program::features());
        features
    }
}
