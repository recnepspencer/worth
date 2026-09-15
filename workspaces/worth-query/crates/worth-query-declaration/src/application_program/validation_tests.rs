use super::*;
use crate::application_schema::{
    ApplicationInvariantMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial, ApplicationStructuredValueBinding,
    ApplicationValueValidationDenial,
};

struct Schema;

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "ProgramGraph";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("program validation does not install a schema")
    }
}

struct ValueBinding;

impl ApplicationStructuredValueBinding for ValueBinding {
    type Value = ();
    const IDENTITY_NAME: &'static str = "worth.query.tests.program-value.v1";

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

struct Source;
struct OtherSource;
struct Target;
struct SourceOutput;
struct OtherOutput;
struct TargetInput;
struct OtherTargetInput;
struct TargetOutput;
struct TargetOutputTwo;
struct SourceInput;

macro_rules! feature {
    ($feature:ty, $identity:literal) => {
        impl ApplicationFeature<Schema> for $feature {
            const IDENTITY: &'static str = $identity;
        }
    };
}

feature!(Source, "source");
feature!(OtherSource, "other-source");
feature!(Target, "target");

impl ApplicationOutputPort<Schema, Source> for SourceOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "output";
}

impl ApplicationOutputPort<Schema, OtherSource> for OtherOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "output";
}

impl ApplicationInputPort<Schema, Target> for TargetInput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}

impl ApplicationInputPort<Schema, Target> for OtherTargetInput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "other-source";
    const REQUIRED: bool = true;
}

impl ApplicationOutputPort<Schema, Target> for TargetOutput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "result";
    const REQUIRED: bool = true;
}

impl ApplicationOutputPort<Schema, Target> for TargetOutputTwo {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "second-result";
    const REQUIRED: bool = true;
}

impl ApplicationInputPort<Schema, Source> for SourceInput {
    type Value = ValueBinding;
    const IDENTITY: &'static str = "feedback";
    const REQUIRED: bool = true;
}

struct SourceToTarget;
struct OtherToTarget;
struct TargetToSource;

impl ApplicationConnectionIdentity for SourceToTarget {
    const IDENTITY: &'static str = "source-to-target";
}

impl ApplicationOccurrenceConnectionBinding<Schema, Source, Target> for SourceToTarget {}

impl ApplicationConnectionIdentity for OtherToTarget {
    const IDENTITY: &'static str = "other-to-target";
}

impl ApplicationOccurrenceConnectionBinding<Schema, OtherSource, Target> for OtherToTarget {}

impl ApplicationConnectionIdentity for TargetToSource {
    const IDENTITY: &'static str = "target-to-source";
}

impl ApplicationOccurrenceConnectionBinding<Schema, Target, Source> for TargetToSource {}

type SourceNode =
    ApplicationProgramFeature<Source, (), (SourceOutput,), ApplicationFeatureAvailable>;
type UnavailableSourceNode =
    ApplicationProgramFeature<Source, (), (SourceOutput,), ApplicationFeatureUnavailable>;
type OtherSourceNode =
    ApplicationProgramFeature<OtherSource, (), (OtherOutput,), ApplicationFeatureAvailable>;
type TargetNode =
    ApplicationProgramFeature<Target, (TargetInput,), (TargetOutput,), ApplicationFeatureAvailable>;
type UnavailableTargetNode = ApplicationProgramFeature<
    Target,
    (TargetInput,),
    (TargetOutput,),
    ApplicationFeatureUnavailable,
>;
type FanInTargetNode = ApplicationProgramFeature<
    Target,
    (TargetInput, OtherTargetInput),
    (TargetOutput,),
    ApplicationFeatureAvailable,
>;
type CycleSourceNode =
    ApplicationProgramFeature<Source, (SourceInput,), (SourceOutput,), ApplicationFeatureAvailable>;
type TwoOutputTargetNode = ApplicationProgramFeature<
    Target,
    (TargetInput,),
    (TargetOutput, TargetOutputTwo),
    ApplicationFeatureAvailable,
>;

type Connection = ApplicationProgramDependentConnection<
    ApplicationConnectionRef<Schema, Source, SourceOutput, Target, TargetInput, SourceToTarget>,
>;
type OtherConnection = ApplicationProgramDependentConnection<
    ApplicationConnectionRef<
        Schema,
        OtherSource,
        OtherOutput,
        Target,
        OtherTargetInput,
        OtherToTarget,
    >,
>;
type UnavailableConnection = ApplicationProgramUnavailableConnection<
    ApplicationConnectionRef<Schema, Source, SourceOutput, Target, TargetInput, SourceToTarget>,
>;
type ReverseConnection = ApplicationProgramDependentConnection<
    ApplicationConnectionRef<Schema, Target, TargetOutput, Source, SourceInput, TargetToSource>,
>;

struct Inventory;

impl ApplicationProgramInventoryIdentity for Inventory {
    const IDENTITY: &'static str = "complete";
}

type Output = ApplicationProgramOutput<Target, TargetOutput>;
type InventoryNode = ApplicationProgramInventory<Inventory, (Output,)>;
type EmptyInventoryNode = ApplicationProgramInventory<Inventory, ()>;
type UnavailableOutput = ApplicationProgramOutput<Source, SourceOutput>;
type UnavailableInventoryNode = ApplicationProgramInventory<Inventory, (UnavailableOutput,)>;

struct Rule;

impl ApplicationInvariantMarkerIdentity<Schema> for Rule {
    const IDENTIFIER: &'static str = "program-rule";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

type Rules = (ApplicationProgramSharedRule<Rule, AtCommitBoundary>,);

macro_rules! program {
    ($name:ident, $features:ty, $connections:ty, $inventories:ty) => {
        struct $name;

        impl ApplicationProgramDefinition<Schema> for $name {
            type Contributions = ();
            type Features = $features;
            type Connections = $connections;
            type Rules = Rules;
            type Inventories = $inventories;
            const IDENTITY: ApplicationProgramIdentity =
                ApplicationProgramIdentity::new(stringify!($name));
        }
    };
}

program!(
    Valid,
    (SourceNode, TargetNode),
    (Connection,),
    (InventoryNode,)
);
program!(
    DuplicateFeature,
    (SourceNode, SourceNode, TargetNode),
    (Connection,),
    (InventoryNode,)
);
program!(
    DuplicateConnection,
    (SourceNode, TargetNode),
    (Connection, Connection),
    (InventoryNode,)
);
program!(
    EmptyInventory,
    (SourceNode, TargetNode),
    (Connection,),
    (EmptyInventoryNode,)
);
program!(
    OrphanFeature,
    (SourceNode, OtherSourceNode, TargetNode),
    (Connection,),
    (InventoryNode,)
);
program!(
    UnavailableDependency,
    (UnavailableSourceNode, TargetNode),
    (Connection,),
    (InventoryNode,)
);
program!(
    UnavailableTargetWithDependent,
    (SourceNode, UnavailableTargetNode),
    (Connection,),
    (InventoryNode,)
);
program!(
    AvailableTargetWithUnavailable,
    (SourceNode, TargetNode),
    (UnavailableConnection,),
    (InventoryNode,)
);
program!(
    FanIn,
    (SourceNode, OtherSourceNode, FanInTargetNode),
    (Connection, OtherConnection),
    (InventoryNode,)
);
program!(
    DuplicateInventory,
    (SourceNode, TargetNode),
    (Connection,),
    (InventoryNode, InventoryNode)
);
program!(MissingInput, (TargetNode,), (), (InventoryNode,));
program!(
    CycleProgram,
    (CycleSourceNode, TargetNode),
    (Connection, ReverseConnection),
    (InventoryNode,)
);
program!(
    IncompleteInventoryProgram,
    (SourceNode, TwoOutputTargetNode),
    (Connection,),
    (InventoryNode,)
);
program!(
    PlannedUnavailable,
    (UnavailableSourceNode,),
    (),
    (UnavailableInventoryNode,)
);

struct NoRules;
impl ApplicationProgramDefinition<Schema> for NoRules {
    type Contributions = ();
    type Features = (SourceNode, TargetNode);
    type Connections = (Connection,);
    type Rules = ();
    type Inventories = (InventoryNode,);
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new("NoRules");
}

mod impostors;

fn denial<Program>() -> ApplicationProgramValidationDenial
where
    Program: ApplicationProgramDefinition<Schema>,
{
    match validate_application_program::<Schema, Program>() {
        Ok(_) => panic!("program must be denied"),
        Err(denial) => denial,
    }
}

mod assertions;
