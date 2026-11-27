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
//!
//! [`ChainTree`] adds the #358 consumer chain instead: `dst-mid` of the
//! model reads `dst-a`'s current output for member `e1`, and `dst-leaf`
//! reads `dst-mid`'s current output, so a readmitted consumer row is
//! consumed again one hop downstream.
//!
//! [`Chain1Tree`], [`Chain4Tree`] and [`Chain8Tree`] bind that chain one,
//! four and eight levels deep for the public #429 chain-depth court:
//! `dst-c3` to `dst-c8` each read the current output of the level below.
use super::*;
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::hierarchy::recording_parent::new_member::program as root;
use family::dst::{dst_a, dst_b, dst_c3, dst_c4, dst_c5, dst_c6, dst_c7, dst_c8, dst_leaf, dst_mid, dst_use};

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

/// A model-keyed consumer demanded once the root has generated the member
/// `upstream:<model>`, as [`Consume`] is.
macro_rules! consume {
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
            type Discovery = query::member_public::Read;
            type Demand = $family::Demand;
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
                Ok(vec![$family::Demand(model.to_owned())])
            }
        }
    };
}
consume!(
    Mid,
    MidConnection,
    hierarchy::ChainInput,
    dst_mid,
    "owned-outputs.dst.mid"
);
consume!(
    Leaf,
    LeafConnection,
    super::super::super::family::borrowed_inputs::program::HeaderInput,
    dst_leaf,
    "owned-outputs.dst.leaf"
);
consume!(C3, C3Connection, hierarchy::ChainInput3, dst_c3, "owned-outputs.dst.c3");
consume!(C4, C4Connection, hierarchy::ChainInput4, dst_c4, "owned-outputs.dst.c4");
consume!(C5, C5Connection, hierarchy::ChainInput5, dst_c5, "owned-outputs.dst.c5");
consume!(C6, C6Connection, hierarchy::ChainInput6, dst_c6, "owned-outputs.dst.c6");
consume!(C7, C7Connection, hierarchy::ChainInput7, dst_c7, "owned-outputs.dst.c7");
consume!(C8, C8Connection, hierarchy::ChainInput8, dst_c8, "owned-outputs.dst.c8");

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
    type Outputs = ApplicationProgramOutputs<(Tree, (ConsumerTree, (ChainTree, (Chain1Tree, (Chain4Tree, Chain8Tree)))))>;
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

/// [`Tree`] with the #358 chain (`dst-mid`, then `dst-leaf`) beside its
/// leaves.
pub type ChainTree = ApplicationDemandOutputGraph<
    root::RootConnection,
    (
        ApplicationOutputEdge<ConnectionA, ApplicationOutputLeaf>,
        (
            ApplicationOutputEdge<ConnectionB, ApplicationOutputLeaf>,
            (
                ApplicationOutputEdge<MidConnection, ApplicationOutputLeaf>,
                ApplicationOutputEdge<LeafConnection, ApplicationOutputLeaf>,
            ),
        ),
    ),
>;

/// [`Tree`] with `dst-mid` alone: the chain one level deep.
pub type Chain1Tree = ApplicationDemandOutputGraph<
    root::RootConnection,
    (
        ApplicationOutputEdge<ConnectionA, ApplicationOutputLeaf>,
        (ApplicationOutputEdge<ConnectionB, ApplicationOutputLeaf>, ApplicationOutputEdge<MidConnection, ApplicationOutputLeaf>),
    ),
>;

/// [`Tree`] with the chain four levels deep, `dst-mid` to `dst-c4`.
pub type Chain4Tree = ApplicationDemandOutputGraph<
    root::RootConnection,
    (
        ApplicationOutputEdge<ConnectionA, ApplicationOutputLeaf>,
        (ApplicationOutputEdge<ConnectionB, ApplicationOutputLeaf>, (ApplicationOutputEdge<MidConnection, ApplicationOutputLeaf>, (ApplicationOutputEdge<LeafConnection, ApplicationOutputLeaf>, (ApplicationOutputEdge<C3Connection, ApplicationOutputLeaf>, ApplicationOutputEdge<C4Connection, ApplicationOutputLeaf>)))),
    ),
>;

/// [`Tree`] with the chain eight levels deep, `dst-mid` to `dst-c8`.
pub type Chain8Tree = ApplicationDemandOutputGraph<
    root::RootConnection,
    (
        ApplicationOutputEdge<ConnectionA, ApplicationOutputLeaf>,
        (ApplicationOutputEdge<ConnectionB, ApplicationOutputLeaf>, (ApplicationOutputEdge<MidConnection, ApplicationOutputLeaf>, (ApplicationOutputEdge<LeafConnection, ApplicationOutputLeaf>, (ApplicationOutputEdge<C3Connection, ApplicationOutputLeaf>, (ApplicationOutputEdge<C4Connection, ApplicationOutputLeaf>, (ApplicationOutputEdge<C5Connection, ApplicationOutputLeaf>, (ApplicationOutputEdge<C6Connection, ApplicationOutputLeaf>, (ApplicationOutputEdge<C7Connection, ApplicationOutputLeaf>, ApplicationOutputEdge<C8Connection, ApplicationOutputLeaf>)))))))),
    ),
>;

/// Which tree a document binds and a write outside settles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Members,
    WithConsumer,
    /// The #358 chain, two levels deep.
    WithChain,
    ChainOf1,
    ChainOf4,
    ChainOf8,
}

impl Shape {
    /// How many chain levels the tree binds above `dst-a`.
    pub fn chain_depth(self) -> usize {
        match self {
            Self::Members | Self::WithConsumer => 0,
            Self::ChainOf1 => 1,
            Self::WithChain => 2,
            Self::ChainOf4 => 4,
            Self::ChainOf8 => 8,
        }
    }

    /// The chain tree `depth` levels deep.
    pub fn chain_of(depth: usize) -> Self {
        match depth {
            1 => Self::ChainOf1,
            2 => Self::WithChain,
            4 => Self::ChainOf4,
            8 => Self::ChainOf8,
            _ => panic!("no chain tree is {depth} levels deep"),
        }
    }
}
