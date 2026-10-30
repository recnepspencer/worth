use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::{MediaCounterSnapshot, MediaCounterTerminal};

#[derive(Debug, Default)]
struct FaultMatchState {
    first: Option<super::MediaOperationContext>,
    terminal: Option<MediaCounterTerminal>,
    completed_bytes: Option<u64>,
}

#[derive(Debug, Default)]
pub(super) struct MediaCounterCells {
    attempted_operations: AtomicU64,
    completed_operations: AtomicU64,
    denied_before_effect: AtomicU64,
    partial_effects: AtomicU64,
    indeterminate_effects: AtomicU64,
    requested_bytes: AtomicU64,
    completed_bytes: AtomicU64,
    eof_observations: AtomicU64,
    retry_attempts: AtomicU64,
    listing_batches: AtomicU64,
    listing_entries: AtomicU64,
    qualification_transactions: AtomicU64,
    ownership_attempts: AtomicU64,
    ownership_acquisitions: AtomicU64,
    ownership_contentions: AtomicU64,
    ownership_releases: AtomicU64,
    file_syncs: AtomicU64,
    directory_syncs: AtomicU64,
    replacements: AtomicU64,
    deletions: AtomicU64,
    pub(super) file_opens: AtomicU64,
    pub(super) file_creates: AtomicU64,
    pub(super) file_closes: AtomicU64,
    pub(super) live_file_handles: AtomicU64,
    pub(super) peak_file_handles: AtomicU64,
    pub(super) directory_opens: AtomicU64,
    pub(super) directory_closes: AtomicU64,
    pub(super) live_directory_handles: AtomicU64,
    pub(super) peak_directory_handles: AtomicU64,
    confinement_denials: AtomicU64,
    stale_handle_denials: AtomicU64,
    unsupported_capabilities: AtomicU64,
    cleanup_actions: AtomicU64,
    preserved_residue: AtomicU64,
    peak_request_width_bytes: AtomicU64,
    explicit_heap_allocation_events: AtomicU64,
    requested_heap_capacity_bytes: AtomicU64,
    fault_matches: AtomicU64,
    fault_match_state: Mutex<FaultMatchState>,
    role_attempts: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_identified_operation_attempts: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_completed_operations: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_denied_before_effect: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_partial_effects: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_indeterminate_effects: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_requested_bytes: [AtomicU64; super::MediaOperationRole::ALL.len()],
    role_completed_bytes: [AtomicU64; super::MediaOperationRole::ALL.len()],
}

impl MediaCounterCells {
    pub(super) fn begin(&self, role: super::MediaOperationRole, requested_bytes: u64) {
        increment(&self.attempted_operations, 1);
        increment(&self.requested_bytes, requested_bytes);
        increment(&self.role_attempts[role.index()], 1);
        increment(&self.role_requested_bytes[role.index()], requested_bytes);
        retain_max(&self.peak_request_width_bytes, requested_bytes);
    }

    pub(super) fn record_fault_match(&self, context: super::MediaOperationContext) {
        increment(&self.fault_matches, 1);
        let mut state = self
            .fault_match_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.first.is_none() {
            state.first = Some(context);
        }
    }

    pub(super) fn observe_operation_context(&self, context: super::MediaOperationContext) {
        if let Some(ordinal) = context.identified_operation_ordinal() {
            retain_max(
                &self.role_identified_operation_attempts[context.role().index()],
                ordinal,
            );
        }
    }

    pub(super) fn record_fault_terminal(
        &self,
        context: super::MediaOperationContext,
        terminal: MediaCounterTerminal,
        completed_bytes: u64,
    ) {
        let mut state = self
            .fault_match_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.first == Some(context) && state.terminal.is_none() {
            state.terminal = Some(terminal);
            state.completed_bytes = Some(completed_bytes);
        }
    }

    pub(super) fn completed(&self, role: super::MediaOperationRole, completed_bytes: u64) {
        increment(&self.completed_operations, 1);
        increment(&self.role_completed_operations[role.index()], 1);
        increment(&self.completed_bytes, completed_bytes);
        increment(&self.role_completed_bytes[role.index()], completed_bytes);
    }

    pub(super) fn denied(&self, role: super::MediaOperationRole) {
        increment(&self.denied_before_effect, 1);
        increment(&self.role_denied_before_effect[role.index()], 1);
    }

    pub(super) fn confinement_denied(&self, role: super::MediaOperationRole) {
        self.denied(role);
        increment(&self.confinement_denials, 1);
    }

    pub(super) fn stale_handle_denied(&self, role: super::MediaOperationRole) {
        self.denied(role);
        increment(&self.stale_handle_denials, 1);
    }

    pub(super) fn unsupported_capability(&self, role: super::MediaOperationRole) {
        self.denied(role);
        increment(&self.unsupported_capabilities, 1);
    }

    pub(super) fn partial(&self, role: super::MediaOperationRole, completed_bytes: u64) {
        increment(&self.partial_effects, 1);
        increment(&self.role_partial_effects[role.index()], 1);
        increment(&self.completed_bytes, completed_bytes);
        increment(&self.role_completed_bytes[role.index()], completed_bytes);
    }

    pub(super) fn indeterminate(&self, role: super::MediaOperationRole, completed_bytes: u64) {
        increment(&self.indeterminate_effects, 1);
        increment(&self.role_indeterminate_effects[role.index()], 1);
        increment(&self.completed_bytes, completed_bytes);
        increment(&self.role_completed_bytes[role.index()], completed_bytes);
    }

    pub(super) fn eof_observation(&self) {
        increment(&self.eof_observations, 1);
    }

    pub(super) fn retry_attempt(&self) {
        increment(&self.retry_attempts, 1);
    }

    pub(super) fn listing_batch(&self, entries: usize) {
        increment(&self.listing_batches, 1);
        increment(&self.listing_entries, entries as u64);
    }

    pub(super) fn explicit_heap_allocation(&self, requested_capacity_bytes: usize) {
        increment(&self.explicit_heap_allocation_events, 1);
        increment(
            &self.requested_heap_capacity_bytes,
            requested_capacity_bytes as u64,
        );
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(super) fn qualification_transaction(&self) {
        increment(&self.qualification_transactions, 1);
    }

    pub(super) fn ownership_attempt(&self) {
        increment(&self.ownership_attempts, 1);
    }

    pub(super) fn ownership_acquired(&self) {
        increment(&self.ownership_acquisitions, 1);
    }

    pub(super) fn ownership_contended(&self) {
        increment(&self.ownership_contentions, 1);
    }

    pub(super) fn ownership_released(&self) {
        increment(&self.ownership_releases, 1);
    }

    pub(super) fn file_sync(&self) {
        increment(&self.file_syncs, 1);
    }

    pub(super) fn directory_sync(&self) {
        increment(&self.directory_syncs, 1);
    }

    pub(super) fn replacement(&self) {
        increment(&self.replacements, 1);
    }

    pub(super) fn deletion(&self) {
        increment(&self.deletions, 1);
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(super) fn cleanup_action(&self) {
        increment(&self.cleanup_actions, 1);
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(super) fn preserve_residue(&self) {
        increment(&self.preserved_residue, 1);
    }

    pub(super) fn snapshot(&self) -> MediaCounterSnapshot {
        let load = |cell: &AtomicU64| cell.load(Ordering::Acquire);
        let role_attempts = std::array::from_fn(|index| load(&self.role_attempts[index]));
        let role_completed_operations =
            std::array::from_fn(|index| load(&self.role_completed_operations[index]));
        let role_identified_operation_attempts =
            std::array::from_fn(|index| load(&self.role_identified_operation_attempts[index]));
        let role_denied_before_effect =
            std::array::from_fn(|index| load(&self.role_denied_before_effect[index]));
        let role_partial_effects =
            std::array::from_fn(|index| load(&self.role_partial_effects[index]));
        let role_indeterminate_effects =
            std::array::from_fn(|index| load(&self.role_indeterminate_effects[index]));
        let role_requested_bytes =
            std::array::from_fn(|index| load(&self.role_requested_bytes[index]));
        let role_completed_bytes =
            std::array::from_fn(|index| load(&self.role_completed_bytes[index]));
        let fault_match = self
            .fault_match_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut snapshot = MediaCounterSnapshot {
            attempted_operations: load(&self.attempted_operations),
            completed_operations: load(&self.completed_operations),
            denied_before_effect: load(&self.denied_before_effect),
            partial_effects: load(&self.partial_effects),
            indeterminate_effects: load(&self.indeterminate_effects),
            requested_bytes: load(&self.requested_bytes),
            completed_bytes: load(&self.completed_bytes),
            eof_observations: load(&self.eof_observations),
            retry_attempts: load(&self.retry_attempts),
            listing_batches: load(&self.listing_batches),
            listing_entries: load(&self.listing_entries),
            qualification_transactions: load(&self.qualification_transactions),
            ownership_attempts: load(&self.ownership_attempts),
            ownership_acquisitions: load(&self.ownership_acquisitions),
            ownership_contentions: load(&self.ownership_contentions),
            ownership_releases: load(&self.ownership_releases),
            file_syncs: load(&self.file_syncs),
            directory_syncs: load(&self.directory_syncs),
            replacements: load(&self.replacements),
            deletions: load(&self.deletions),
            file_opens: load(&self.file_opens),
            file_creates: load(&self.file_creates),
            file_closes: load(&self.file_closes),
            live_file_handles: load(&self.live_file_handles),
            peak_file_handles: load(&self.peak_file_handles),
            directory_opens: load(&self.directory_opens),
            directory_closes: load(&self.directory_closes),
            live_directory_handles: load(&self.live_directory_handles),
            peak_directory_handles: load(&self.peak_directory_handles),
            confinement_denials: load(&self.confinement_denials),
            stale_handle_denials: load(&self.stale_handle_denials),
            unsupported_capabilities: load(&self.unsupported_capabilities),
            cleanup_actions: load(&self.cleanup_actions),
            preserved_residue: load(&self.preserved_residue),
            peak_request_width_bytes: load(&self.peak_request_width_bytes),
            explicit_heap_allocation_events: load(&self.explicit_heap_allocation_events),
            requested_heap_capacity_bytes: load(&self.requested_heap_capacity_bytes),
            fault_matches: load(&self.fault_matches),
            first_fault_match: fault_match.first,
            first_fault_terminal: fault_match.terminal,
            first_fault_completed_bytes: fault_match.completed_bytes,
            saturated: false,
            role_attempts,
            role_identified_operation_attempts,
            role_completed_operations,
            role_denied_before_effect,
            role_partial_effects,
            role_indeterminate_effects,
            role_requested_bytes,
            role_completed_bytes,
        };
        snapshot.mark_saturation();
        snapshot
    }
}

pub(super) fn increment(cell: &AtomicU64, amount: u64) {
    let _ = {
        let update_counter = cell;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = Some(current.saturating_add(amount));
            let Some(updated_value) = next_value else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    };
}

fn retain_max(cell: &AtomicU64, candidate: u64) {
    let _ = {
        let update_counter = cell;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = (candidate > current).then_some(candidate);
            let Some(updated_value) = next_value else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    };
}
