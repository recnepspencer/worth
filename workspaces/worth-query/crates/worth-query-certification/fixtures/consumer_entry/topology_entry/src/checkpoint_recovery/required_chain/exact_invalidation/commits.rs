//! The commits the courtroom drives, each through a declared operation.

use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome as Performed;

impl Court<'_, '_, '_, '_> {
    /// Writes the Y of `body` through the declared source operation.
    pub(super) fn write_y(&self, body: &str, y: u64, at: &str) {
        let source = self.read(|| {
            self.request
                .query(PlanarRead {
                    body_key: body.to_owned(),
                })
                .execute()
        });
        let outcome = self
            .request
            .mutate(PlanarSourceAdjustment {
                scope_key: body.to_owned(),
                replacement_y: length(y),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_performed::<program::ChainProgram, program::ChainRoot>(self.application);
        let answer = match &outcome {
            Ok(Performed::Performed(_)) => return,
            Ok(Performed::RequiredOutputDenied { denial, .. }) => format!("{denial:?}"),
            Ok(Performed::NotPerformed(refused)) => format!("{refused:?}"),
            Err(denial) => format!("{denial:?}"),
        };
        panic!("{at}: the Y of {body} is written: {answer}");
    }

    /// Overwrites the published Length of `body` as a writer that is not its
    /// producer.
    pub(super) fn overwrite_length(&self, body: &str, value: u64, at: &str) {
        let source = self.read(|| {
            self.request
                .query(PlanarRead {
                    body_key: body.to_owned(),
                })
                .execute()
        });
        let outcome = self
            .request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: body.to_owned(),
                operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                    body_key: body.to_owned(),
                    value: length(value),
                }),
                validator_work: 4_096,
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_in_program::<program::ChainProgram>(self.application);
        assert!(
            matches!(
                &outcome,
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
            ),
            "{at}: the Length of {body} is overwritten: {outcome:?}"
        );
    }

    /// Deletes the body `a` precedes and creates `replacement` in its place:
    /// two relations go, two come, and an index key leaves and enters.
    pub(super) fn replace_successor(&self, ring: &mut Ring, replacement: String, y: u64, at: &str) {
        let source = self.read(|| {
            self.request
                .query(PlanarRead {
                    body_key: ring.key("a"),
                })
                .execute()
        });
        let outcome = self
            .request
            .mutate(VertexReplacement {
                scope_key: ring.key("a"),
                replacement: PlanarVertexReplacement {
                    retired_key: ring.successor.clone(),
                    next_key: ring.key("source-c"),
                    replacement: PlanarVertex {
                        body_key: replacement.clone(),
                        x: length(10 + ring_world::RING_SPACING * ring.index as u64),
                        y: length(y),
                    },
                },
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_in_program::<program::ChainProgram>(self.application);
        assert!(
            matches!(
                &outcome,
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
            ),
            "{at}: {} is replaced by {replacement}: {outcome:?}",
            ring.successor
        );
        ring.successor = replacement;
        ring.successor_y = y;
    }

    /// Creates ring `index` beside the rings already there, in one commit
    /// anchored on the first ring's root.
    pub(super) fn create_ring(&self, index: usize, at: &str) {
        let anchor = ring_world::key(0, "a");
        let source = self.read(|| {
            self.request
                .query(PlanarRead {
                    body_key: anchor.clone(),
                })
                .execute()
        });
        let outcome = self
            .request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: anchor,
                operation: PlanarOperation::CreateCycle(ring_world::vertices(index)),
                validator_work: 4_096,
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_in_program::<program::ChainProgram>(self.application);
        assert!(
            matches!(
                &outcome,
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
            ),
            "{at}: ring {index} is created: {outcome:?}"
        );
    }
}
