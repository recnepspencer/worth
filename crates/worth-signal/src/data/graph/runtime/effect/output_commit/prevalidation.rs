//! Output packet validation and work admission before storage staging.
#[cfg(test)]
use super::OutputCommitPacket;
use super::{
    EvaluationVerdict, EvaluationWork, OutputCommitStorage, PreparedDirectInvalidation,
    SignalError, SignalGraph,
};
impl SignalGraph {
    #[cfg(test)]
    pub(super) fn prevalidate_output_commit_packet(
        &self,
        packet: &OutputCommitPacket,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        self.prevalidate_output_commit_storage(&packet.storage, work)
    }

    pub(super) fn prevalidate_output_commit_storage(
        &self,
        packet: &OutputCommitStorage,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        let producer = packet.apply.effect.operational.node;
        self.admit_effect_node_mutation_work(work)?;
        self.validate_handle(producer)?;
        self.admit_effect_node_copy_work(producer, work)?;
        if super::super::vocabulary::verdict_transitions_clean(
            &packet.apply.effect.operational.verdict,
        ) {
            self.cause_sets
                .admit_release_work(self.node_pending_cause_set_id(producer)?, work)?;
        }
        if matches!(
            packet.apply.effect.operational.verdict,
            EvaluationVerdict::Recomputed
        ) {
            self.admit_node_aspect_evaluation_work(
                producer,
                packet.apply.effect.changed_regions(),
                work,
            )?;
        }
        self.admit_effect_branch_record_work(
            producer,
            packet
                .artifact_write
                .runtime
                .as_ref()
                .map(|runtime| &**runtime.reuse_basis()),
            work,
        )?;
        if let Some(snapshot) = packet.apply.pending_snapshot.as_ref() {
            if snapshot.node != producer {
                return Err(SignalError::internal(
                    "prepared output commit snapshot belongs to another producer",
                ));
            }
            self.validate_handle(snapshot.node)?;
        }
        if let Some(delta) = packet
            .prepared_direct
            .as_ref()
            .map(PreparedDirectInvalidation::delta)
        {
            if delta.producer != producer {
                return Err(SignalError::internal(
                    "prepared output delta belongs to another producer",
                ));
            }
        }
        if let Some(causes) = packet.direct_causes.as_ref() {
            causes.admit_node_and_waiter_publication_work(self, producer, work)?;
            causes.admit_cause_store_work(self, work)?;
            causes.validate_before_evaluation(
                self,
                packet.apply.effect.operational.aspect_version,
                packet.apply.effect.changed_regions(),
                work,
            )?;
            causes.validate_packet(
                producer,
                packet
                    .prepared_direct
                    .as_ref()
                    .map(PreparedDirectInvalidation::delta),
                work,
            )?;
        }
        if let Some(prepared) = packet.prepared_direct.as_ref() {
            // Both copies occur during non-fallible publication: one enters the
            // commit ledger and one enters performed invalidation evidence.
            // Only packets admitted here can reach that publication boundary.
            for _ in 0..2 {
                crate::logic::invalidation::causality::admit_delta_copy(prepared.delta(), work)?;
            }
        }
        Ok(())
    }
}
