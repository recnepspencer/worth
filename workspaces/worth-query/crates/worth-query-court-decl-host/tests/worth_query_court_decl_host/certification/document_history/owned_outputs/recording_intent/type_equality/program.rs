//! Two connections from one parent capture different queries under one tree identity.
use super::super::super::hierarchy::SeedInput;
use super::super::super::hierarchy::{Child, HierarchyProgram, Output, Parent};
use super::super::super::program::{Entry, EntryOutput};
use super::super::{non_leaf_program as original, *};
pub struct OtherDiscover;
impl ApplicationConnectionIdentity for OtherDiscover {
    const IDENTITY: &'static str = "recording-intent.type-equality.other-discover";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Parent, Child> for OtherDiscover {}
pub type OtherConnection =
    ApplicationConnectionRef<Schema, Parent, Output, Child, SeedInput, OtherDiscover>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for OtherDiscover {
    type RootDemand = original::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        let key = if root.discovery.load(std::sync::atomic::Ordering::Relaxed) {
            "a"
        } else {
            "b"
        };
        Ok(query::first_output::Read(format!("first:{key}")))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        DISCOVERED.with(|rows| {
            if let Some(rows) = &mut *rows.borrow_mut() {
                rows.push(row.key.clone());
            }
        });
        Ok(vec![second::Demand(
            row.key
                .strip_prefix("first:")
                .expect("real first output")
                .into(),
        )])
    }
}
pub type Tree = ApplicationDemandOutputGraph<
    original::RootConnection,
    (
        ApplicationOutputEdge<original::ChildConnection, ApplicationOutputLeaf>,
        ApplicationOutputEdge<OtherConnection, ApplicationOutputLeaf>,
    ),
>;
thread_local! {
    static DISCOVERED: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}
pub(super) struct Observation;
impl Observation {
    pub(super) fn begin() -> Self {
        DISCOVERED.with(|rows| assert!(rows.replace(Some(Vec::new())).is_none()));
        Self
    }
    pub(super) fn saw(&self, key: &str) -> bool {
        DISCOVERED.with(|rows| {
            rows.borrow()
                .as_ref()
                .unwrap()
                .iter()
                .any(|row| row == &format!("first:{key}"))
        })
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        DISCOVERED.with(|rows| {
            rows.replace(None);
        });
    }
}
macro_rules! seed_root {
    ($binding:ident, $tree:ident, $feature:ident, $input:ident, $family:ident, $identity:literal) => {
        pub struct $feature;
        pub struct $input;
        impl ApplicationFeature<Schema> for $feature {
            type Inputs = ApplicationFeatureInputList<$input, ApplicationFeatureInputLeaf>;
            const IDENTITY: &'static str = concat!($identity, ".feature");
        }
        impl ApplicationInputPort<Schema, $feature> for $input {
            type Value = query::RowBinding;
            const IDENTITY: &'static str = concat!($identity, ".input");
            const REQUIRED: bool = false;
        }
        pub struct $binding;
        impl ApplicationConnectionIdentity for $binding {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, Entry, $feature> for $binding {}
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for $binding {
            type Source = $family::initial::Binding;
            type Demand = $family::Demand;
            const IDENTITY: &'static str = $identity;
            fn demand_from_source(
                input: &$family::initial::Input,
            ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok($family::Demand(input.0.key.clone()))
            }
        }
        pub type $tree = ApplicationDemandOutputGraph<
            ApplicationConnectionRef<Schema, Entry, EntryOutput, $feature, $input, $binding>,
            ApplicationOutputLeaf,
        >;
    };
}
seed_root!(
    FirstSeed,
    FirstSeedTree,
    FirstSeedFeature,
    FirstSeedInput,
    first,
    "recording-intent.type-equality.seed-first"
);
seed_root!(
    SecondSeed,
    SecondSeedTree,
    SecondSeedFeature,
    SecondSeedInput,
    second,
    "recording-intent.type-equality.seed-second"
);
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<((FirstSeedTree, SecondSeedTree), Tree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("recording-intent.type-equality");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = HierarchyProgram::feature_specs();
        features.extend([
            ApplicationFeatureSpec::root::<Schema, FirstSeedFeature>()
                .conditional_operation::<first::initial::Operation>()
                .conditional_operation::<first::preserve::Operation>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, SecondSeedFeature>()
                .conditional_operation::<second::initial::Operation>()
                .conditional_operation::<second::preserve::Operation>()
                .finish(),
        ]);
        features
    }
}
