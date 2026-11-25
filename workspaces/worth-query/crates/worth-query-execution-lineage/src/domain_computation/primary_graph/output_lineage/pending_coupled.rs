//! Retryable owner-side pending coupled-output durable retention queue.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity;

#[derive(Clone)]
pub struct PendingCoupledOutputRecord {
    pub commit: worth_foundational::facade::DurableCompositeCommitIdentity,
    pub branch: String,
    pub identity: WorthQueryAcceptedOutputCheckpointIdentity,
}

impl PendingCoupledOutputRecord {
    pub fn matches_slot(&self, other: &Self) -> bool {
        self.branch == other.branch
            && self.identity.producer == other.identity.producer
            && self.identity.scope == other.identity.scope
            && self.identity.source_partition == other.identity.source_partition
    }

    pub fn matches_exact(&self, other: &Self) -> bool {
        self.commit == other.commit && self.matches_slot(other)
    }
}

impl WorthQueryApplicationOutputLineage {
    /// Extract coupled member checkpoint identities from a committed World
    /// publication and enqueue them monotonically in owner custody before the
    /// commit answer is delivered to any caller.
    pub fn enqueue_coupled_outputs_from_committed(
        &mut self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        branch: &str,
        committed: &WorthQueryPrimaryGraphCommittedApplication,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
        let mut pending = Vec::new();
        for settlement in committed
            .publication_members()
            .filter(|member| member.is_coupled_member())
            .filter_map(|member| member.exact_output_settlement())
        {
            if let Some(identity) = self.checkpoint_identity_for_settlement(settlement)? {
                pending.push(PendingCoupledOutputRecord {
                    commit,
                    branch: branch.to_owned(),
                    identity,
                });
            }
        }
        self.enqueue_pending_coupled_outputs(pending);
        Ok(())
    }

    /// Record coupled outputs committed by World in retryable owner custody so
    /// an interruption or lost commit answer never drops unwritten members.
    /// Entries for the same `(branch, producer, scope, partition)` advance
    /// monotonically by World commit identity so a delayed older commit cannot
    /// overwrite a newer same-slot commit.
    pub fn enqueue_pending_coupled_outputs(&mut self, records: Vec<PendingCoupledOutputRecord>) {
        for record in records {
            if let Some(existing) = self
                .pending_coupled
                .iter_mut()
                .find(|entry| entry.matches_slot(&record))
            {
                if existing.commit.owner().home() == record.commit.owner().home()
                    && existing.commit.ordinal() > record.commit.ordinal()
                {
                    continue;
                }
                *existing = record;
            } else {
                self.pending_coupled.push(record);
            }
        }
    }

    pub fn peek_pending_coupled_output(&self) -> Option<PendingCoupledOutputRecord> {
        self.pending_coupled.first().cloned()
    }

    pub fn complete_pending_coupled_output(&mut self, completed: &PendingCoupledOutputRecord) {
        if self
            .pending_coupled
            .first()
            .is_some_and(|head| head.matches_exact(completed))
        {
            self.pending_coupled.remove(0);
        }
    }

    pub fn has_pending_coupled_outputs(&self) -> bool {
        !self.pending_coupled.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
    use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointPosture;
    use worth_foundational::facade::{
        ApplicationBuildIdentity, DurableCompositeCommitIdentity, DurableWorldOwnerIdentity,
        HomeIdentity, WorldOwnerSlotIdentity,
    };
    use worth_relational::facade::identity::{EntityId, PartitionId};
    use worth_relational::facade::mvcc::RequiredOutputProducingImplementation;

    fn record(
        owner: DurableWorldOwnerIdentity,
        ordinal: u64,
        producer: &str,
        key_byte: u8,
    ) -> PendingCoupledOutputRecord {
        PendingCoupledOutputRecord {
            commit: DurableCompositeCommitIdentity::from_recorded(owner, ordinal),
            branch: "main".to_owned(),
            identity: WorthQueryAcceptedOutputCheckpointIdentity {
                implementation: RequiredOutputProducingImplementation::new(
                    ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111")
                        .unwrap(),
                    [1; 32],
                ),
                producer: Arc::new(producer.to_owned()),
                posture: WorthQueryAcceptedOutputCheckpointPosture::StableReused,
                source: [2; 32],
                scope: WorthQueryOperationScopeEntityBinding::from_entity(EntityId::new(
                    PartitionId(1),
                    2,
                    3,
                )),
                source_partition: [7; 32],
                producer_dependency: None,
                idempotency_key: [key_byte; 32],
                roles: Arc::new(Vec::new()),
                producer_facts: None,
                producer_fact_wire_version: 0,
                metadata: None,
            },
        }
    }

    #[test]
    fn delayed_older_pending_coupled_record_cannot_overwrite_newer_same_slot_commit() {
        let home = HomeIdentity::from_observed_bytes([0x22; 16]).unwrap();
        let owner = DurableWorldOwnerIdentity::from_recorded(
            home,
            WorldOwnerSlotIdentity::from_recorded(1),
        );
        let mut lineage = WorthQueryApplicationOutputLineage::default();

        lineage.enqueue_pending_coupled_outputs(vec![record(owner, 5, "producer.a", 5)]);
        lineage.enqueue_pending_coupled_outputs(vec![record(owner, 3, "producer.a", 3)]);

        let head = lineage
            .peek_pending_coupled_output()
            .expect("newer commit remains queued");
        assert_eq!(head.commit.ordinal(), 5);
        assert_eq!(head.identity.idempotency_key, [5; 32]);

        // Newer commit for the same slot replaces ordinal 5.
        lineage.enqueue_pending_coupled_outputs(vec![record(owner, 7, "producer.a", 7)]);
        let head = lineage
            .peek_pending_coupled_output()
            .expect("newest commit replaces older queued commit");
        assert_eq!(head.commit.ordinal(), 7);
        assert_eq!(head.identity.idempotency_key, [7; 32]);

        // Independent producer slot is preserved alongside ordinal 7.
        lineage.enqueue_pending_coupled_outputs(vec![record(owner, 2, "producer.b", 2)]);
        assert_eq!(lineage.pending_coupled.len(), 2);
        lineage.complete_pending_coupled_output(&head);
        let second = lineage
            .peek_pending_coupled_output()
            .expect("second slot remains after first pops");
        assert_eq!(second.identity.producer.as_str(), "producer.b");
        assert_eq!(second.commit.ordinal(), 2);
    }
}
