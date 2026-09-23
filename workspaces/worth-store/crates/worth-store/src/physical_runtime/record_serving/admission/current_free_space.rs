use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    FreeSpaceHeaderScopeIdentity, PhysicalGeneration, PhysicalTreeIdentity, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use crate::physical_runtime::integrity::resident_admission::{
    free_space::admit_resident_free_space_header, load::ResidentAdmissionContext,
};

use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::super::{
    admission::bootstrap::{BootstrapTransitionFailure, RecordServingStaleReason},
    PhysicalRecordFormatMismatch, RecordBootstrapDenial, UnsupportedPhysicalRecordFormat,
};
use super::integrity_denial::classify_free_space;
use super::open::CurrentRootAdmission;

/// Admits the free-space manifest the selected root names, requiring its
/// generation, root, format and allocation cursors to agree with that root.
pub(super) fn load_free_space_manifest(
    admission: &CurrentRootAdmission<'_>,
    current_root: &DurablePhysicalRootManifest,
) -> Result<DurableFreeSpaceManifestHeader, BootstrapTransitionFailure> {
    let free_space_bytes = ServingRecordArtifacts::new(admission.media, admission.loader)
        .load_bounded(
            admission.allocation,
            RecordArtifactFile::FreeSpaceManifest {
                generation: admission.generation,
            },
            admission.limits.current_root_bytes().get(),
        )
        .map_err(|failure| {
            BootstrapTransitionFailure::Denied(match failure.kind() {
                super::super::residency::frame_loading::FrameLoadFailureKind::Residency(reason) => {
                    RecordBootstrapDenial::from_residency(reason)
                }
                _ => RecordBootstrapDenial::FreeSpaceManifestDamaged,
            })
        })?;
    let generation = PhysicalGeneration::from_raw(admission.generation).map_err(|_| {
        BootstrapTransitionFailure::Denied(RecordBootstrapDenial::FreeSpaceManifestDamaged)
    })?;
    let tree = PhysicalTreeIdentity::new(current_root.tree_identity()).ok_or_else(|| {
        BootstrapTransitionFailure::Denied(RecordBootstrapDenial::FreeSpaceManifestDamaged)
    })?;
    let range = PhysicalByteRange::new(0, free_space_bytes.len() as u64).map_err(|_| {
        BootstrapTransitionFailure::Denied(RecordBootstrapDenial::FreeSpaceManifestDamaged)
    })?;
    let scope = PhysicalArtifactScope::free_space_header(
        admission.media.store_identity(),
        admission.expected_format,
        FreeSpaceHeaderScopeIdentity::new(
            generation,
            tree,
            current_root.free_space_root(),
            DurableArtifactCrc32c::new(current_root.free_space_checksum()),
        ),
        range,
    );
    let context = ResidentAdmissionContext::new(
        std::sync::Arc::clone(&admission.lifecycle),
        admission.resident_integrity_counters,
    );
    let admitted =
        admit_resident_free_space_header(free_space_bytes.lease(), scope, context.clone())
            .map_err(classify_free_space)?;
    let (free_space, free_format) = admitted
        .with_owner_decoder(context, |view| {
            view.project_header(admission.limits.current_root_entries())
        })
        .map_err(classify_free_space)?
        .map_err(classify_free_space_denial)?;
    if !super::super::planning::policy_units::manifest_capacity_can_branch(
        free_space.node_capacity(),
    ) {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::FreeSpaceManifestDamaged,
        ));
    }
    if free_space.generation() != admission.generation {
        return Err(BootstrapTransitionFailure::Stale(
            RecordServingStaleReason::FreeSpaceGenerationMismatch,
        ));
    }
    if free_space.root() != current_root.free_space_root() {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::FreeSpaceManifestDamaged,
        ));
    }
    if free_format != admission.expected_format {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::PhysicalRecordFormatMismatch(PhysicalRecordFormatMismatch::new(
                admission.expected_format,
                free_format,
            )),
        ));
    }
    if free_space.next_segment() == 0
        || free_space.next_page() == 0
        || free_space.next_extent() == 0
    {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::FreeSpaceManifestDamaged,
        ));
    }
    Ok(free_space)
}

fn classify_free_space_denial(
    denial: worth_store_physical_format::FreeSpaceRoutingDenial,
) -> BootstrapTransitionFailure {
    match denial {
        worth_store_physical_format::FreeSpaceRoutingDenial::Frame(
            worth_store_physical_format::DurableFrameDenial::UnsupportedFormat(reason),
        ) => BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::UnsupportedPhysicalRecordFormat(
                UnsupportedPhysicalRecordFormat::new(reason),
            ),
        ),
        _ => BootstrapTransitionFailure::Denied(RecordBootstrapDenial::FreeSpaceManifestDamaged),
    }
}
