//! Shared custody for an immutable checkpoint page and its native frame.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use crate::contracts::storage::data::storage_unavailable::{
    RelationalStorageUnavailable, RelationalStorageUnavailableCause as Cause,
    RelationalStorageUnavailableCauseOwnerPort as _,
};
use worth_execution::ExecutionRequest;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

pub(super) mod member_export;

pub(super) trait ColdPageDescriptor {
    const BYTE_LIMIT: u64;
    const KIND: MemberKind;

    fn member(&self) -> RelationalDurableMemberLocator;
    fn storage_failure(&self, cause: Cause) -> RelationalStorageUnavailable;
}

#[derive(Debug)]
enum ColdPageState<P> {
    Cold,
    Resident(Arc<P>),
    Damaged(RelationalStorageUnavailable),
}

#[derive(Debug)]
pub(super) struct ColdPageCell<D, P> {
    pub(super) descriptor: D,
    state: Mutex<ColdPageState<P>>,
    leases: AtomicUsize,
}

impl<D: ColdPageDescriptor, P> ColdPageCell<D, P> {
    pub(super) fn new(descriptor: D) -> Self {
        Self {
            descriptor,
            state: Mutex::new(ColdPageState::Cold),
            leases: AtomicUsize::new(0),
        }
    }

    pub(super) fn demand(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
        admit: impl FnOnce(&[u8], &D) -> Result<P, Failure>,
    ) -> Result<ColdPageLease<D, P>, Failure> {
        crate::contracts::execution::request_custody::check_request(execution)
            .map_err(Failure::Request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if matches!(*state, ColdPageState::Cold) {
            let result = self
                .read_frame(source, D::BYTE_LIMIT)
                .map_err(|cause| Failure::from(self.descriptor.storage_failure(cause)))
                .and_then(|bytes| admit(&bytes, &self.descriptor));
            match result {
                Ok(page) => *state = ColdPageState::Resident(Arc::new(page)),
                Err(Failure::StorageUnavailable(failure)) => {
                    if failure.cause.is_proven_damage() {
                        *state = ColdPageState::Damaged(failure);
                    }
                    return Err(failure.into());
                }
                Err(stop) => return Err(stop),
            }
        }
        match &*state {
            ColdPageState::Resident(page) => self.lease(page).map_err(Into::into),
            ColdPageState::Damaged(failure) => Err((*failure).into()),
            ColdPageState::Cold => Err(self
                .descriptor
                .storage_failure(Cause::SourceUnavailable)
                .into()),
        }
    }

    fn read_frame(
        &self,
        source: &dyn RelationalDurableColdSource,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let member = self.descriptor.member();
        if member.length > limit {
            return Err(Cause::LimitExceeded {
                length: member.length,
                limit,
            });
        }
        source.read_member(member, limit)
    }

    fn lease(
        self: &Arc<Self>,
        page: &Arc<P>,
    ) -> Result<ColdPageLease<D, P>, RelationalStorageUnavailable> {
        self.leases
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_add(1)
            })
            .map_err(|_| self.descriptor.storage_failure(Cause::MemoryExhausted))?;
        Ok(ColdPageLease {
            cell: self.clone(),
            page: page.clone(),
        })
    }

    pub(super) fn capture_resident(
        self: &Arc<Self>,
    ) -> Result<Option<ColdPageLease<D, P>>, RelationalStorageUnavailable> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &*state {
            ColdPageState::Cold => Ok(None),
            ColdPageState::Damaged(failure) => Err(*failure),
            ColdPageState::Resident(page) => self.lease(page).map(Some),
        }
    }

    pub(super) fn reject_binding(&self) -> RelationalStorageUnavailable {
        let failure = self.descriptor.storage_failure(Cause::Damaged);
        *self.state.lock().unwrap_or_else(|error| error.into_inner()) =
            ColdPageState::Damaged(failure);
        failure
    }
}

#[derive(Debug)]
pub(super) struct ColdPageLease<D, P> {
    cell: Arc<ColdPageCell<D, P>>,
    page: Arc<P>,
}

impl<D, P> ColdPageLease<D, P> {
    pub(super) fn page(&self) -> &P {
        &self.page
    }

    pub(super) fn cell(&self) -> &ColdPageCell<D, P> {
        &self.cell
    }
}

impl<D, P> Drop for ColdPageLease<D, P> {
    fn drop(&mut self) {
        let previous = self.cell.leases.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(
            /* Proof: Each non-Clone lease owns one increment and retains its issuing cell until Drop. */
            previous > 0
        );
    }
}
