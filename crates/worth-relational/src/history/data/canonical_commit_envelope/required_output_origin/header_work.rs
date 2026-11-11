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

impl CanonicalCommitEnvelopeBody {
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
        let bytes = copied_roster_bytes(event)?;
        let _copy_memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| format!("{cause:?}"))?;
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
        let event = CanonicalAuthoredRestoration {
            _decoded_memory: event._decoded_memory.clone(),
            retirement: event.retirement,
            original_anchor: event.original_anchor,
            anchor: event.anchor,
            co_anchors: event.co_anchors.clone(),
            entities,
            relations: event.relations.clone(),
            publications: event.publications.clone(),
        };
        // This is the ordinary header validator with descriptive corruption,
        // never a cloned body, Original frame, or newly issued publication.
        self.observe_header_work(execution, |context| {
            self.validate_authored_restoration_components(&event, false, context)
        })
    }

    /// Court door: exercise the same retained creation-frame view used by
    /// incidence, then every origin lookup family, without creating effects.
    /// Reports actual validator entries and its existing charged work meter.
    pub fn output_origin_header_work_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        repetitions: usize,
    ) -> Result<(u64, u64), String> {
        self.observe_header_work(execution, |context| {
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
        })
    }

    fn observe_header_work(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        validate: impl FnOnce(
            &mut MapKernelContext<'_, '_>,
        ) -> Result<(), crate::history::data::CodecRetirementClosureStop>,
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
                ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, validate)
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
    envelope: &CanonicalCommitEnvelopeBody,
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
    envelope: &CanonicalCommitEnvelopeBody,
    context: &MapKernelContext<'_, '_>,
    before: Option<u64>,
) {
    if let Some(before) = before {
        ACTIVE.set(ACTIVE.get().map(|mut o| {
            if o.commit == envelope.commit.commit_id && o.version == envelope.commit.version_id {
                o.validations += 1;
                o.work += context.charged_work() - before;
            }
            o
        }));
    }
}

fn copied_roster_bytes(event: &CanonicalAuthoredRestoration) -> Result<u64, String> {
    use crate::identity::data::RelationId;
    let overflow = || "restoration observation copy layout overflow".to_owned();
    let entities = event
        .entities
        .len()
        .checked_add(event.co_anchors.len())
        .ok_or_else(overflow)?;
    let entity_bytes = std::alloc::Layout::array::<(EntityId, EntityId)>(entities)
        .map_err(|_| overflow())?
        .size();
    let relation_bytes =
        std::alloc::Layout::array::<(RelationId, RelationId)>(event.relations.len())
            .map_err(|_| overflow())?
            .size();
    let publication_bytes = std::alloc::Layout::array::<
        std::sync::Arc<CanonicalRequiredOutputOrigin>,
    >(event.publications.len())
    .map_err(|_| overflow())?
    .size();
    let bytes = entity_bytes
        .checked_add(relation_bytes)
        .and_then(|n| n.checked_add(publication_bytes))
        .ok_or_else(overflow)?;
    u64::try_from(bytes).map_err(|_| overflow())
}
