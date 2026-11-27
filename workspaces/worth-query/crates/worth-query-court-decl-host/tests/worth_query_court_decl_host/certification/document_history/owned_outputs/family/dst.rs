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
//! `dst-c3` to `dst-c8` extend that chain for the public #429 chain-depth
//! court: each reads the current output of the level below plus 1.
use super::*;

thread_local! {
    /// Chain candidates built on this thread, `dst-mid` first (public #429).
    static CHAIN_BUILDS: std::cell::Cell<[u64; 8]> = const { std::cell::Cell::new([0; 8]) };
}

/// Chain candidates built on this thread so far, `dst-mid`, `dst-leaf`, then
/// `dst-c3` to `dst-c8`: each is one execution of that output.
pub fn chain_builds() -> [u64; 8] {
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

/// One chain level above `dst-leaf`: the current output of the level below
/// plus 1, counted as build `$index`.
macro_rules! chain_level {
    ($module:ident, $profile:literal, $initial:literal, $preserve:literal, $initial_op:ident, $preserve_op:ident, $index:literal) => {
        family!(
            $module,
            SourceScopeCascade,
            output,
            Model,
            ModelKey,
            ModelValue,
            upstream_source,
            $profile,
            $initial,
            $preserve,
            $initial_op,
            $preserve_op,
            FirstOutput,
            FirstFacts,
            FirstKey,
            FirstValue,
            FirstOf,
            |_: u64| built($index, 1)
        );
    };
}
chain_level!(dst_c3, "dst-c3", "owned-outputs.dst-c3.initial", "owned-outputs.dst-c3.preserve", DstC3Initial, DstC3Preserve, 2);
chain_level!(dst_c4, "dst-c4", "owned-outputs.dst-c4.initial", "owned-outputs.dst-c4.preserve", DstC4Initial, DstC4Preserve, 3);
chain_level!(dst_c5, "dst-c5", "owned-outputs.dst-c5.initial", "owned-outputs.dst-c5.preserve", DstC5Initial, DstC5Preserve, 4);
chain_level!(dst_c6, "dst-c6", "owned-outputs.dst-c6.initial", "owned-outputs.dst-c6.preserve", DstC6Initial, DstC6Preserve, 5);
chain_level!(dst_c7, "dst-c7", "owned-outputs.dst-c7.initial", "owned-outputs.dst-c7.preserve", DstC7Initial, DstC7Preserve, 6);
chain_level!(dst_c8, "dst-c8", "owned-outputs.dst-c8.initial", "owned-outputs.dst-c8.preserve", DstC8Initial, DstC8Preserve, 7);

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
            .conditional_operation::<dst_c3::initial::Operation>()
            .conditional_operation::<dst_c3::preserve::Operation>()
            .conditional_operation::<dst_c4::initial::Operation>()
            .conditional_operation::<dst_c4::preserve::Operation>()
            .conditional_operation::<dst_c5::initial::Operation>()
            .conditional_operation::<dst_c5::preserve::Operation>()
            .conditional_operation::<dst_c6::initial::Operation>()
            .conditional_operation::<dst_c6::preserve::Operation>()
            .conditional_operation::<dst_c7::initial::Operation>()
            .conditional_operation::<dst_c7::preserve::Operation>()
            .conditional_operation::<dst_c8::initial::Operation>()
            .conditional_operation::<dst_c8::preserve::Operation>()
    }
}
