//! Store-owned completed-history head fold and two rooted media observations.
//! C.8's effective roster is compared only after the Store has walked and
//! folded the selected media independently.

use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    verify_release_custody_head_controls_view, verify_release_custody_head_successor_view,
    BlobReclaimDescriptorV3, DropSetManifestV3View, DurablePhysicalRootManifest,
    OriginalDropReservedV1, PersistedReleaseCustodyHeadEffectV1, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadControlIdentityV1,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadRosterDigestV1,
};
use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
};

use super::super::{
    control_frames::SelectedControlMediaFingerprint, release_heads,
    resident::StoreRejoinResidentLedger, SelectedMediaRejoinDenial as Denial,
};
use crate::physical_runtime::{
    durability::ReleaseHeadCapacityCharge, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryReadAllocation,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct ObservedCompletedHeadControls<
    'a,
> {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) frames:
        ReleaseCustodyHeadControlIdentityV1,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) descriptor:
        &'a BlobReclaimDescriptorV3,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) reservation:
        &'a OriginalDropReservedV1,
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) manifest_view:
        DropSetManifestV3View<'a>,
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) enum CompletedHistoryHeadStep<
    'a,
> {
    Ordinary {
        source: &'a DurablePhysicalRootManifest,
        result: &'a DurablePhysicalRootManifest,
    },
    Released {
        effect: &'a PersistedReleaseCustodyHeadEffectV1,
        controls: ObservedCompletedHeadControls<'a>,
        source: &'a DurablePhysicalRootManifest,
        result: &'a DurablePhysicalRootManifest,
    },
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct CompletedHistoryHeadFold
{
    checkpoint: release_heads::ObservedReleaseHeads,
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    entry_grant: Option<OperationAllocationGrant>,
    current_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    current_next_block: u64,
    maximum_entries: usize,
}

impl CompletedHistoryHeadFold {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn start(
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        window: &PhysicalRecoveryReadAllocation<'_>,
        checkpoint_source_root: &DurablePhysicalRootManifest,
        claim: &VerifiedOrderedHistoricalReleaseCustody,
        effective: &VerifiedEffectiveReleaseHeadRosterV14,
        allocation: PhysicalRecoveryAllocationAdmission,
        format: PhysicalRecordFormatDeclaration,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, Denial> {
        let checkpoint_ref = checkpoint_source_root.release_custody_head_root();
        let checkpoint_next_block = checkpoint_source_root.next_release_custody_head_block();
        if checkpoint_ref != effective.checkpoint_source_root()
            || checkpoint_next_block != effective.checkpoint_source_next_block()
            || discovery.store_identity() != allocation.store_identity()
        {
            return Err(Denial::CertificateRoster);
        }
        // The fold starts from the checkpoint-source roster: empty for a
        // NoRelease base, the checkpoint-attested heads for a HeadV2 base.
        let (checkpoint_count, checkpoint_digest) = match (claim.marker(), claim.selected_head_v2())
        {
            (Some(_), None) if checkpoint_ref.is_none() && checkpoint_next_block == 1 => {
                (0, ReleaseCustodyHeadRosterDigestV1::new(None, 0).finish().1)
            }
            (None, Some(base))
                if base.checkpoint_source_root() == checkpoint_source_root
                    && base.selected_heads() == effective.checkpoint_source_heads() =>
            {
                let accumulator = base.accumulator_v2();
                (accumulator.head_count(), accumulator.head_roster_digest())
            }
            _ => return Err(Denial::CertificateRoster),
        };
        let maximum_entries = effective.effective_heads().len();
        let page = u64::from(format.page_size().bytes());
        if maximum_entries == 0
            || (maximum_entries as u64) < checkpoint_count
            || ReleaseHeadCapacityCharge::selected_roster_closure_bytes(
                maximum_entries as u64,
                page,
            )
            .is_none_or(|bytes| bytes > allocation.byte_limit())
        {
            return Err(Denial::BoundExceeded);
        }
        let checkpoint = release_heads::observe_with_resident(
            discovery,
            window,
            checkpoint_source_root,
            format,
            allocation,
            checkpoint_count,
            checkpoint_digest,
            resident,
        )?;
        if checkpoint.entries() != effective.checkpoint_source_heads() {
            return Err(Denial::CertificateRoster);
        }
        let requested = entry_bytes(maximum_entries)?;
        let entry_grant = window.reserve_owned(requested).map_err(Denial::Resident)?;
        let mut entries = resident
            .reserve_vec::<ReleaseCustodyHeadEntryV1>(maximum_entries)
            .map_err(Denial::Resident)?;
        let actual = entry_bytes(entries.capacity())?;
        if entry_grant
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes)
            < actual
        {
            return Err(Denial::Resident(
                crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::AllocatorExceededReservation {
                    requested,
                    actual,
                },
            ));
        }
        entries.extend_from_slice(checkpoint.entries());
        Ok(Self {
            checkpoint,
            entries,
            entry_grant,
            current_root: checkpoint_ref,
            current_next_block: checkpoint_next_block,
            maximum_entries,
        })
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn apply(
        &mut self,
        step: CompletedHistoryHeadStep<'_>,
    ) -> Result<(), Denial> {
        match step {
            CompletedHistoryHeadStep::Ordinary { source, result } => {
                self.check_source(source)?;
                if result.release_custody_head_root() != self.current_root
                    || result.next_release_custody_head_block() != self.current_next_block
                {
                    return Err(Denial::CertificateRoster);
                }
            }
            CompletedHistoryHeadStep::Released {
                effect,
                controls,
                source,
                result,
            } => self.apply_released(effect, controls, source, result)?,
        }
        Ok(())
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn finish(
        self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        window: &PhysicalRecoveryReadAllocation<'_>,
        selected_root: &DurablePhysicalRootManifest,
        effective: &VerifiedEffectiveReleaseHeadRosterV14,
        allocation: PhysicalRecoveryAllocationAdmission,
        format: PhysicalRecordFormatDeclaration,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        self.check_source(selected_root)?;
        let rooted = self.current_root.ok_or(Denial::CertificateRoster)?;
        let mut digest =
            ReleaseCustodyHeadRosterDigestV1::new(Some(rooted), self.entries.len() as u64);
        for entry in &self.entries {
            digest.push(*entry).map_err(|_| Denial::CertificateRoster)?;
        }
        let (count, expected_digest) = digest.finish();
        let selected = release_heads::observe_with_resident(
            discovery,
            window,
            selected_root,
            format,
            allocation,
            count,
            expected_digest,
            resident,
        )?;
        if selected.entries() != self.entries
            || effective.effective_heads() != self.entries
            || effective.effective_root() != rooted
            || effective.effective_next_block() != self.current_next_block
            || effective.effective_digest() != expected_digest
            || effective.checkpoint_source_heads() != self.checkpoint.entries()
        {
            return Err(Denial::CertificateRoster);
        }
        let fold_bytes = resident
            .vector_bytes(&self.entries)
            .map_err(Denial::Resident)?;
        drop(self.entries);
        drop(self.entry_grant);
        resident.release(fold_bytes);
        let checkpoint = self.checkpoint.into_funded_slices_with_resident(resident)?;
        let selected = selected.into_funded_slices_with_resident(resident)?;
        Ok(SelectedControlMediaFingerprint::completed_history_heads(
            checkpoint, selected,
        ))
    }

    fn check_source(&self, source: &DurablePhysicalRootManifest) -> Result<(), Denial> {
        (source.release_custody_head_root() == self.current_root
            && source.next_release_custody_head_block() == self.current_next_block)
            .then_some(())
            .ok_or(Denial::CertificateRoster)
    }

    fn apply_released(
        &mut self,
        effect: &PersistedReleaseCustodyHeadEffectV1,
        controls: ObservedCompletedHeadControls<'_>,
        source: &DurablePhysicalRootManifest,
        result: &DurablePhysicalRootManifest,
    ) -> Result<(), Denial> {
        self.check_source(source)?;
        if effect.source_root() != self.current_root
            || effect.source_next_block() != self.current_next_block
            || result.release_custody_head_root() != Some(effect.result_root())
            || result.next_release_custody_head_block() != effect.result_next_block()
        {
            return Err(Denial::CertificateRoster);
        }
        let ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior,
            next,
        } = effect.mutation()
        else {
            return Err(Denial::CertificateRoster);
        };
        verify_release_custody_head_controls_view(
            next,
            controls.frames,
            *controls.descriptor,
            *controls.reservation,
            controls.manifest_view,
        )
        .map_err(|_| Denial::ControlFrame)?;
        let index = self
            .entries
            .binary_search_by_key(&next.key(), |entry| entry.key());
        let actual_prior = index.ok().map(|at| self.entries[at]);
        if actual_prior != expected_prior {
            return Err(Denial::ControlFrame);
        }
        if let Some(prior) = actual_prior {
            verify_release_custody_head_successor_view(
                prior,
                next,
                controls.frames,
                *controls.descriptor,
                *controls.reservation,
                controls.manifest_view,
            )
            .map_err(|_| Denial::ControlFrame)?;
        } else if next.cumulative_dropped() != u64::from(controls.manifest_view.count()) {
            return Err(Denial::ControlFrame);
        }
        match index {
            Ok(at) => self.entries[at] = next,
            Err(at) => {
                if self.entries.len() >= self.maximum_entries
                    || self.entries.len() >= self.entries.capacity()
                {
                    return Err(Denial::BoundExceeded);
                }
                self.entries.insert(at, next);
            }
        }
        self.current_root = Some(effect.result_root());
        self.current_next_block = effect.result_next_block();
        Ok(())
    }
}

fn entry_bytes(count: usize) -> Result<u64, Denial> {
    count
        .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::BoundExceeded)
}
