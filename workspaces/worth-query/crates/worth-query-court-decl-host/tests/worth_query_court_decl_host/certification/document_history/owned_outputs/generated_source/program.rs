//! Fixed edit completion and actual child discovery share the normal program owners.
use super::super::hierarchy::{Child, ChildInput, HierarchyProgram, Input, Output, Parent};
use super::super::program::{Entry, EntryOutput};
use super::*;
macro_rules! program {
    ($module:ident, $family:ident, $discovery:ident, $id:literal) => { pub mod $module {
        use super::*;
        pub struct Start;
        impl ApplicationConnectionIdentity for Start { const IDENTITY: &'static str = concat!($id, ".start"); }
        impl ApplicationOccurrenceConnectionBinding<Schema, Entry, Parent> for Start {}
        pub type RootConnection = ApplicationConnectionRef<Schema, Entry, EntryOutput, Parent, Input, Start>;
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
            type Source = edit::Binding;
            type Demand = first::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn demand_from_source(input: &edit::Intent) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok(first::Demand(input.key.clone()))
            }
        }
        impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
            fn demand_from_source(input: &edit::Intent) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
                <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
            }
        }
        pub struct Discover;
        impl ApplicationConnectionIdentity for Discover { const IDENTITY: &'static str = concat!($id, ".discover"); }
        impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for Discover {}
        pub type ChildConnection = ApplicationConnectionRef<Schema, Parent, Output, Child, ChildInput, Discover>;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
            type RootDemand = first::Demand;
            type Discovery = query::$discovery::Read;
            type Demand = $family::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_root(root: &first::Demand) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::$discovery::Read(format!("first:{}", root.0)))
            }
            fn demands_from_discovery(row: &query::Row) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(vec![$family::Demand(row.key.clone())])
            }
        }
        pub type Root = ApplicationOutputGraph<RootConnection, ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>>;
        pub struct Delivery;
        impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
            type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
            const IDENTITY: &'static str = concat!($id, ".delivery");
        }
        impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
            type Root = Root;
            type Projection = ApplicationNoProjection;
        }
        pub type Action = ApplicationActionReference<Schema, ApplicationRootComposition, Entry, edit::Binding, Delivery>;
        pub struct Program;
        impl ApplicationProgramDefinition<Schema> for Program {
            type Contributions = (Contribution,);
            type Outputs = ApplicationProgramOutputs<Root>;
            type Rules = ApplicationRuleLeaf;
            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($id);
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                let mut specs = HierarchyProgram::feature_specs();
                specs[0] = ApplicationFeatureSpec::root::<Schema, Entry>().provides::<EntryOutput>()
                    .action::<synchronous_edit::Binding, Completion>().finish_action()
                    .action::<predecessor::synchronous::Binding, Completion>().finish_action()
                    .action::<edit::Binding, Delivery>().finish_action()
                    .action::<member_edit::Binding, Completion>().finish_action()
                    .action::<authored_member::retire::Binding, Completion>().finish_action()
                    .action::<authored_member::restore::Binding, Completion>().finish_action()
                    .action::<member_delete::Binding, Completion>().finish_action().finish();
                specs

            }
        }
    }};
}
program!(
    public,
    from_first,
    first_output,
    "owned-outputs.generated-source.public.v1"
);
program!(
    protected,
    restricted_from_first,
    first_output,
    "owned-outputs.generated-source.protected.v1"
);

program!(
    denied_discovery,
    restricted_from_first,
    first_discovery,
    "owned-outputs.generated-source.denied-discovery.v1"
);
