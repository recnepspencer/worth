//! Copy actual published input into an independent physical source placement.
use std::sync::Arc;
use worth_relational::facade::durability::RelationalDurableRecoverySource;
use worth_testkit_relational::{
    cold_recovery::ColdRecoverySourceFixture,
    durable_medium::immutable_members::ImmutableCheckpointMemberSource,
};
pub(super) fn copy_input(
    original: &ColdRecoverySourceFixture,
) -> (
    RelationalDurableRecoverySource,
    Arc<ImmutableCheckpointMemberSource>,
) {
    let source = original.source.with_published_members(&[]);
    let input = RelationalDurableRecoverySource::new(
        original.input.checkpoint().cloned(),
        original.input.tail().to_vec(),
        original.input.transitions().to_vec(),
        source.clone(),
    );
    (input, source)
}
