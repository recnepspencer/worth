use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::{
    PhysicalSubmissionState, PhysicalWorkCapacityDimension, PhysicalWorkSubmissionDeferred,
};

impl PhysicalSubmissionState {
    pub(super) fn reserve(
        self: &Arc<Self>,
        scope_members: usize,
        semantic_bytes: usize,
    ) -> Result<SubmissionReservation, PhysicalWorkSubmissionDeferred> {
        if scope_members > self.capacity.scope_members_per_work() {
            return Err(deferred(
                PhysicalWorkCapacityDimension::ScopeMembersPerWork,
                self.capacity.scope_members_per_work(),
            ));
        }
        if semantic_bytes > self.capacity.semantic_bytes_per_work() {
            return Err(deferred(
                PhysicalWorkCapacityDimension::SemanticBytesPerWork,
                self.capacity.semantic_bytes_per_work(),
            ));
        }
        reserve_counter(&self.reserved_commands, 1, self.capacity.commands()).map_err(|_| {
            deferred(
                PhysicalWorkCapacityDimension::Commands,
                self.commands.capacity(),
            )
        })?;
        if reserve_counter(
            &self.reserved_scope_members,
            scope_members,
            self.capacity.total_scope_members(),
        )
        .is_err()
        {
            self.reserved_commands.fetch_sub(1, Ordering::AcqRel);
            return Err(deferred(
                PhysicalWorkCapacityDimension::TotalScopeMembers,
                self.capacity.total_scope_members(),
            ));
        }
        if reserve_counter(
            &self.reserved_semantic_bytes,
            semantic_bytes,
            self.capacity.total_semantic_bytes(),
        )
        .is_err()
        {
            self.reserved_scope_members
                .fetch_sub(scope_members, Ordering::AcqRel);
            self.reserved_commands.fetch_sub(1, Ordering::AcqRel);
            return Err(deferred(
                PhysicalWorkCapacityDimension::TotalSemanticBytes,
                self.capacity.total_semantic_bytes(),
            ));
        }
        Ok(SubmissionReservation {
            state: Arc::clone(self),
            scope_members,
            semantic_bytes,
            committed: false,
        })
    }

    pub(super) fn release_capacity(&self, scope_members: usize, semantic_bytes: usize) {
        self.reserved_semantic_bytes
            .fetch_sub(semantic_bytes, Ordering::AcqRel);
        self.reserved_scope_members
            .fetch_sub(scope_members, Ordering::AcqRel);
        self.reserved_commands.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(super) struct SubmissionReservation {
    state: Arc<PhysicalSubmissionState>,
    scope_members: usize,
    semantic_bytes: usize,
    committed: bool,
}

impl SubmissionReservation {
    pub(super) fn commit(mut self) -> CommittedSubmissionReservation {
        self.committed = true;
        CommittedSubmissionReservation {
            scope_members: self.scope_members,
            semantic_bytes: self.semantic_bytes,
        }
    }
}

impl Drop for SubmissionReservation {
    fn drop(&mut self) {
        if !self.committed {
            self.state
                .reserved_semantic_bytes
                .fetch_sub(self.semantic_bytes, Ordering::AcqRel);
            self.state
                .reserved_scope_members
                .fetch_sub(self.scope_members, Ordering::AcqRel);
            self.state.reserved_commands.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

pub(super) struct CommittedSubmissionReservation {
    pub(super) scope_members: usize,
    pub(super) semantic_bytes: usize,
}

fn reserve_counter(counter: &AtomicUsize, amount: usize, limit: usize) -> Result<(), ()> {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = current.checked_add(amount).filter(|next| *next <= limit);
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
    }
    .map(|_| ())
    .map_err(|_| ())
}

const fn deferred(
    dimension: PhysicalWorkCapacityDimension,
    capacity: usize,
) -> PhysicalWorkSubmissionDeferred {
    PhysicalWorkSubmissionDeferred {
        dimension,
        capacity,
    }
}
