//! The commits the courtroom drives, each through a declared operation.

use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome as Performed;

impl Court<'_, '_, '_, '_> {
    /// Writes the Y of `body` through the declared source operation. The
    /// write is dropped: its required outputs never start.
    pub(super) fn write_y(&self, body: &str, y: u64, at: &str) {
        self.perform_y(body, y, false, at);
    }

    /// Writes the Y of `body` and starts the write's required outputs, which
    /// nobody advances.
    pub(super) fn write_y_and_abandon_its_outputs(&self, body: &str, y: u64, at: &str) {
        self.perform_y(body, y, true, at);
    }

    fn perform_y(&self, body: &str, y: u64, start: bool, at: &str) {
        let source = self
            .request
            .query(PlanarRead {
                body_key: body.to_owned(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
        let outcome = self
            .request
            .mutate(PlanarSourceAdjustment {
                scope_key: body.to_owned(),
                replacement_y: length(y),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&self.next_idempotency())
            .execute_performed::<program::ChainProgram, program::ChainRoot>(self.application);
        let answer = match outcome {
            Ok(Performed::Performed(performed)) => {
                if start {
                    let started = performed
                        .start_required_outputs(
                            self.request,
                            WorthQueryOutputDemandControls::host_policy(),
                        )
                        .map_err(|failure| format!("{:?}", failure.denial()));
                    assert!(
                        started.is_ok(),
                        "{at}: the required outputs of {body} start: {:?}",
                        started.err()
                    );
                }
                return;
            }
            Ok(Performed::RequiredOutputDenied { denial, .. }) => format!("{denial:?}"),
            Ok(Performed::NotPerformed(refused)) => format!("{refused:?}"),
            Err(denial) => format!("{denial:?}"),
        };
        panic!("{at}: the Y of {body} is written: {answer}");
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
        let source = self
            .request
            .query(PlanarRead {
                body_key: ring.key("a"),
            })
            .execute()
            .expect("the branch admits a courtroom read");
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
        let source = self
            .request
            .query(PlanarRead {
                body_key: anchor.clone(),
            })
            .execute()
            .expect("the branch admits a courtroom read");
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
