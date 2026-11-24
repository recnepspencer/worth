//! The output families of the history DST (`dst/fixture.rs`). `dst-a` and
//! `dst-b` each derive one child output per authored member, owned by the
//! member's source scope. The model's `outA` is `dst-a`'s value, `outB` is
//! `dst-b`'s.
//! `dst-use` consumes the tree's root: its decision reads the root's output
//! through `current_output`, so its row carries consumed-output evidence.
use super::*;
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
    }
}
