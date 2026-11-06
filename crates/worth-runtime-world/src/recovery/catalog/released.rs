//! Pre-admitted terminal destinations retain data independently of active custody.
use super::*;
use crate::durability::WorldDurableMediumDenial as Denial;
use crate::recovery::released_original::{
    ReleasedOriginalSlot, ReleasedOriginalSource, WorldReleasedOriginal, WorldReleasedOriginalPage,
};
use std::sync::{OnceLock, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMapCell, ExecutionRequest};
use worth_foundational::facade::{
    DurableProductBranchReference, MemoryProductBranchReference, RecordedProductBranchReference,
};

#[derive(Debug)]
pub(crate) struct PreparedReleasedOriginal {
    owner: Weak<Mutex<RecoveryCatalogState>>,
    slot: Arc<ReleasedOriginalSlot>,
    armed: bool,
}
impl PreparedReleasedOriginal {
    pub(crate) fn finish_directory(
        mut self,
        disposition: crate::durability::AdmittedWorldDisposition,
        native: Option<worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect>,
    ) -> WorldReleasedOriginal {
        self.slot
            .source
            .set(ReleasedOriginalSource::Directory {
                disposition,
                native,
            })
            .expect("one actual release fills one original destination");
        self.armed = false;
        WorldReleasedOriginal::from_sealed(self.slot.clone())
    }
    pub(crate) fn finish_memory(
        mut self,
        native: Option<worth_relational::facade::durability::ObservedRelationalMemoryCommit>,
    ) -> WorldReleasedOriginal {
        self.slot
            .source
            .set(ReleasedOriginalSource::Memory { native })
            .expect("one actual release fills one original destination");
        self.armed = false;
        WorldReleasedOriginal::from_sealed(self.slot.clone())
    }
}
impl Drop for PreparedReleasedOriginal {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let Some(owner) = self.owner.upgrade() else {
            return;
        };
        let removed = owner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .released
            .remove_owned_retaining_unique(&self.slot.identity.ordinal())
            .expect("terminal catalog never shares its map nodes");
        drop(removed);
    }
}

fn backing(
    request: &ExecutionRequest<'_, '_>,
    label_bytes: usize,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    request.consult().map_err(Denial::Request)?;
    let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<ReleasedOriginalSlot>())
        .map_err(|_| Denial::Unavailable)?;
    let bytes = layout
        .pad_to_align()
        .size()
        .checked_add(label_bytes)
        .ok_or(Denial::Unavailable)?;
    request
        .reserve_retained_memory(bytes as u64)
        .map_err(Denial::Request)
}

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn readmit_released_original(
        &self,
        request: &ExecutionRequest<'_, '_>,
        disposition: &crate::durability::AdmittedWorldDisposition,
        expected: &crate::branch::ProductBranchObservation,
        native: Option<worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect>,
    ) -> Result<(), Denial> {
        use crate::identity::readmission::{readmit_attempt, readmit_commit, readmit_custody};
        let open = disposition.open();
        let owner = expected.owner_identity();
        let identity = readmit_custody(owner, open.custody).ok_or(Denial::PublicationUnresolved)?;
        // Recovery may retry an already admitted prefix. Only the identical
        // authenticated source can reuse a terminal cell.
        if let Some(slot) = self
            .state
            .try_lock()
            .map_err(|_| Denial::PublicationBusy)?
            .released
            .get(&identity.ordinal())
        {
            let Some(ReleasedOriginalSource::Directory {
                disposition: prior,
                native: prior_native,
                ..
            }) = slot.source.get()
            else {
                return Err(Denial::PublicationUnresolved);
            };
            if prior.open().association != open.association
                || prior.position() != disposition.position()
                || prior.digest() != disposition.digest()
                || prior_native.as_ref().map(|n| {
                    (
                        n.subject().receipt(),
                        n.subject().position(),
                        n.subject().artifact_digest(),
                    )
                }) != native.as_ref().map(|n| {
                    (
                        n.subject().receipt(),
                        n.subject().position(),
                        n.subject().artifact_digest(),
                    )
                })
            {
                return Err(Denial::PublicationUnresolved);
            }
            return Ok(());
        }
        let labels = open.expected.branch.name().len();
        let funding = backing(request, labels)?;
        let head = &open.expected;
        let recorded = DurableProductBranchReference::from_recorded(
            head.branch.clone(),
            head.incarnation,
            head.generation,
            head.commit,
        )
        .map_err(|_| Denial::PublicationUnresolved)?;
        let destination = self.reserve_released_slot(
            request,
            ReleasedOriginalSlot {
                identity,
                attempt: readmit_attempt(owner, open.association.attempt())
                    .ok_or(Denial::PublicationUnresolved)?,
                commit: Some(
                    readmit_commit(owner, open.commit).ok_or(Denial::PublicationUnresolved)?,
                ),
                branch: expected.branch_identity().clone(),
                incarnation: expected.lifecycle_incarnation(),
                expected: RecordedProductBranchReference::Home(recorded),
                source: OnceLock::new(),
                _funding: funding,
            },
        )?;
        destination.finish_directory(disposition.clone(), native);
        Ok(())
    }
    fn reserve_released_slot(
        &self,
        request: &ExecutionRequest<'_, '_>,
        slot: ReleasedOriginalSlot,
    ) -> Result<PreparedReleasedOriginal, Denial> {
        let slot = Arc::new(slot);
        let cell = ExecutionOrderedMapCell::prepare_owned_native_for_request(
            slot.identity.ordinal(),
            slot.clone(),
            *request,
        )
        .map_err(Denial::Ordered)?;
        let mut state = self.state.try_lock().map_err(|_| Denial::PublicationBusy)?;
        if slot.identity.owner_identity() != state.owner {
            return Err(Denial::Unavailable);
        }
        state
            .released
            .insert_owned_vacant_prepared(cell)
            .map_err(|_| Denial::Unavailable)?;
        Ok(PreparedReleasedOriginal {
            owner: Arc::downgrade(&self.state),
            slot,
            armed: true,
        })
    }
    pub(crate) fn prepare_recovered_original_release(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &crate::recovery::RecoveredWorldPublicationRecord,
    ) -> Result<PreparedReleasedOriginal, Denial> {
        let head = &original.open.expected;
        let funding = backing(request, head.branch.name().len())?;
        let expected = DurableProductBranchReference::from_recorded(
            head.branch.clone(),
            head.incarnation,
            head.generation,
            head.commit,
        )
        .map_err(|_| Denial::PublicationUnresolved)?;
        self.reserve_released_slot(
            request,
            ReleasedOriginalSlot {
                identity: original.identity.clone(),
                attempt: original.attempt.clone(),
                commit: Some(original.commit.clone()),
                branch: original.expected.branch_identity().clone(),
                incarnation: original.expected.lifecycle_incarnation(),
                expected: RecordedProductBranchReference::Home(expected),
                source: OnceLock::new(),
                _funding: funding,
            },
        )
    }
    pub(crate) fn prepare_memory_original_release(
        &self,
        request: &ExecutionRequest<'_, '_>,
        record: &Arc<ProductUnpublishedOwnerEffectsRecord>,
    ) -> Result<PreparedReleasedOriginal, Denial> {
        let facts = ProductUnpublishedOwnerEffects::from_catalog_record(record.clone());
        let expected = facts.expected_head();
        if expected.owner_identity().durable().is_some() {
            return Err(Denial::Unavailable);
        }
        let funding = backing(request, expected.branch_identity().name().as_str().len())?;
        let head = MemoryProductBranchReference::from_observed(
            expected.owner_identity().get(),
            expected.branch_identity().name().as_str().to_owned(),
            expected.lifecycle_incarnation().ordinal(),
            expected.reference_generation().get(),
            expected.selected_commit().ordinal(),
        )
        .map_err(|_| Denial::Unavailable)?;
        self.reserve_released_slot(
            request,
            ReleasedOriginalSlot {
                identity: facts.identity().clone(),
                attempt: facts.attempt_identity().clone(),
                commit: facts.successor_commit().cloned(),
                branch: expected.branch_identity().clone(),
                incarnation: expected.lifecycle_incarnation(),
                expected: RecordedProductBranchReference::Memory(head),
                source: OnceLock::new(),
                _funding: funding,
            },
        )
    }
    pub(crate) fn released_original_page(
        &self,
        request: &ExecutionRequest<'_, '_>,
        after: Option<&ProductUnpublishedOwnerEffectsIdentity>,
        maximum: std::num::NonZeroUsize,
    ) -> Result<WorldReleasedOriginalPage, Denial> {
        request.consult().map_err(Denial::Request)?;
        let state = self.state.try_lock().map_err(|_| Denial::PublicationBusy)?;
        if after.is_some_and(|key| key.owner_identity() != state.owner) {
            return Err(Denial::Unavailable);
        }
        let count = state.released.len().min(maximum.get());
        let bytes = count
            .checked_mul(std::mem::size_of::<WorldReleasedOriginal>())
            .ok_or(Denial::Unavailable)?;
        let funding = request
            .reserve_retained_memory(bytes as u64)
            .map_err(Denial::Request)?;
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| Denial::Unavailable)?;
        let mut next_after = None;
        for (ordinal, slot) in state.released.iter() {
            request.consult().map_err(Denial::Request)?;
            if after.is_some_and(|key| *ordinal <= key.ordinal()) || slot.source.get().is_none() {
                continue;
            }
            if rows.len() == maximum.get() {
                next_after = rows
                    .last()
                    .map(|row: &WorldReleasedOriginal| row.identity().clone());
                break;
            }
            rows.push(WorldReleasedOriginal::from_sealed(slot.clone()));
        }
        Ok(WorldReleasedOriginalPage {
            rows,
            next_after,
            _funding: funding,
        })
    }
}
