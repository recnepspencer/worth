use worth_store_physical_backend::{ArtifactTreeFailureKind, QualifiedFilesystemMedia};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use crate::physical_runtime::integrity::resident_admission::root_manifest::admit_loaded_root_manifest;
use crate::physical_runtime::integrity::resident_admission::{
    load::ResidentAdmissionContext, root_protocol::admit_resident_bootstrap_catalog,
};

use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::super::{
    admission::bootstrap::{
        backend_before_effect, BootstrapCatalogReadLimits, BootstrapTransitionFailure,
        PhysicalRecordBootstrapOwner, RecordServingStaleReason, RecordServingState,
    },
    publication::publication_residue::observe_publication_residue,
    residency::artifact_tree::RecordFamilyInventory,
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, RecordBootstrapDenial,
};
use super::integrity_denial::{classify_catalog, classify_root};

#[derive(Clone)]
pub(super) struct CurrentRootAdmission<'a> {
    pub(super) media: &'a QualifiedFilesystemMedia,
    pub(super) loader: &'a (dyn super::super::residency::frame_ports::FrameLoadPort + Send + Sync),
    pub(super) allocation: &'a worth_store_buffer_pool::OperationAllocationGrant,
    pub(super) limits: BootstrapCatalogReadLimits,
    pub(super) generation: u64,
    pub(super) expected_format: PhysicalRecordFormatDeclaration,
    pub(super) lifecycle: std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    pub(super) route: crate::physical_runtime::PhysicalRootProtocolRoute,
    pub(super) counters: &'a crate::physical_runtime::RootProtocolRouteCounterCells,
    pub(super) resident_integrity_counters:
        &'a crate::physical_runtime::ResidentAdmissionCounterCells,
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
    let current_root = load_root_manifest(&admission, true)?;
    let previous_root = if generation == 1 {
        None
    } else {
        let previous = CurrentRootAdmission {
            generation: generation - 1,
            lifecycle: std::sync::Arc::clone(&admission.lifecycle),
            ..admission.clone()
        };
        Some(load_root_manifest(&previous, true)?)
    };
    let artifacts = ServingRecordArtifacts::new(media, loader);
    let prior_roots = publication_roots(&admission, &current_root, previous_root.as_ref())?;
    let displaced_artifacts = if current_root.requires_maintenance_protocol() {
        let membership =
            super::super::access::segment_membership::SegmentMembershipReader::with_loader(
                media,
                loader,
                bootstrap.format,
                bootstrap.access,
                &current_root,
                std::sync::Arc::clone(&admission.lifecycle),
                resident_integrity_counters,
            );
        let mut displaced = super::displaced_segments::retained_displaced_segments(
            &prior_roots,
            &current_root,
            &artifacts,
            &membership,
            allocation,
        )?;
        displaced.extend(super::displaced_extents::retained_displaced_extents(
            &prior_roots,
            &current_root,
            &artifacts,
            |root| {
                super::super::access::manifest_routing::ManifestReader::with_loader(
                    media,
                    loader,
                    bootstrap.format,
                    bootstrap.access,
                    root,
                    std::sync::Arc::clone(&admission.lifecycle),
                    resident_integrity_counters,
                )
            },
            allocation,
        )?);
        displaced
    } else {
        Vec::new()
    };
    let publication_overheads = super::publication_charge::publication_overheads(
        &[prior_roots.as_slice(), std::slice::from_ref(&current_root)].concat(),
    );
    let free_space =
        super::current_free_space::load_free_space_manifest(&admission, &current_root)?;
    let publication_residue = observe_publication_residue(
        &artifacts,
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
        displaced_artifacts,
        publication_overheads,
        publication_residue,
        free_space,
        root_protocol_counters: counters.snapshot(),
    })
}

fn publication_roots(
    admission: &CurrentRootAdmission<'_>,
    current_root: &DurablePhysicalRootManifest,
    previous_root: Option<&DurablePhysicalRootManifest>,
) -> Result<Vec<DurablePhysicalRootManifest>, BootstrapTransitionFailure> {
    let mut prior = Vec::new();
    for generation in 1..current_root.generation() {
        let root = match previous_root {
            Some(root) if root.generation() == generation => root.clone(),
            _ => {
                let older = CurrentRootAdmission {
                    generation,
                    lifecycle: std::sync::Arc::clone(&admission.lifecycle),
                    ..admission.clone()
                };
                load_root_manifest(&older, false)?
            }
        };
        prior.push(root);
    }
    Ok(prior)
}

fn load_root_manifest(
    admission: &CurrentRootAdmission<'_>,
    record_route: bool,
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
    if record_route {
        admission.counters.observe_root(admission.route);
    }
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
