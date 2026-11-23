//! Collect and validate durable branch and history retirements in structural order.
use super::*;
use crate::durability::checkpoint::WorldBranchImageKey;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableProductBranchIncarnation, HomeIncarnation,
};

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldBranchRetirement {
    record: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldBranchRetirement {
    pub(super) fn from_validated_journal(
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            record,
            position,
            digest,
        }
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.record
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub(crate) fn head(&self) -> &crate::durability::DurableWorldHead {
        match &self.record.record {
            WorldRecord::BranchRetired { head, .. } => head,
            _ => unreachable!("branch retirement record"),
        }
    }
    pub(crate) fn retiring_incarnation(&self) -> HomeIncarnation {
        match &self.record.record {
            WorldRecord::BranchRetired {
                retiring_incarnation,
                ..
            } => *retiring_incarnation,
            _ => unreachable!("branch retirement record"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldHistoryRetirement {
    record: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldHistoryRetirement {
    pub(super) fn from_validated_journal(
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            record,
            position,
            digest,
        }
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.record
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub(crate) fn commits(&self) -> &[DurableCompositeCommitIdentity] {
        match &self.record.record {
            WorldRecord::HistoryRetired { commits, .. } => commits,
            _ => unreachable!("history retirement record"),
        }
    }
    pub(crate) fn retiring_incarnation(&self) -> HomeIncarnation {
        match &self.record.record {
            WorldRecord::HistoryRetired {
                retiring_incarnation,
                ..
            } => *retiring_incarnation,
            _ => unreachable!("history retirement record"),
        }
    }
}

impl WorldCheckpointImage {
    pub(in crate::durability) fn collect_branch_retirement(
        &mut self,
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let WorldRecord::BranchRetired {
            head,
            retiring_incarnation,
        } = &record.record
        else {
            unreachable!("branch retirement class")
        };
        if *retiring_incarnation > self.binding.incarnation() {
            return Err(WorldReadmissionDenial::FutureIncarnation);
        }
        let incarnation = head.incarnation;
        if self.retired_branches.contains_key(&incarnation) {
            return Err(WorldReadmissionDenial::DuplicateIdentity);
        }
        let coordinate = (incarnation, head.generation);
        let row = AdmittedWorldBranchRetirement::from_validated_journal(record, position, digest);
        super::journal::insert(&mut self.retired_branches, &incarnation, &row, request)?;
        self.queue_retirement_step(coordinate, incarnation, request)
    }

    pub(in crate::durability) fn collect_history_retirement(
        &mut self,
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let WorldRecord::HistoryRetired {
            commits,
            retiring_incarnation,
            ..
        } = &record.record
        else {
            unreachable!("history retirement class")
        };
        if *retiring_incarnation > self.binding.incarnation() {
            return Err(WorldReadmissionDenial::FutureIncarnation);
        }
        if commits.is_empty() {
            return Err(WorldReadmissionDenial::BranchLineage);
        }
        if self.history_retirements.contains_key(&position) {
            return Err(WorldReadmissionDenial::JournalSourceMismatch);
        }
        let row = AdmittedWorldHistoryRetirement::from_validated_journal(record, position, digest);
        super::journal::insert(&mut self.history_retirements, &position, &row, request)?;
        super::journal::insert(
            &mut self.collected_history_retirements,
            &position,
            &(),
            request,
        )
    }

    pub(super) fn admit_collected_branch_retirement(
        &mut self,
        incarnation: DurableProductBranchIncarnation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        let retired = self
            .retired_branches
            .get(&incarnation)
            .expect("collected branch retirement")
            .clone();
        let head = retired.head();
        let key =
            WorldBranchImageKey::new(retired.record().clone(), retired.position(), *retired.digest());
        let current = self
            .heads
            .get(&key)
            .ok_or(WorldReadmissionDenial::MissingPredecessor)?;
        if !current.matches(head) || retired.position() <= current.origin_position() {
            return Err(WorldReadmissionDenial::BranchLineage);
        }
        self.heads.remove_for_request(&key, *request).map_err(|cause| {
            WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Ordered(cause))
        })?;
        Ok(())
    }

    pub(super) fn resolve_collected_history_retirements(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldReadmissionDenial> {
        for index in 0..self.collected_history_retirements.len() {
            let position = *self
                .collected_history_retirements
                .entry_at(index)
                .expect("collected history retirement")
                .0;
            request.consult().map_err(|cause| {
                WorldReadmissionDenial::Medium(WorldDurableMediumDenial::Request(cause))
            })?;
            let row = self
                .history_retirements
                .get(&position)
                .expect("history retirement row")
                .clone();
            for commit in row.commits() {
                if !self.history.contains_key(commit) {
                    return Err(WorldReadmissionDenial::MissingPredecessor);
                }
                if self.retired_history.contains_key(commit) {
                    return Err(WorldReadmissionDenial::DuplicateIdentity);
                }
                super::journal::insert(&mut self.retired_history, commit, &(), request)?;
            }
        }
        self.collected_history_retirements = Default::default();
        for head in self.heads.values() {
            if self.retired_history.contains_key(&head.fields().2) {
                return Err(WorldReadmissionDenial::BranchLineage);
            }
        }
        Ok(())
    }
}
