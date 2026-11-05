//! The program: the recorded model edit and one demand root per output family.
use super::*;
pub struct Entry;
impl ApplicationFeature<Schema> for Entry {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "owned-outputs.entry.v1";
}
pub struct Derived;
impl ApplicationFeature<Schema> for Derived {
    type Inputs = ApplicationFeatureInputList<
        FirstInput,
        ApplicationFeatureInputList<SecondInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "owned-outputs.derived.v1";
}
pub struct EntryOutput;
impl ApplicationOutputPort<Schema, Entry> for EntryOutput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "entry-output";
}
macro_rules! root {
    ($input:ident, $port:literal, $binding:ident, $alias:ident, $tree:ident, $family:ident, $name:literal) => {
        pub struct $input;
        impl ApplicationInputPort<Schema, Derived> for $input {
            type Value = query::RowBinding;
            const IDENTITY: &'static str = $port;
            const REQUIRED: bool = false;
        }
        pub struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $name;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Derived> for $binding {}
        pub type $alias =
            ApplicationConnectionRef<Schema, Entry, EntryOutput, Derived, $input, $binding>;
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
        pub type $tree = ApplicationDemandOutputGraph<$alias, ApplicationOutputLeaf>;
    };
}
root!(
    FirstInput,
    "first-input",
    StartFirst,
    FirstConnection,
    FirstTree,
    first,
    "owned-outputs.start-first"
);
root!(
    SecondInput,
    "second-input",
    StartSecond,
    SecondConnection,
    SecondTree,
    second,
    "owned-outputs.start-second"
);
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(FirstTree, SecondTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, Entry>()
                .provides::<EntryOutput>()
                .action::<edit::Binding, Completion>()
                .finish_action()
                .action::<member_edit::Binding, Completion>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, Derived>()
                .conditional_operation::<first::initial::Operation>()
                .conditional_operation::<first::preserve::Operation>()
                .conditional_operation::<second::initial::Operation>()
                .conditional_operation::<second::preserve::Operation>()
                .finish(),
        ]
    }
}
/// The recorded history action of the model edit.
pub type EditAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    edit::Binding,
    Completion,
>;

/// The child-scoped edit belongs to this model through ModelMember.
pub type MemberEditAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    member_edit::Binding,
    Completion,
>;
