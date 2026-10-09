//! Fingerprint composition keeps admissibility, backing growth, head-effect
//! merge and donor disposal together across all rejoin funding postures.

use super::{
    FundedHeadEffectSlices, SelectedArtifactSlice, SelectedControlMediaFingerprint,
    SelectedHeadMediaWitness,
};
use crate::physical_runtime::recovery_construction::selected_rejoin::{
    resident::StoreRejoinResidentLedger, tier::routes::RouteWalkStorage, SelectedMediaRejoinDenial,
};

impl SelectedControlMediaFingerprint {
    pub(in crate::physical_runtime) fn extend(
        &mut self,
        other: Self,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        self.admit_merge(&other)?;
        self.merge_effects(other.effects)?;
        self.slices.extend(other.slices);
        self.heads.merge_admitted(other.heads);
        Ok(())
    }

    /// Both fingerprints' backing is already retained in this same ledger.
    /// Growth keeps the donor and previous destination charged through the
    /// reallocation, then releases the donor only after its backing is dropped.
    pub(in crate::physical_runtime) fn extend_with_resident(
        &mut self,
        mut other: Self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        self.admit_merge(&other)?;
        let donor_bytes = resident
            .vector_bytes(&other.slices)
            .map_err(SelectedMediaRejoinDenial::Resident)?;
        resident
            .grow_vec(&mut self.slices, other.slices.len())
            .map_err(SelectedMediaRejoinDenial::Resident)?;
        self.merge_effects(other.effects.take())?;
        self.slices.append(&mut other.slices);
        self.heads.merge_admitted(std::mem::replace(
            &mut other.heads,
            SelectedHeadMediaWitness::Absent,
        ));
        drop(other);
        resident.release(donor_bytes);
        Ok(())
    }

    /// Completed-history raw vectors use the original pool's native grant as
    /// well as the aggregate resident ledger. Both old and donor storage stay
    /// charged while a replacement destination is allocated.
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn extend_with_storage<
        S: RouteWalkStorage,
    >(
        &mut self,
        mut other: Self,
        storage: &mut S,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        self.admit_merge(&other)?;
        storage.grow_vec(&mut self.slices, other.slices.len())?;
        self.merge_effects(other.effects.take())?;
        self.slices.append(&mut other.slices);
        self.heads.merge_admitted(std::mem::replace(
            &mut other.heads,
            SelectedHeadMediaWitness::Absent,
        ));
        storage.discard_vec(other.slices)
    }

    pub(in crate::physical_runtime) fn try_extend_bounded(
        &mut self,
        other: Self,
        maximum_retained_bytes: u64,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        self.admit_merge(&other)?;
        let head_bytes = match self.heads.owned_heap_bytes().and_then(|bytes| {
            bytes
                .checked_add(other.heads.owned_heap_bytes()?)?
                .checked_mul(4)
        }) {
            Some(bytes) => bytes,
            None => return Err(SelectedMediaRejoinDenial::BoundExceeded),
        };
        let funded_effect_bytes = match (&self.effects, &other.effects) {
            (Some(destination), Some(donor)) => destination.merged_owned_heap_bytes(donor)?,
            _ => self
                .effect_heap_bytes()
                .and_then(|bytes| bytes.checked_add(other.effect_heap_bytes()?))
                .ok_or(SelectedMediaRejoinDenial::BoundExceeded)?,
        }
        .checked_mul(4)
        .ok_or(SelectedMediaRejoinDenial::BoundExceeded)?;
        let next_len = match self.slices.len().checked_add(other.slices.len()) {
            Some(next) => next,
            None => return Err(SelectedMediaRejoinDenial::BoundExceeded),
        };
        let needed = match (next_len as u64)
            .checked_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64)
            .and_then(|bytes| bytes.checked_add(head_bytes))
            .and_then(|bytes| bytes.checked_add(funded_effect_bytes))
        {
            Some(bytes) if bytes <= maximum_retained_bytes => bytes,
            _ => return Err(SelectedMediaRejoinDenial::BoundExceeded),
        };
        if self.slices.try_reserve_exact(other.slices.len()).is_err()
            || self.retained_memory_bytes() > maximum_retained_bytes
            || needed > maximum_retained_bytes
        {
            return Err(SelectedMediaRejoinDenial::BoundExceeded);
        }
        self.merge_effects(other.effects)?;
        self.slices.extend(other.slices);
        self.heads.merge_admitted(other.heads);
        (self.retained_memory_bytes() <= maximum_retained_bytes)
            .then_some(())
            .ok_or(SelectedMediaRejoinDenial::BoundExceeded)
    }

    fn admit_merge(&self, other: &Self) -> Result<(), SelectedMediaRejoinDenial> {
        if self.completed_raw.is_some() || other.completed_raw.is_some() {
            return Err(SelectedMediaRejoinDenial::RootBinding);
        }
        if !self.heads.can_merge(&other.heads) {
            return Err(SelectedMediaRejoinDenial::CertificateRoster);
        }
        if let (Some(destination), Some(donor)) = (&self.effects, &other.effects) {
            if !destination.can_merge(donor) {
                return Err(SelectedMediaRejoinDenial::RootBinding);
            }
        }
        Ok(())
    }

    fn merge_effects(
        &mut self,
        other: Option<FundedHeadEffectSlices>,
    ) -> Result<(), SelectedMediaRejoinDenial> {
        match (&mut self.effects, other) {
            (_, None) => Ok(()),
            (None, Some(effects)) => {
                self.effects = Some(effects);
                Ok(())
            }
            (Some(destination), Some(donor)) => destination.merge(donor),
        }
    }
}
