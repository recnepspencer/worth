//! The commits the courtroom drives, each through a declared operation.

use super::*;

impl Court<'_, '_, '_, '_> {
    /// Change source facts while leaving output publication to the tested demand.
    pub(super) fn write_y(&self, body: &str, y: u64, at: &str) {
        let source = self
            .request
            .query(PlanarRead {
                body_key: body.to_owned(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        let outcome = self
            .request
            .advanced()
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: body.to_owned(),
                operation: PlanarOperation::Adjust(vec![
                    worth_query_consumer_values::PlanarAdjustment {
                        body_key: body.to_owned(),
                        replacement_y: length(y),
                    },
                ]),
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_in_program::<program::ChainProgram>(
                self.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        assert!(
            matches!(
                &outcome,
                Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
            ),
            "{at}: the Y of {body} is written: {outcome:?}"
        );
    }

    /// Overwrites the published Length of `body` as a writer that is not its
    /// producer.
    pub(super) fn overwrite_length(&self, body: &str, value: u64, at: &str) {
        let source = self
            .request
            .query(PlanarRead {
                body_key: body.to_owned(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        let outcome = self
            .request
            .advanced()
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: body.to_owned(),
                operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                    body_key: body.to_owned(),
                    value: length(value),
                }),
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_in_program::<program::ChainProgram>(
                self.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
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
        let source = self
            .request
            .query(PlanarRead {
                body_key: ring.key("a"),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        let outcome = self
            .request
            .advanced()
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
            .execute_in_program::<program::ChainProgram>(
                self.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
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
}
