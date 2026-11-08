//! Both installed connections compute the same child route from their actual output.
use super::*;
use hierarchy::ChildOutput as ChildOutputPort;
pub use joined::{Join, LeftInput, RightInput};
macro_rules! connection {
    ($name:ident, $alias:ident, $from:ty, $output:ty, $to:ty, $input:ty, $root:ty, $read:ty, $demand:ty, $identity:literal, $intent:expr, $children:expr) => {
        pub struct $name;
        impl ApplicationConnectionIdentity for $name {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, $from, $to> for $name {}
        pub type $alias = ApplicationConnectionRef<Schema, $from, $output, $to, $input, $name>;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $name {
            type RootDemand = $root;
            type Discovery = $read;
            type Demand = $demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_root(
                root: &$root,
            ) -> Result<$read, WorthQueryRequiredOutputConnectionDenial> {
                Ok(($intent)(root))
            }
            fn demands_from_discovery(
                row: &query::Row,
            ) -> Result<Vec<$demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(($children)(row))
            }
        }
    };
}
connection!(
    Left,
    LeftConnection,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    first::Demand,
    query::first_output::Read,
    from_first::Demand,
    "owned-outputs.diamond.left",
    |d: &first::Demand| query::first_output::Read(format!("first:{}", d.0)),
    |row: &query::Row| vec![from_first::Demand(row.key.clone())]
);
connection!(
    Right,
    RightConnection,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::SeedInput,
    first::Demand,
    query::first_output::Read,
    second::Demand,
    "owned-outputs.diamond.right",
    |d: &first::Demand| query::first_output::Read(format!("first:{}", d.0)),
    |row: &query::Row| vec![second::Demand(
        row.key.strip_prefix("first:").unwrap().into()
    )]
);
connection!(
    LeftJoin,
    LeftJoinConnection,
    hierarchy::Child,
    ChildOutputPort,
    Join,
    LeftInput,
    from_first::Demand,
    query::child_output::Read,
    joined::Demand,
    "owned-outputs.diamond.left-join",
    |d: &from_first::Demand| query::child_output::Read(format!("from-first:{}", d.0)),
    |row: &query::Row| vec![joined::Demand(
        row.key.strip_prefix("from-first:").unwrap().into()
    )]
);
connection!(
    RightJoin,
    RightJoinConnection,
    hierarchy::Child,
    ChildOutputPort,
    Join,
    RightInput,
    second::Demand,
    query::second_output::Read,
    joined::Demand,
    "owned-outputs.diamond.right-join",
    |d: &second::Demand| query::second_output::Read(format!("second:{}", d.0)),
    |row: &query::Row| vec![joined::Demand(format!(
        "first:{}",
        row.key.strip_prefix("second:").unwrap()
    ))]
);

pub struct Extra;
pub struct ExtraInput;
impl ApplicationFeature<Schema> for Extra {
    type Inputs = ApplicationFeatureInputList<ExtraInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "owned-outputs.diamond.extra";
}
impl ApplicationInputPort<Schema, Extra> for ExtraInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "diamond-extra-input";
    const REQUIRED: bool = false;
}
connection!(
    ExtraChildren,
    ExtraConnection,
    hierarchy::Child,
    ChildOutputPort,
    Extra,
    ExtraInput,
    from_first::Demand,
    query::joined::Read,
    upstream::Demand,
    "owned-outputs.diamond.extra-children",
    |d: &from_first::Demand| query::joined::Read(d.0.clone()),
    |row: &query::Row| if row.value % 1000 != 0 {
        vec![
            upstream::Demand(row.key.strip_prefix("first:").unwrap().into()),
            upstream::Demand("b".into()),
        ]
    } else {
        Vec::new()
    }
);
pub type LeftChildren = (
    ApplicationOutputEdge<LeftJoinConnection, ApplicationOutputLeaf>,
    ApplicationOutputEdge<ExtraConnection, ApplicationOutputLeaf>,
);
pub type Tree = ApplicationOutputGraph<
    super::super::program::Connection,
    (
        ApplicationOutputEdge<LeftConnection, LeftChildren>,
        ApplicationOutputEdge<
            RightConnection,
            ApplicationOutputEdge<RightJoinConnection, ApplicationOutputLeaf>,
        >,
    ),
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.diamond.delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.diamond.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();

        specs.push(ApplicationFeatureSpec::root::<Schema, Extra>().finish());
        specs
    }
}

pub mod unequal;
