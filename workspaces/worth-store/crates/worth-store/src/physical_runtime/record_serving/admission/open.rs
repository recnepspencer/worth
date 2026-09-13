use worth_store_physical_backend::{ArtifactTreeFailureKind, QualifiedFilesystemMedia};
use worth_store_physical_format::{
    DurableArtifactCrc32c, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    FreeSpaceHeaderScopeIdentity, PhysicalGeneration, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use crate::physical_runtime::integrity::resident_admission::root_manifest::admit_loaded_root_manifest;
use crate::physical_runtime::integrity::resident_admission::{
    free_space::admit_resident_free_space_header, load::ResidentAdmissionContext,
    root_protocol::admit_resident_bootstrap_catalog,
};

use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::super::{
    admission::bootstrap::{
        backend_before_effect, BootstrapCatalogReadLimits, BootstrapTransitionFailure,
        PhysicalRecordBootstrapOwner, RecordServingStaleReason, RecordServingState,
    },
    publication::publication_residue::observe_publication_residue,
    residency::artifact_tree::RecordFamilyInventory,
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, PhysicalRecordFormatMismatch,
    RecordBootstrapDenial, UnsupportedPhysicalRecordFormat,
};
use super::integrity_denial::{classify_catalog, classify_free_space, classify_root};

#[derive(Clone)]
struct CurrentRootAdmission<'a> {
    media: &'a QualifiedFilesystemMedia,
    loader: &'a (dyn super::super::residency::frame_ports::FrameLoadPort + Send + Sync),
    allocation: &'a worth_store_buffer_pool::OperationAllocationGrant,
    limits: BootstrapCatalogReadLimits,
    generation: u64,
    expected_format: PhysicalRecordFormatDeclaration,
    lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    route: crate::physical_runtime::PhysicalRootProtocolRoute,
    counters: &'a crate::physical_runtime::RootProtocolRouteCounterCells,
    resident_integrity_counters: &'a crate::physical_runtime::ResidentAdmissionCounterCells,
}

pub(in crate::physical_runtime::record_serving) fn open(
    media: &QualifiedFilesystemMedia,
    loader: &(dyn super::super::residency::frame_ports::FrameLoadPort + Send + Sync),
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    resident_integrity_counters: &crate::physical_runtime::ResidentAdmissionCounterCells,
) -> Result<PhysicalRecordBootstrapOwner, BootstrapTransitionFailure> {
    if !access.admits(format) {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::ConfigurationMismatch,
        ));
    }
    let artifacts = ServingRecordArtifacts::new(media, loader);
    let limits = BootstrapCatalogReadLimits::for_format(format, access);
    match artifacts.inventory().map_err(backend_before_effect)? {
        RecordFamilyInventory::ProvenAbsent => {
            return Err(BootstrapTransitionFailure::Denied(
                RecordBootstrapDenial::RecordFamilyAbsent,
            ));
        }
        RecordFamilyInventory::Residue => {
            return Err(BootstrapTransitionFailure::Denied(
                RecordBootstrapDenial::AmbiguousRecordFamilyResidue,
            ));
        }
        RecordFamilyInventory::Published => {}
    }
    let catalog_frame = artifacts
        .load_bounded(
            allocation,
            RecordArtifactFile::BootstrapCatalog,
            limits.catalog_bytes(),
        )
        .map_err(|failure| match failure.kind() {
            super::super::residency::frame_loading::FrameLoadFailureKind::Backend(failure)
                if failure.kind() == ArtifactTreeFailureKind::Absent =>
            {
                BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CatalogMissing)
            }
            super::super::residency::frame_loading::FrameLoadFailureKind::Residency(reason) => {
                BootstrapTransitionFailure::Denied(RecordBootstrapDenial::from_residency(reason))
            }
            super::super::residency::frame_loading::FrameLoadFailureKind::Backend(failure) => {
                match failure.kind() {
                    ArtifactTreeFailureKind::AccessLimitExceeded
                    | ArtifactTreeFailureKind::Damaged => {
                        BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CatalogDamaged)
                    }
                    _ => backend_before_effect(failure),
                }
            }
            _ => BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CatalogDamaged),
        })?;
    let scope = PhysicalArtifactScope::bootstrap_catalog(
        media.store_identity(),
        format.declaration(),
        PhysicalByteRange::new(0, u64::from(limits.catalog_bytes()))
            .expect("the bootstrap catalog has a nonzero fixed width"),
    );
    let admission_context = ResidentAdmissionContext::new(lifecycle, resident_integrity_counters);
    let admitted =
        admit_resident_bootstrap_catalog(catalog_frame.lease(), scope, admission_context.clone())
            .map_err(classify_catalog)?;
    let catalog = admitted
        .project(admission_context)
        .map_err(classify_catalog)?;
    let observed_staging_residue = artifacts
        .has_staging_residue()
        .map_err(backend_before_effect)?;
    Ok(PhysicalRecordBootstrapOwner {
        format,
        access,
        current_root: catalog.current_root,
        observed_staging_residue,
    })
}

pub(in crate::physical_runtime::record_serving) fn load_current_root(
    media: &QualifiedFilesystemMedia,
    loader: &(dyn super::super::residency::frame_ports::FrameLoadPort + Send + Sync),
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    bootstrap: PhysicalRecordBootstrapOwner,
    lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    route: crate::physical_runtime::PhysicalRootProtocolRoute,
    counters: &crate::physical_runtime::RootProtocolRouteCounterCells,
    resident_integrity_counters: &crate::physical_runtime::ResidentAdmissionCounterCells,
) -> Result<RecordServingState, BootstrapTransitionFailure> {
    let limits = BootstrapCatalogReadLimits::for_format(bootstrap.format, bootstrap.access);
    let generation = bootstrap.current_root.generation().get();
    let admission = CurrentRootAdmission {
        media,
        loader,
        allocation,
        limits,
        generation,
        expected_format: bootstrap.format.declaration(),
        lifecycle,
        route,
        counters,
        resident_integrity_counters,
    };
    let current_root = load_root_manifest(&admission)?;
    let previous_root = if generation == 1 {
        None
    } else {
        let previous = CurrentRootAdmission {
            generation: generation - 1,
            lifecycle: std::sync::Arc::clone(&admission.lifecycle),
            ..admission.clone()
        };
        Some(load_root_manifest(&previous)?)
    };
    let free_space = load_free_space_manifest(&admission, &current_root)?;
    let publication_residue = observe_publication_residue(
        &ServingRecordArtifacts::new(media, loader),
        &current_root,
        &free_space,
        bootstrap.observed_staging_residue,
    )
    .map_err(backend_before_effect)?;
    Ok(RecordServingState {
        format: bootstrap.format,
        access: bootstrap.access,
        current_root,
        previous_root,
        publication_residue,
        free_space,
        root_protocol_counters: counters.snapshot(),
    })
}

fn load_root_manifest(
    admission: &CurrentRootAdmission<'_>,
) -> Result<DurablePhysicalRootManifest, BootstrapTransitionFailure> {
    let root_frame = ServingRecordArtifacts::new(admission.media, admission.loader)
        .load_bounded(
            admission.allocation,
            RecordArtifactFile::RootManifest {
                generation: admission.generation,
            },
            admission.limits.current_root_bytes().get(),
        )
        .map_err(|failure| {
            BootstrapTransitionFailure::Denied(match failure.kind() {
                super::super::residency::frame_loading::FrameLoadFailureKind::Residency(reason) => {
                    RecordBootstrapDenial::from_residency(reason)
                }
                super::super::residency::frame_loading::FrameLoadFailureKind::Backend(failure) => {
                    match failure.kind() {
                        ArtifactTreeFailureKind::Absent
                        | ArtifactTreeFailureKind::AccessLimitExceeded
                        | ArtifactTreeFailureKind::Damaged => {
                            RecordBootstrapDenial::CurrentRootDamaged
                        }
                        _ => RecordBootstrapDenial::BackendUnavailable(failure),
                    }
                }
                _ => RecordBootstrapDenial::CurrentRootDamaged,
            })
        })?;
    let admitted = admit_loaded_root_manifest(
        root_frame.lease(),
        std::sync::Arc::clone(&admission.lifecycle),
        admission.media.store_identity(),
        admission.expected_format,
        admission.generation,
        admission.resident_integrity_counters,
    )
    .map_err(classify_root)?;
    let current_root = admitted
        .project(
            std::sync::Arc::clone(&admission.lifecycle),
            admission.resident_integrity_counters,
        )
        .map_err(classify_root)?;
    admission.counters.observe_root(admission.route);
    if !super::super::planning::policy_units::manifest_capacity_can_branch(
        current_root.node_capacity(),
    ) {
        return Err(BootstrapTransitionFailure::Denied(
            RecordBootstrapDenial::CurrentRootDamaged,
        ));
    }
    if current_root.generation() != admission.generation {
        return Err(BootstrapTransitionFailure::Stale(
            RecordServingStaleReason::CatalogSelectedRootGenerationMismatch,
        ));
    }
    Ok(current_root)
}

fn load_free_space_manifest(
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
