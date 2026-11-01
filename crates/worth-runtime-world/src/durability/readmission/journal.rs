//! Admit original streams into the same normalized, request-owned image tables.
use super::*;
use crate::durability::checkpoint::{
    WorldBranchImageKey, WorldImageHead, WorldOwnerRecordLocation,
};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) fn insert<K: Ord + Clone, V: Clone>(
    map: &mut ExecutionOrderedMap<K, V>,
    key: &K,
    value: &V,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), WorldReadmissionDenial> {
    map.insert_cloned_native_for_request(key, value, *execution)
        .map_err(|cause| WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Ordered(cause)))
}
impl WorldCheckpointImage {
    pub(in crate::durability) fn admit_journal_record(
        &mut self,
        journal: WorldDurableJournal,
        position: u64,
        digest: [u8; 32],
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        if bytes.len() > 64 << 10 {
            return Err(WorldReadmissionDenial::Medium(
                WorldDurableMediumDenial::Frame(
                    crate::durability::WorldDurableFrameDenial::SizeLimit,
                ),
            ));
        }
        execution
            .run_native_with_checkpoint(|context| context.checkpoint(bytes.len() as u64))
            .map_err(|cause| {
                WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(cause))
            })?
            .0
            .map_err(|cause| {
                WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(
                    worth_execution::WorkCeilingDenial::Stopped(cause),
                ))
            })?;
        execution.consult().map_err(|cause| {
            WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(cause))
        })?;
        if self.coverage.journal(journal).position.checked_add(1) != Some(position)
            || <[u8; 32]>::from(Sha256::digest(bytes)) != digest
        {
            return Err(WorldReadmissionDenial::JournalSourceMismatch);
        }
        let record =
            crate::durability::codec::decode(bytes, self.binding.owner().home(), execution)
                .map_err(WorldReadmissionDenial::Medium)?;
        if record.record.journal() != journal {
            return Err(WorldReadmissionDenial::WrongJournal);
        }
        if record.record.owner() != self.binding.owner() {
            return Err(WorldReadmissionDenial::ForeignOwner);
        }
        self.frontiers
            .include_record(&record.record)
            .map_err(WorldReadmissionDenial::IdentityExhaustion)?;
        self.admit_record(record, position, digest, execution)?;
        self.coverage.include(journal, position, digest)
    }
    pub(in crate::durability) fn admit_record(
        &mut self,
        record: std::sync::Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        match &record.record {
            WorldRecord::Owner { .. } => {
                if self.owner.is_some() {
                    return Err(WorldReadmissionDenial::DuplicateIdentity);
                }
                self.owner = Some(WorldOwnerRecordLocation { position, digest });
            }
            WorldRecord::Branch { head, .. } => {
                let key = WorldBranchImageKey::new(record.clone(), position, digest);
                if head.generation.get() != 0
                    || self.branches.contains_key(&key)
                    || self.history.contains_key(&head.commit)
                {
                    return Err(WorldReadmissionDenial::BranchLineage);
                }
                let image_head = WorldImageHead::branch(record.clone());
                insert(&mut self.branches, &key, &(), execution)?;
                insert(&mut self.heads, &key, &image_head, execution)?;
                insert(&mut self.history, &head.commit, &image_head, execution)?;
            }
            WorldRecord::OpenAttempt(open) => {
                let attempt = open.association.attempt();
                if self.opens.contains_key(&attempt) {
                    return Err(WorldReadmissionDenial::DuplicateIdentity);
                }
                let admitted = AdmittedWorldOpenAttempt::from_validated_journal(
                    record.clone(),
                    position,
                    digest,
                );
                insert(&mut self.opens, &attempt, &admitted, execution)?;
            }
            WorldRecord::Released {
                association,
                custody,
                relational,
            } => {
                self.admit_disposition(
                    association,
                    *custody,
                    RecoveredCustodyDisposition::Released(relational.clone()),
                    position,
                    digest,
                    execution,
                )?;
            }
            WorldRecord::Continued {
                association,
                custody,
            } => {
                self.admit_disposition(
                    association,
                    *custody,
                    RecoveredCustodyDisposition::Continued,
                    position,
                    digest,
                    execution,
                )?;
            }
            WorldRecord::Occurrence {
                open,
                relational,
                publishing_incarnation,
            } => {
                self.require_open(&open.association, open.custody)?;
                if self
                    .opens
                    .get(&open.association.attempt())
                    .map(|open| open.open())
                    != Some(open)
                {
                    return Err(WorldReadmissionDenial::ConflictingOpenAttempt);
                }
                self.admit_occurrence(
                    open.association.attempt(),
                    relational.clone(),
                    *publishing_incarnation,
                    position,
                    digest,
                    execution,
                )?;
            }
        }
        Ok(())
    }
    fn require_open(
        &self,
        association: &CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
    ) -> Result<(), WorldReadmissionDenial> {
        let open = self
            .opens
            .get(&association.attempt())
            .ok_or(WorldReadmissionDenial::MissingOpenAttempt)?;
        if open.association != *association || open.custody != custody {
            return Err(WorldReadmissionDenial::ConflictingOpenAttempt);
        }
        Ok(())
    }
    pub(in crate::durability) fn admit_disposition(
        &mut self,
        association: &CompositeAttemptAssociation,
        custody: DurableProductUnpublishedOwnerEffectsIdentity,
        value: RecoveredCustodyDisposition,
        position: u64,
        digest: [u8; 32],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        self.require_open(association, custody)?;
        let attempt = association.attempt();
        if position <= self.opens[&attempt].position() || self.dispositions.contains_key(&attempt) {
            return Err(WorldReadmissionDenial::ConflictingDisposition);
        }
        let row =
            AdmittedWorldDisposition::new(self.opens[&attempt].clone(), value, position, digest);
        insert(&mut self.dispositions, &attempt, &row, execution)
    }
    pub(in crate::durability) fn validate_open_identities(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let mut commits = ExecutionOrderedMap::default();
        let mut custody = ExecutionOrderedMap::default();
        for open in self.opens.values() {
            if let Some(published) = self.history.get(&open.commit) {
                if !published.represents_open(open.open()) {
                    return Err(WorldReadmissionDenial::DuplicateIdentity);
                }
                if matches!(
                    self.dispositions
                        .get(&open.association.attempt())
                        .map(|row| row.disposition()),
                    Some(RecoveredCustodyDisposition::Released(_))
                ) {
                    return Err(WorldReadmissionDenial::ConflictingDisposition);
                }
            }
            if commits.contains_key(&open.commit) || custody.contains_key(&open.custody) {
                return Err(WorldReadmissionDenial::DuplicateIdentity);
            }
            insert(&mut commits, &open.commit, &(), execution)?;
            insert(&mut custody, &open.custody, &(), execution)?;
            if self
                .history
                .get(&open.expected.commit)
                .is_none_or(|head| !head.matches(&open.expected))
            {
                return Err(WorldReadmissionDenial::MissingPredecessor);
            }
        }
        Ok(())
    }
}
