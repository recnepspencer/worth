//! Read-only, receiver-scoped observation of restoration validation work.
use super::*;
use std::cell::Cell;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};

#[derive(Clone, Copy)]
struct Observation {
    commit: CommitId,
    version: VersionId,
    validations: u64,
    work: u64,
}
thread_local! {
    static ACTIVE: Cell<Option<Observation>> = const { Cell::new(None) };
}
struct Recording;
impl Drop for Recording {
    fn drop(&mut self) {
        ACTIVE.set(None);
    }
}

impl CanonicalCommitEnvelope {
    /// Corrupt only a copied restoration roster and run ordinary validation.
    /// The copy cannot publish or yield an origin when that validation refuses.
    pub fn output_origin_header_with_missing_entity_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(u64, u64), String> {
        let event = self
            .authored_cascade_restoration
            .as_ref()
            .ok_or("restoration absent")?;
        let mut entities = event.entities.clone();
        let (_, member) = entities
            .iter_mut()
            .find(|(_, id)| {
                *id != event.anchor && !event.co_anchors.iter().any(|(_, anchor)| anchor == id)
            })
            .ok_or("restoration has no output member")?;
        *member = EntityId::new(
            member.partition_id,
            member.local_slot_value(),
            member
                .generation_value()
                .checked_add(1024)
                .ok_or("corrupt generation overflows")?,
        );
        let mut envelope = self.clone();
        envelope.authored_cascade_restoration =
            Some(std::sync::Arc::new(CanonicalAuthoredRestoration {
                owned_members: event.owned_members,
                retirement: event.retirement,
                original_anchor: event.original_anchor,
                anchor: event.anchor,
                co_anchors: event.co_anchors.clone(),
                entities,
                relations: event.relations.clone(),
                publications: event.publications.clone(),
            }));
        envelope.output_origin_header_work_for_test(execution, 0)
    }

    /// Court door: exercise the same retained creation-frame view used by
    /// incidence, then every origin lookup family, without creating effects.
    /// Reports actual validator entries and its existing charged work meter.
    pub fn output_origin_header_work_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        repetitions: usize,
    ) -> Result<(u64, u64), String> {
        assert!(ACTIVE.get().is_none(), "header recording cannot nest");
        ACTIVE.set(Some(Observation {
            commit: self.commit.commit_id,
            version: self.commit.version_id,
            validations: 0,
            work: 0,
        }));
        let _recording = Recording;
        execution
            .in_scope(|lease| {
                ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                    lease,
                    None,
                    |context| {
                        let view = self.validate_output_origins(context)?;
                        for _ in 0..repetitions {
                            for origin in self.output_origins() {
                                view.output_origin_for_route(&origin.meaning, context)?;
                                for &id in &origin.outputs {
                                    view.output_origin_for_entity(id, context)?;
                                }
                                for relation in &origin.relations {
                                    view.output_origin_for_relation(relation.id, context)?;
                                }
                            }
                        }
                        Ok::<_, crate::history::data::CodecRetirementClosureStop>(())
                    },
                )
            })
            .map_err(|stop| format!("{stop:?}"))?
            .map_err(|stop| format!("{stop:?}"))?
            .0
            .map_err(|stop| format!("{stop:?}"))?;
        let observation = ACTIVE.get().expect("recording stays armed until return");
        Ok((observation.validations, observation.work))
    }
}

pub(super) fn begin(
    envelope: &CanonicalCommitEnvelope,
    context: &MapKernelContext<'_, '_>,
) -> Option<u64> {
    ACTIVE
        .get()
        .filter(|o| {
            o.commit == envelope.commit.commit_id && o.version == envelope.commit.version_id
        })
        .map(|_| context.charged_work())
}

pub(super) fn finish(
    envelope: &CanonicalCommitEnvelope,
    context: &MapKernelContext<'_, '_>,
    before: Option<u64>,
) {
    if let Some(before) = before {
        ACTIVE.update(|observation| {
            observation.map(|mut o| {
                if o.commit == envelope.commit.commit_id && o.version == envelope.commit.version_id
                {
                    o.validations += 1;
                    o.work += context.charged_work() - before;
                }
                o
            })
        });
    }
}
