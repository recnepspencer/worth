//! Actual pending Home evidence, never caller-supplied generation or fate.
use super::*;
use worth_runtime_world::facade::{WorldCheckpointSelection, WorldCheckpointSubject};
pub(super) fn observe(
    source: &ApplicationWorldCheckpointSource,
    subject: &WorldCheckpointSubject,
) -> Result<Option<WorldCheckpointSelection>, Denial> {
    let Some(owner) = source.owner.upgrade() else {
        return Ok(None);
    };
    owner
        .with_selected_world(subject, |world, expected, membership| {
            let actual = membership.version();
            let anchor = worth_store::physical_runtime::AnchoredRecordName::new(
                crate::registry::CHECKPOINT_ANCHOR,
            )
            .expect("closed Home checkpoint anchor");
            if !world.publication_subject().same_capture(subject)
                || actual.name() != anchor
                || actual.generation() != expected.generation
                || actual.payload_sha256() != expected.payload_sha256
            {
                return None;
            }
            if actual.payload_bytes() != expected.payload_bytes as u64 {
                return None;
            }
            Some(WorldCheckpointSelection::from_home_checkpoint_selection(
                subject,
                actual.generation(),
                actual.payload_sha256(),
            ))
        })
        .map(Option::flatten)
}
