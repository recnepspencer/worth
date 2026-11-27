//! The output families of the history DST (`dst/fixture.rs`). `dst-a` and
//! `dst-b` each derive one child output per authored member, owned by the
//! member's source scope. The model's `outA` is `dst-a`'s value, `outB` is
//! `dst-b`'s.
//! `dst-use` consumes the tree's root: its decision reads the root's output
//! through `current_output`, so its row carries consumed-output evidence.
//! `dst-mid` and `dst-leaf` are a two-hop consumer chain rooted on member
//! `e1` (public #358): `dst-mid` of the model reads `dst-a`'s current output
//! for `e1` and keeps its parity, so an `e1` write that keeps the parity
//! re-executes it to an equal output; `dst-leaf` reads `dst-mid`'s current
//! output plus 100. Neither reads `e1`'s value directly.
use super::*;

thread_local! {
    /// `dst-mid` and `dst-leaf` candidates built on this thread (public #429).
    static CHAIN_BUILDS: std::cell::Cell<[u64; 2]> = const { std::cell::Cell::new([0; 2]) };
}

/// `dst-mid` and `dst-leaf` candidates built on this thread so far: each is
/// one execution of that output.
pub fn chain_builds() -> [u64; 2] {
    CHAIN_BUILDS.get()
}

/// Count a build of chain output `index`, then derive `value`.
fn built(index: usize, value: u64) -> u64 {
    let mut builds = CHAIN_BUILDS.get();
    builds[index] += 1;
    CHAIN_BUILDS.set(builds);
    value
}
family!(
    dst_a,
    SourceScopeCascade,
    source,
    Member,
    MemberKey,
    MemberValue,
    member_public,
    "dst-a",
    "owned-outputs.dst-a.initial",
    "owned-outputs.dst-a.preserve",
    DstAInitial,
    DstAPreserve,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOf,
    |value: u64| value
);
family!(
    dst_b,
    SourceScopeCascade,
    source,
    Member,
    MemberKey,
    MemberValue,
    member_public,
    "dst-b",
    "owned-outputs.dst-b.initial",
    "owned-outputs.dst-b.preserve",
    DstBInitial,
    DstBPreserve,
    ChildOutput,
    ChildFacts,
    ChildKey,
    ChildValue,
    ChildOf,
    |value: u64| value + 10
);
family!(
    dst_use,
    SourceScopeCascade,
    output,
    Model,
    ModelKey,
    ModelValue,
    upstream_source,
    "dst-use",
    "owned-outputs.dst-use.initial",
    "owned-outputs.dst-use.preserve",
    DstUseInitial,
    DstUsePreserve,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf,
    |_: u64| 0
);

family!(
    dst_mid,
    SourceScopeCascade,
    output,
    Model,
    ModelKey,
    ModelValue,
    upstream_source,
    "dst-mid",
    "owned-outputs.dst-mid.initial",
    "owned-outputs.dst-mid.preserve",
    DstMidInitial,
    DstMidPreserve,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf,
    |_: u64| built(0, 0)
);
family!(
    dst_leaf,
    SourceScopeCascade,
    output,
    Model,
    ModelKey,
    ModelValue,
    upstream_source,
    "dst-leaf",
    "owned-outputs.dst-leaf.initial",
    "owned-outputs.dst-leaf.preserve",
    DstLeafInitial,
    DstLeafPreserve,
    FirstOutput,
    FirstFacts,
    FirstKey,
    FirstValue,
    FirstOf,
    |_: u64| built(1, 100)
);

/// Declares the DST families' conditional operations on a root feature: a
/// program that installs the owned world must name every family it holds.
pub trait DstOperations {
    fn dst_operations(self) -> Self;
}
impl<Feature: ApplicationFeature<Schema>> DstOperations
    for ApplicationFeatureSpecBuilder<Schema, ApplicationRootComposition, Feature>
{
    fn dst_operations(self) -> Self {
        self.conditional_operation::<dst_a::initial::Operation>()
            .conditional_operation::<dst_a::preserve::Operation>()
            .conditional_operation::<dst_b::initial::Operation>()
            .conditional_operation::<dst_b::preserve::Operation>()
            .conditional_operation::<dst_use::initial::Operation>()
            .conditional_operation::<dst_use::preserve::Operation>()
            .conditional_operation::<dst_mid::initial::Operation>()
            .conditional_operation::<dst_mid::preserve::Operation>()
            .conditional_operation::<dst_leaf::initial::Operation>()
            .conditional_operation::<dst_leaf::preserve::Operation>()
    }
}
