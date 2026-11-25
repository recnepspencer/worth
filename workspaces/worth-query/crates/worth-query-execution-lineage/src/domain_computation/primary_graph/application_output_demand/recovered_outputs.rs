//! Exact restored-output locators; reconstructed storage keeps its bootstrap payer.
use super::WorthQueryReadmittedAcceptedOutput;
use crate::domain_computation::{
    authorization::WorthQueryOperationScopeEntityBinding,
    primary_graph::application_checkpoint::{CheckpointDecodeDenial, CheckpointMetadataFunding},
};

struct ObligationBasisPinSlot<O> {
    _observation: O,
    in_flight: usize,
}

pub(crate) struct PinTable<O> {
    slots: std::sync::Arc<
        std::sync::Mutex<
            std::collections::BTreeMap<
                worth_foundational::facade::DurableCompositeCommitIdentity,
                ObligationBasisPinSlot<O>,
            >,
        >,
    >,
}

impl<O> Clone for PinTable<O> {
    fn clone(&self) -> Self {
        Self {
            slots: std::sync::Arc::clone(&self.slots),
        }
    }
}

impl<O> Default for PinTable<O> {
    fn default() -> Self {
        Self {
            slots: std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
        }
    }
}

impl<O> PinTable<O> {
    pub(crate) fn has_pin(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
    ) -> bool {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains_key(&commit)
    }

    pub(crate) fn pin_durable(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        observation: O,
    ) {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(commit)
            .or_insert(ObligationBasisPinSlot {
                _observation: observation,
                in_flight: 0,
            });
    }

    pub(crate) fn try_increment_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
    ) -> bool {
        let mut slots = self
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(slot) = slots.get_mut(&commit) else {
            return false;
        };
        slot.in_flight += 1;
        true
    }

    pub(crate) fn insert_and_increment_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        observation: O,
    ) {
        let mut slots = self
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = slots.entry(commit).or_insert(ObligationBasisPinSlot {
            _observation: observation,
            in_flight: 0,
        });
        slot.in_flight += 1;
    }

    pub(crate) fn release_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        mut still_active: impl FnMut(worth_foundational::facade::DurableCompositeCommitIdentity) -> bool,
    ) {
        let mut slots = self
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(slot) = slots.get_mut(&commit) {
            slot.in_flight = slot.in_flight.saturating_sub(1);
            if slot.in_flight == 0 && !still_active(commit) {
                slots.remove(&commit);
            }
        }
    }

    pub(crate) fn prune(
        &self,
        mut still_active: impl FnMut(worth_foundational::facade::DurableCompositeCommitIdentity) -> bool,
    ) {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|commit, slot| slot.in_flight > 0 || still_active(*commit));
    }

    pub(crate) fn clear(&self) {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

/// Shared pin custody for durably retained output demands and sources.
/// In-flight guards prevent concurrent pruners from dropping a pin before its
/// journal frame is staged or committed.
#[derive(Clone, Default)]
pub struct ObligationBasisPins {
    table: PinTable<worth_runtime_world::facade::ProductBranchObservation>,
}

impl ObligationBasisPins {
    pub fn has_pin(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
    ) -> bool {
        self.table.has_pin(commit)
    }

    pub fn pin_durable(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        observation: worth_runtime_world::facade::ProductBranchObservation,
    ) {
        self.table.pin_durable(commit, observation);
    }

    pub fn try_retain_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        history: std::sync::Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    ) -> Option<ObligationBasisPinGuard> {
        self.table
            .try_increment_in_flight(commit)
            .then(|| ObligationBasisPinGuard {
                pins: self.clone(),
                history,
                commit,
            })
    }

    pub fn pin_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        observation: worth_runtime_world::facade::ProductBranchObservation,
        history: std::sync::Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    ) -> ObligationBasisPinGuard {
        self.table
            .insert_and_increment_in_flight(commit, observation);
        ObligationBasisPinGuard {
            pins: self.clone(),
            history,
            commit,
        }
    }

    fn release_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        still_active: impl FnMut(worth_foundational::facade::DurableCompositeCommitIdentity) -> bool,
    ) {
        self.table.release_in_flight(commit, still_active);
    }

    pub fn prune(
        &self,
        still_active: impl FnMut(worth_foundational::facade::DurableCompositeCommitIdentity) -> bool,
    ) {
        self.table.prune(still_active);
    }

    pub fn clear(&self) {
        self.table.clear();
    }
}

/// RAII custody keeping a basis pin resident across an in-flight obligation
/// publication until the frame is staged/acknowledged or proven no-effect.
pub struct ObligationBasisPinGuard {
    pins: ObligationBasisPins,
    history: std::sync::Arc<
        crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
    >,
    commit: worth_foundational::facade::DurableCompositeCommitIdentity,
}

impl Drop for ObligationBasisPinGuard {
    fn drop(&mut self) {
        self.pins.release_in_flight(self.commit, |commit| {
            self.history.has_active_output_basis(commit)
        });
    }
}

#[derive(Default)]
pub struct WorthQueryRecoveredOutputs {
    checkpoint: RecoveredOutputRecords,
    journal: std::sync::OnceLock<RecoveredOutputRecords>,
    obligation_basis_pins: ObligationBasisPins,
}
#[derive(Default)]
struct RecoveredOutputRecords {
    records: Vec<WorthQueryReadmittedAcceptedOutput>,
    by_source_partition: Vec<usize>,
    // Declared last: every retained row/string/role/wire and index drops first.
    _metadata: Option<CheckpointMetadataFunding>,
}
impl WorthQueryRecoveredOutputs {
    /// Journal selectors are a rebuilt cache. After World seal they must stop
    /// retaining the old product image before Native closes its read custody.
    pub fn retire_home_observations(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::super::WorthQueryHomeRetirementDenial> {
        for record in self.checkpoint.records.iter_mut().chain(
            self.journal
                .get_mut()
                .into_iter()
                .flat_map(|rows| rows.records.iter_mut()),
        ) {
            request
                .consult()
                .map_err(super::super::WorthQueryHomeRetirementDenial::Execution)?;
            record.observation = None;
        }
        self.obligation_basis_pins.clear();
        Ok(())
    }
    pub fn basis_pins_handle(&self) -> ObligationBasisPins {
        self.obligation_basis_pins.clone()
    }
    /// Pin the World basis commit for a durably retained output demand or source
    /// obligation so automatic history retirement does not dispose it before
    /// readmission or discharge. Released on discharge and on Home close.
    pub fn has_obligation_basis_pin(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
    ) -> bool {
        self.obligation_basis_pins.has_pin(commit)
    }
    pub fn pin_obligation_basis(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        observation: worth_runtime_world::facade::ProductBranchObservation,
    ) {
        self.obligation_basis_pins.pin_durable(commit, observation);
    }
    pub fn try_retain_obligation_basis_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        history: std::sync::Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    ) -> Option<ObligationBasisPinGuard> {
        self.obligation_basis_pins
            .try_retain_in_flight(commit, history)
    }
    pub fn pin_obligation_basis_in_flight(
        &self,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        observation: worth_runtime_world::facade::ProductBranchObservation,
        history: std::sync::Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    ) -> ObligationBasisPinGuard {
        self.obligation_basis_pins
            .pin_in_flight(commit, observation, history)
    }
    pub fn prune_obligation_basis_pins(
        &self,
        still_active: impl FnMut(worth_foundational::facade::DurableCompositeCommitIdentity) -> bool,
    ) {
        self.obligation_basis_pins.prune(still_active);
    }
    pub fn readmit_journal(
        &self,
        records: Vec<WorthQueryReadmittedAcceptedOutput>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        if self.journal.get().is_some() {
            return Ok(());
        }
        let index = RecoveredOutputRecords::from_records(
            records,
            CheckpointMetadataFunding::new(execution)?,
            execution,
        )?;
        self.journal
            .set(index)
            .map_err(|_| "accepted output journal already admitted".to_owned().into())
    }
    pub fn iter(&self) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        self.checkpoint.records.iter().chain(
            self.journal
                .get()
                .into_iter()
                .flat_map(|rows| rows.records.iter()),
        )
    }
    pub fn matching_source_partition(
        &self,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        self.journal
            .get()
            .into_iter()
            .flat_map(move |rows| rows.matching_source_partition(scope, partition))
            .chain(self.checkpoint.matching_source_partition(scope, partition))
    }
}
impl RecoveredOutputRecords {
    fn from_records(
        records: Vec<WorthQueryReadmittedAcceptedOutput>,
        mut metadata: CheckpointMetadataFunding,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        let mut by_source_partition = metadata.vec::<usize>(records.len(), execution)?;
        by_source_partition.extend(0..records.len());
        by_source_partition.sort_unstable_by_key(|slot| {
            (
                records[*slot].checkpoint.scope,
                records[*slot].checkpoint.source_partition,
            )
        });
        metadata.check_live(execution)?;
        Ok(Self {
            records,
            by_source_partition,
            _metadata: Some(metadata),
        })
    }
    pub fn matching_source_partition(
        &self,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        let key = (scope, partition);
        let start = self.by_source_partition.partition_point(|slot| {
            (
                self.records[*slot].checkpoint.scope,
                self.records[*slot].checkpoint.source_partition,
            ) < key
        });
        self.by_source_partition[start..]
            .iter()
            .map(|slot| &self.records[*slot])
            .take_while(move |output| {
                (output.checkpoint.scope, output.checkpoint.source_partition) == key
            })
    }
}

#[cfg(test)]
mod tests;
