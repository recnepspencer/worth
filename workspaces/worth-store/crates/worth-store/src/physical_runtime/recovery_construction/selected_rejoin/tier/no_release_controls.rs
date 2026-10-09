//! One current failed-ingest join, with ordinary NoRelease and native-funded
//! read ports.

use std::collections::BTreeMap;

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
};

use super::super::control_frames::SelectedArtifactSlice;
use super::super::SelectedMediaRejoinDenial as Denial;
use super::{no_release_frame, routes::RouteWalkStorage};

#[path = "no_release_controls/core.rs"]
mod core;
#[path = "no_release_controls/semantic.rs"]
mod semantic;
#[cfg(test)]
use semantic::{descriptor_matches_manifest, verify_reservation, Descriptor, Manifest};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) trait FailedIngestFrameStorage:
    RouteWalkStorage
{
    fn read_selected(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        route: CurrentPhysicalRecordPlacement,
        maximum: u64,
        slices: &mut Vec<SelectedArtifactSlice>,
        max_slices: usize,
    ) -> Result<Vec<u8>, Denial>;
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) trait FailedIngestRoutes {
    type Ordered<'a>: Iterator<Item = CurrentPhysicalRecordPlacement>
    where
        Self: 'a;

    fn ordered(&self) -> Self::Ordered<'_>;
    fn route(&self, record: PersistedRecordIdentity) -> Option<CurrentPhysicalRecordPlacement>;
}

impl FailedIngestRoutes for BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement> {
    type Ordered<'a> = std::iter::Copied<
        std::collections::btree_map::Values<
            'a,
            PersistedRecordIdentity,
            CurrentPhysicalRecordPlacement,
        >,
    >;

    fn ordered(&self) -> Self::Ordered<'_> {
        self.values().copied()
    }
    fn route(&self, record: PersistedRecordIdentity) -> Option<CurrentPhysicalRecordPlacement> {
        self.get(&record).copied()
    }
}

impl FailedIngestRoutes for [CurrentPhysicalRecordPlacement] {
    type Ordered<'a> = std::iter::Copied<std::slice::Iter<'a, CurrentPhysicalRecordPlacement>>;

    fn ordered(&self) -> Self::Ordered<'_> {
        self.iter().copied()
    }
    fn route(&self, record: PersistedRecordIdentity) -> Option<CurrentPhysicalRecordPlacement> {
        self.binary_search_by_key(&record, |route| route.record())
            .ok()
            .map(|index| self[index])
    }
}

impl FailedIngestFrameStorage for () {
    fn read_selected(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        route: CurrentPhysicalRecordPlacement,
        maximum: u64,
        slices: &mut Vec<SelectedArtifactSlice>,
        max_slices: usize,
    ) -> Result<Vec<u8>, Denial> {
        let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
            return Err(Denial::UnsupportedSelectedPlacement);
        };
        no_release_frame::read_with_slice_limit(
            discovery, format, extent, maximum, slices, max_slices,
        )
    }
}

pub(super) fn verify(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<BTreeMap<PersistedRecordIdentity, ([u8; 32], u64)>, Denial> {
    verify_bounded(
        discovery,
        routes,
        format,
        selected_generation,
        checkpoint_sequence,
        slices,
        usize::MAX,
    )
}

pub(super) fn verify_bounded(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    selected_generation: u64,
    checkpoint_sequence: u64,
    slices: &mut Vec<SelectedArtifactSlice>,
    max_slices: usize,
) -> Result<BTreeMap<PersistedRecordIdentity, ([u8; 32], u64)>, Denial> {
    let proofs = core::verify_with_storage(
        discovery,
        routes,
        format,
        selected_generation,
        checkpoint_sequence,
        slices,
        max_slices,
        &mut (),
    )?;
    Ok(proofs
        .into_iter()
        .map(|proof| {
            (
                proof.record,
                (proof.frame_digest, proof.candidate_generation),
            )
        })
        .collect())
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) use core::verify_with_storage;

#[cfg(test)]
#[path = "no_release_controls/tests.rs"]
mod tests;
