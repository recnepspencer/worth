use super::producers::{child, independent_parent, leaf, parent};
use super::*;
macro_rules! feature {
    ($feature:ident, $inputs:ty, $name:literal) => {
        pub struct $feature;
        impl ApplicationFeature<Schema> for $feature {
            type Inputs = $inputs;
            const IDENTITY: &'static str = $name;
        }
    };
}
feature!(
    SplitEntry,
    ApplicationFeatureInputLeaf,
    "dispatch.split.entry"
);
feature!(SplitParent, ApplicationFeatureInputList<SplitParentInput, ApplicationFeatureInputList<IndependentParentInput, ApplicationFeatureInputLeaf>>, "dispatch.split.parent");
feature!(SplitChild, ApplicationFeatureInputList<SplitChildInput, ApplicationFeatureInputList<SplitChildRootInput, ApplicationFeatureInputList<IndependentChildInput, ApplicationFeatureInputLeaf>>>, "dispatch.split.child");
feature!(SplitLeaf, ApplicationFeatureInputList<SplitLeafInput, ApplicationFeatureInputList<SplitLeafRootInput, ApplicationFeatureInputList<IndependentLeafInput, ApplicationFeatureInputLeaf>>>, "dispatch.split.leaf");
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
output_port!(SplitEntryOutput, SplitEntry, "entry-output");
output_port!(SplitParentOutput, SplitParent, "parent-output");
output_port!(SplitChildOutput, SplitChild, "child-output");
input_port!(SplitParentInput, SplitParent, "parent-input");
input_port!(
    IndependentParentInput,
    SplitParent,
    "independent-parent-input"
);
input_port!(IndependentChildInput, SplitChild, "independent-child-input");
input_port!(IndependentLeafInput, SplitLeaf, "independent-leaf-input");
input_port!(SplitChildInput, SplitChild, "child-input");
input_port!(SplitChildRootInput, SplitChild, "child-root-input");
input_port!(SplitLeafInput, SplitLeaf, "leaf-input");
input_port!(SplitLeafRootInput, SplitLeaf, "leaf-root-input");
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
    StartSplitParent,
    SplitParentConnection,
    SplitEntry,
    SplitEntryOutput,
    SplitParent,
    SplitParentInput,
    "dispatch.split.start-parent"
);
connection!(
    StartSplitChild,
    SplitChildRootConnection,
    SplitEntry,
    SplitEntryOutput,
    SplitChild,
    SplitChildRootInput,
    "dispatch.split.start-child"
);
connection!(
    SplitParentSplitChild,
    SplitChildConnection,
    SplitParent,
    SplitParentOutput,
    SplitChild,
    SplitChildInput,
    "dispatch.split.parent-child"
);
connection!(
    SplitChildSplitLeaf,
    SplitLeafConnection,
    SplitChild,
    SplitChildOutput,
    SplitLeaf,
    SplitLeafInput,
    "dispatch.split.child-leaf"
);
connection!(
    SplitChildRootSplitLeaf,
    DirectSplitLeafConnection,
    SplitChild,
    SplitChildOutput,
    SplitLeaf,
    SplitLeafRootInput,
    "dispatch.split.child-root-leaf"
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
root_connection!(StartSplitParent, parent);
root_connection!(StartSplitChild, child);
connection!(
    StartIndependentParent,
    IndependentParentConnection,
    SplitEntry,
    SplitEntryOutput,
    SplitParent,
    IndependentParentInput,
    "dispatch.independent.start-parent"
);
root_connection!(StartIndependentParent, independent_parent);
connection!(
    IndependentChildSplitLeaf,
    IndependentLeafConnection,
    SplitChild,
    SplitChildOutput,
    SplitLeaf,
    IndependentLeafInput,
    "dispatch.independent.child-leaf"
);
connection!(
    IndependentParentSplitChild,
    IndependentChildConnection,
    SplitParent,
    SplitParentOutput,
    SplitChild,
    IndependentChildInput,
    "dispatch.independent.parent-child"
);
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
edge!(SplitParentSplitChild, parent, child);
edge!(SplitChildSplitLeaf, child, leaf);
edge!(SplitChildRootSplitLeaf, child, leaf);
edge!(IndependentParentSplitChild, independent_parent, child);
edge!(IndependentChildSplitLeaf, child, leaf);
pub type IndependentTree = ApplicationDemandOutputGraph<
    IndependentParentConnection,
    ApplicationOutputEdge<
        IndependentChildConnection,
        ApplicationOutputEdge<IndependentLeafConnection, ApplicationOutputLeaf>,
    >,
>;
pub type Tree = ApplicationDemandOutputGraph<
    SplitParentConnection,
    ApplicationOutputEdge<
        SplitChildConnection,
        ApplicationOutputEdge<SplitLeafConnection, ApplicationOutputLeaf>,
    >,
>;
pub type ChildTree = ApplicationDemandOutputGraph<
    SplitChildRootConnection,
    ApplicationOutputEdge<DirectSplitLeafConnection, ApplicationOutputLeaf>,
>;

pub fn features() -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<Schema, SplitEntry>()
            .provides::<SplitEntryOutput>()
            .native_action::<source_control::Binding>()
            .finish_action()
            .finish(),
        ApplicationFeatureSpec::root::<Schema, SplitParent>()
            .provides::<SplitParentOutput>()
            .conditional_operation::<parent::initial::Operation>()
            .conditional_operation::<parent::preserve::Operation>()
            .conditional_operation::<independent_parent::initial::Operation>()
            .conditional_operation::<independent_parent::preserve::Operation>()
            .finish(),
        ApplicationFeatureSpec::root::<Schema, SplitChild>()
            .provides::<SplitChildOutput>()
            .conditional_operation::<child::initial::Operation>()
            .conditional_operation::<child::preserve::Operation>()
            .finish(),
        ApplicationFeatureSpec::root::<Schema, SplitLeaf>()
            .conditional_operation::<leaf::initial::Operation>()
            .conditional_operation::<leaf::preserve::Operation>()
            .finish(),
    ]
}

pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, (ChildTree, IndependentTree))>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("succession.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features()
    }
}
