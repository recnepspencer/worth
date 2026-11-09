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
        ApplicationFeatureInputList<
            SecondInput,
            ApplicationFeatureInputList<
                ChildInput,
                ApplicationFeatureInputList<
                    RestrictedInput,
                    ApplicationFeatureInputList<
                        UpstreamInput,
                        ApplicationFeatureInputList<
                            DownstreamInput,
                            ApplicationFeatureInputList<
                                RestrictedOwnedInput,
                                ApplicationFeatureInputList<
                                    FromFirstInput,
                                    ApplicationFeatureInputList<
                                        RestrictedFromFirstInput,
                                        ApplicationFeatureInputLeaf,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
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
root!(
    ChildInput,
    "child-input",
    StartChild,
    ChildConnection,
    ChildTree,
    child,
    "owned-outputs.start-child"
);
root!(
    RestrictedInput,
    "restricted-input",
    StartRestricted,
    RestrictedConnection,
    RestrictedTree,
    restricted,
    "owned-outputs.start-restricted"
);
root!(
    UpstreamInput,
    "upstream-input",
    StartUpstream,
    UpstreamConnection,
    UpstreamTree,
    upstream,
    "owned-outputs.start-upstream"
);
root!(
    DownstreamInput,
    "downstream-input",
    StartDownstream,
    DownstreamConnection,
    DownstreamTree,
    downstream,
    "owned-outputs.start-downstream"
);
root!(
    RestrictedOwnedInput,
    "restricted-owned-input",
    StartRestrictedOwned,
    RestrictedOwnedConnection,
    RestrictedOwnedTree,
    restricted_owned,
    "owned-outputs.start-restricted-owned"
);
root!(
    FromFirstInput,
    "from-first-input",
    StartFromFirst,
    FromFirstConnection,
    FromFirstTree,
    from_first,
    "owned-outputs.start-from-first"
);
root!(
    RestrictedFromFirstInput,
    "restricted-from-first-input",
    StartRestrictedFromFirst,
    RestrictedFromFirstConnection,
    RestrictedFromFirstTree,
    restricted_from_first,
    "owned-outputs.start-restricted-from-first"
);
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        FirstTree,
        (
            SecondTree,
            (
                ChildTree,
                (
                    RestrictedTree,
                    (
                        UpstreamTree,
                        (
                            DownstreamTree,
                            (
                                RestrictedOwnedTree,
                                (FromFirstTree, RestrictedFromFirstTree),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            entry_spec::<Completion>(),
            ApplicationFeatureSpec::root::<Schema, Derived>()
                .conditional_operation::<from_first::initial::Operation>()
                .conditional_operation::<from_first::preserve::Operation>()
                .conditional_operation::<restricted_from_first::initial::Operation>()
                .conditional_operation::<restricted_from_first::preserve::Operation>()
                .conditional_operation::<first::initial::Operation>()
                .conditional_operation::<first::preserve::Operation>()
                .conditional_operation::<second::initial::Operation>()
                .conditional_operation::<second::preserve::Operation>()
                .conditional_operation::<child::initial::Operation>()
                .conditional_operation::<child::preserve::Operation>()
                .conditional_operation::<restricted::initial::Operation>()
                .conditional_operation::<restricted::preserve::Operation>()
                .conditional_operation::<upstream::initial::Operation>()
                .conditional_operation::<upstream::preserve::Operation>()
                .conditional_operation::<downstream::initial::Operation>()
                .conditional_operation::<downstream::preserve::Operation>()
                .conditional_operation::<restricted_owned::initial::Operation>()
                .conditional_operation::<restricted_owned::preserve::Operation>()
                .finish(),
            predecessor::program::feature(),
            family::diamond::feature(),
            family::decision_input::feature(),
            family::convergent::feature(),
            family::ordinary_input::feature(),
            family::decision_write::feature(),
            synchronous_edit::reconciliation::feature(),
            synchronous_edit::reconciliation::child_feature(),
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

pub type DeleteMemberAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    member_delete::Binding,
    Completion,
>;

pub type AuthoredRetireAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    authored_member::retire::Binding,
    Completion,
>;

pub type AuthoredRestoreAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    authored_member::restore::Binding,
    Completion,
>;

pub(super) fn entry_spec<Delivery: ApplicationActionDelivery<Schema, edit::Binding>>(
) -> ApplicationFeatureSpec {
    entry_spec_with_member_delivery::<Delivery, Completion>()
}
pub(super) fn entry_spec_with_member_delivery<
    Delivery: ApplicationActionDelivery<Schema, edit::Binding>,
    MemberDelivery: ApplicationActionDelivery<Schema, member_edit::Binding>,
>() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<Schema, Entry>()
        .provides::<EntryOutput>()
        .action::<predecessor::synchronous::Binding, Completion>()
        .finish_action()
        .action::<synchronous_edit::new_source::Binding, Completion>()
        .finish_action()
        .action::<synchronous_edit::new_source::restricted::Binding, Completion>()
        .finish_action()
        .action::<synchronous_edit::Binding, Completion>()
        .finish_action()
        .action::<edit::Binding, Delivery>()
        .finish_action()
        .action::<member_edit::Binding, MemberDelivery>()
        .finish_action()
        .action::<authored_member::retire::Binding, Completion>()
        .finish_action()
        .action::<authored_member::restore::Binding, Completion>()
        .finish_action()
        .action::<member_delete::Binding, Completion>()
        .finish_action()
        .finish()
}
