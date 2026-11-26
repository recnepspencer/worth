//! Only the exact original medium's terminal witness changes the semantic fold.
use super::*;
use crate::domain_computation::application_aftermath::correction_material::QueryCorrectionMaterialPublicationOutcome as Outcome;

impl MaterialPublication<'_> {
    pub fn publish(
        &self,
        record: MaterialRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        self.publish_with_ordinal(record, None, request)
    }
    pub(super) fn publish_with_ordinal(
        &self,
        record: MaterialRecord,
        pending_ordinal: Option<(
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
            u64,
        )>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let medium = self.owner.medium()?;
        self.finish_pending(medium.as_ref(), request)?;
        {
            let mut state = self.owner.state();
            let generation = state
                .allocated_generation
                .checked_add(1)
                .and_then(std::num::NonZeroU64::new)
                .ok_or(Denial::IdentityExhausted)?;
            state.pins.validate(&record)?;
            let frame = state.catalog.frame(generation, &record)?;
            state.allocated_generation = generation.get();
            let retained = (frame.bytes().len() as u64)
                .checked_mul(4)
                .ok_or(Denial::Allocation)?;
            state.funding.retain(
                request
                    .reserve_retained_memory(retained)
                    .map_err(Denial::Execution)?,
            );
            state.pending = Some(Arc::new(frame));
            state.pending_world_ordinal = pending_ordinal;
        }
        self.finish_pending(medium.as_ref(), request)
    }
    pub(super) fn finish_pending(
        &self,
        medium: &dyn QueryCorrectionsMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let Some(frame) = self.owner.state().pending.clone() else {
            return Ok(());
        };
        loop {
            match medium.advance_publication(request, &frame) {
                Outcome::Committed(proof) => {
                    return proof.consume(|committed| {
                        Self::admit_subject(medium, &frame, committed.subject())?;
                        let mut state = self.owner.state();
                        let (change, terminal) = state.catalog.fold_with_terminal(&frame)?;
                        state.pins.committed(change, medium.authority());
                        state.pending = None;
                        if let Some((original, ordinal)) = state.pending_world_ordinal.take() {
                            state.world_ordinals.insert(original, ordinal);
                            self.owner.retain_world_ordinal(ordinal);
                        }
                        if let Some(ordinal) =
                            terminal.and_then(|original| state.world_ordinals.remove(&original))
                        {
                            self.owner.release_world_ordinal(ordinal);
                        }
                        Ok(())
                    });
                }
                Outcome::ProvenNoEffect(proof) => {
                    Self::admit_subject(medium, &frame, proof.subject())?;
                    medium.acknowledge_no_effect(proof)?;
                    let mut state = self.owner.state();
                    state.pending = None;
                    state.pending_world_ordinal = None;
                    return Err(Denial::PublicationNoEffect);
                }
                Outcome::Pending => {
                    request.consult().map_err(Denial::Execution)?;
                    std::thread::yield_now();
                }
                Outcome::Refused(cause) => return Err(cause),
                Outcome::Unresolved => return Err(Denial::PublicationUnresolved),
            }
        }
    }
    fn admit_subject(
        medium: &dyn QueryCorrectionsMedium,
        frame: &QueryCorrectionMaterialFrame,
        subject: &worth_proof::HomeQueryCorrectionSubject,
    ) -> Result<(), Denial> {
        subject
            .admit_to(medium.authority())
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignHome,
                worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            })?;
        if subject.position() != frame.position()
            || subject.payload_digest() != frame.payload_digest()
        {
            return Err(Denial::PrefixMismatch);
        }
        Ok(())
    }
}
