//! Reconstitute recorded identities directly; never reissue matching counters.
use super::*;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIncarnation, DurableProductUnpublishedOwnerEffectsIdentity,
    DurableWorldBootstrapAttemptIdentity,
};

pub(crate) fn readmit_commit(
    owner: RuntimeWorldOwnerIdentity,
    recorded: DurableCompositeCommitIdentity,
) -> Option<CompositeCommitIdentity> {
    (owner.durable() == Some(recorded.owner()))
        .then(|| CompositeCommitIdentity::issued(owner, recorded.ordinal()))
}
pub(crate) fn readmit_bootstrap(
    owner: RuntimeWorldOwnerIdentity,
    recorded: DurableWorldBootstrapAttemptIdentity,
) -> Option<RuntimeWorldBootstrapAttemptIdentity> {
    (owner.durable() == Some(recorded.owner()))
        .then(|| RuntimeWorldBootstrapAttemptIdentity::issued(owner, recorded.ordinal()))
}
pub(crate) fn readmit_incarnation(
    owner: RuntimeWorldOwnerIdentity,
    recorded: DurableProductBranchIncarnation,
) -> Option<ProductBranchIncarnation> {
    (owner.durable() == Some(recorded.owner()))
        .then(|| ProductBranchIncarnation::issued(owner, recorded.ordinal()))
}
pub(crate) fn readmit_attempt(
    owner: RuntimeWorldOwnerIdentity,
    recorded: DurableCompositePublicationAttemptIdentity,
) -> Option<CompositePublicationAttemptIdentity> {
    (owner.durable() == Some(recorded.owner()))
        .then(|| CompositePublicationAttemptIdentity::issued(owner, recorded.ordinal()))
}
pub(crate) fn readmit_custody(
    owner: RuntimeWorldOwnerIdentity,
    recorded: DurableProductUnpublishedOwnerEffectsIdentity,
) -> Option<ProductUnpublishedOwnerEffectsIdentity> {
    (owner.durable() == Some(recorded.owner()))
        .then(|| ProductUnpublishedOwnerEffectsIdentity::issued(owner, recorded.ordinal()))
}
