//! The DST's program: one output tree rooted at model `a`'s upstream
//! output, with one `dst-a` and one `dst-b` output per member of `a` (the
//! model's `outA` and `outB`). The root is the new-member court's: its
//! source reads only the model, so a member the DST creates leaves it
//! healthy. A root whose source follows the member links (`first`) refuses
//! the creation with `StaleAuthorization` (traced, S1 probe).
//!
//! [`ConsumerTree`] adds one `dst-use` output of the model, which reads the
//! root's output through `current_output`; only the regression held on
//! public #358 demands it.
use super::*;
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::hierarchy::recording_parent::new_member::program as root;
use family::dst::{dst_a, dst_b, dst_use};

macro_rules! discover {
    ($name:ident, $connection:ident, $port:ty, $family:ident, $identity:literal) => {
        pub struct $name;
        impl ApplicationConnectionIdentity for $name {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child> for $name {}
        pub type $connection =
            ApplicationConnectionRef<Schema, hierarchy::Parent, hierarchy::Output, hierarchy::Child, $port, $name>;
        impl WorthQueryApplicationDependentOutputConnection<Schema> for $name {
            type RootDemand = upstream::Demand;
            type Discovery = query::members::Read;
            type Demand = $family::Demand;
            const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
            fn discovery_from_root(
                root: &upstream::Demand,
            ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
                Ok(query::members::Read(root.0.clone()))
            }
            fn demands_from_discovery(
                rows: &query::members::Members,
            ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
                Ok(rows.members.iter().map(|row| $family::Demand(row.key.clone())).collect())
            }
        }
    };
}
discover!(DiscoverA, ConnectionA, hierarchy::ChildInput, dst_a, "owned-outputs.dst.discover-a");
discover!(DiscoverB, ConnectionB, hierarchy::SeedInput, dst_b, "owned-outputs.dst.discover-b");

/// The consumer: once the root has generated the member `upstream:<model>`,
/// `dst-use` of that model reads the root's current output, so its row
/// records the root's output as consumed.
pub struct Consume;
impl ApplicationConnectionIdentity for Consume {
    const IDENTITY: &'static str = "owned-outputs.dst.consume";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child> for Consume {}
pub type ConsumerConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    super::super::super::synchronous_edit::interleaved::PendingSeedInput,
    Consume,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Consume {
    type RootDemand = upstream::Demand;
    type Discovery = query::member_public::Read;
    type Demand = dst_use::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &upstream::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::member_public::Read(format!("upstream:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        let model = row.key.strip_prefix("upstream:").unwrap_or(&row.key);
        Ok(vec![dst_use::Demand(model.to_owned())])
    }
}

pub type Tree = ApplicationDemandOutputGraph<
    root::RootConnection,
    (
        ApplicationOutputEdge<ConnectionA, ApplicationOutputLeaf>,
        ApplicationOutputEdge<ConnectionB, ApplicationOutputLeaf>,
    ),
>;

pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, ConsumerTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new("owned-outputs.dst.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        root::Program::feature_specs()
    }
}

/// [`Tree`] with the `dst-use` consumer of the root beside its leaves.
/// [`Program`] declares both; a document binds one ([`Shape`]).
pub type ConsumerTree = ApplicationDemandOutputGraph<
    root::RootConnection,
    (
        ApplicationOutputEdge<ConnectionA, ApplicationOutputLeaf>,
        (
            ApplicationOutputEdge<ConnectionB, ApplicationOutputLeaf>,
            ApplicationOutputEdge<ConsumerConnection, ApplicationOutputLeaf>,
        ),
    ),
>;

/// Which tree a document binds and a write outside settles.
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Members,
    WithConsumer,
}
