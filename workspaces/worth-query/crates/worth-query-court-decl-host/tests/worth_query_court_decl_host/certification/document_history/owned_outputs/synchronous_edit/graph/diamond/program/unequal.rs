//! The right route is discovered through a longer declared occurrence path.
use super::*;
pub struct Middle;
pub struct MiddleInput;
pub struct MiddleOutput;
impl ApplicationFeature<Schema> for Middle {
    type Inputs = ApplicationFeatureInputList<MiddleInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "owned-outputs.diamond.middle";
}
impl ApplicationInputPort<Schema, Middle> for MiddleInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "diamond-middle-input";
    const REQUIRED: bool = false;
}
impl ApplicationOutputPort<Schema, Middle> for MiddleOutput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "diamond-middle-output";
}
connection!(
    StartMiddle,
    MiddleConnection,
    hierarchy::Parent,
    hierarchy::Output,
    Middle,
    MiddleInput,
    first::Demand,
    query::first_output::Read,
    first::Demand,
    "owned-outputs.diamond.middle-start",
    |d: &first::Demand| query::first_output::Read(format!("first:{}", d.0)),
    |row: &query::Row| if row.key == "first:a" {
        vec![first::Demand("b".into())]
    } else {
        Vec::new()
    }
);
connection!(
    ToSecond,
    SecondConnection,
    Middle,
    MiddleOutput,
    hierarchy::Child,
    hierarchy::SeedInput,
    first::Demand,
    query::first_output::Read,
    second::Demand,
    "owned-outputs.diamond.middle-second",
    |d: &first::Demand| query::first_output::Read(format!("first:{}", d.0)),
    |row: &query::Row| if row.key == "first:b" {
        vec![second::Demand("a".into())]
    } else {
        Vec::new()
    }
);
pub type Tree = ApplicationOutputGraph<
    super::super::super::program::Connection,
    (
        ApplicationOutputEdge<LeftConnection, LeftChildren>,
        ApplicationOutputEdge<
            MiddleConnection,
            ApplicationOutputEdge<
                SecondConnection,
                ApplicationOutputEdge<RightJoinConnection, ApplicationOutputLeaf>,
            >,
        >,
    ),
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.diamond.unequal-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.diamond.unequal-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::super::program::entry_spec::<Delivery>();
        specs.push(
            ApplicationFeatureSpec::root::<Schema, Middle>()
                .provides::<MiddleOutput>()
                .finish(),
        );
        specs.push(ApplicationFeatureSpec::root::<Schema, Extra>().finish());
        specs
    }
}
