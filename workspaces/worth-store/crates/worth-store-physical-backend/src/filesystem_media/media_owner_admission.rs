use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use worth_store_physical_format::store_namespace::StoreNamespaceRelativeRole;

use super::super::{
    ArtifactFamilyDirectory, MediaFaultSchedule, MutationOwnershipDenial, MutationOwnershipLease,
    StagingDirectory,
};
use super::{
    FilesystemMediaAdmissionAuthority, FilesystemMediaOwner, FilesystemMediaOwnerAdmissionDenial,
    ObservedMediaOwnerAdmissionFailure,
};

impl FilesystemMediaOwner {
    pub fn admit(
        root: &Path,
        _authority: FilesystemMediaAdmissionAuthority,
    ) -> Result<Self, FilesystemMediaOwnerAdmissionDenial> {
        Self::admit_with_schedule(root, MediaFaultSchedule::default())
    }

    pub(in crate::filesystem_media) fn admit_with_schedule(
        root: &Path,
        schedule: MediaFaultSchedule,
    ) -> Result<Self, FilesystemMediaOwnerAdmissionDenial> {
        Self::admit_with_observation(root, schedule).map_err(|failure| failure.denial)
    }

    pub(in crate::filesystem_media) fn admit_with_observation(
        root: &Path,
        schedule: MediaFaultSchedule,
    ) -> Result<Self, ObservedMediaOwnerAdmissionFailure> {
        let counters = Arc::new(super::super::operation_counters::MediaCounterCells::default());
        let boundary = super::super::fault_interposition::MediaFaultInterposer::new(
            schedule,
            Arc::clone(&counters),
        );
        Self::admit_with_boundary(root, boundary, counters)
    }

    pub(in crate::filesystem_media) fn admit_with_boundary(
        root: &Path,
        boundary: super::super::fault_interposition::MediaFaultInterposer,
        counters: Arc<super::super::operation_counters::MediaCounterCells>,
    ) -> Result<Self, ObservedMediaOwnerAdmissionFailure> {
        Self::admit_with_policy(root, boundary, counters, OrdinaryNamespaceAdmission)
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::filesystem_media) fn admit_existing_with_boundary(
        root: &Path,
        boundary: super::super::fault_interposition::MediaFaultInterposer,
        counters: Arc<super::super::operation_counters::MediaCounterCells>,
    ) -> Result<Self, ObservedMediaOwnerAdmissionFailure> {
        Self::admit_with_policy(root, boundary, counters, ExistingRecoveryNamespaceAdmission)
    }

    fn admit_with_policy<P: NamespaceAdmissionPolicy>(
        root: &Path,
        boundary: super::super::fault_interposition::MediaFaultInterposer,
        counters: Arc<super::super::operation_counters::MediaCounterCells>,
        policy: P,
    ) -> Result<Self, ObservedMediaOwnerAdmissionFailure> {
        counters.ownership_attempt();
        let mut namespace = policy.open_namespace(root, &boundary, &counters)?;
        let namespace_effect_fate = namespace.admission_effect_fate();
        let namespace_directory = match namespace
            .open_role_directory(StoreNamespaceRelativeRole::NamespaceDirectory, &boundary)
        {
            Ok(directory) => directory,
            Err(denial) => {
                drop(namespace);
                return Err(ObservedMediaOwnerAdmissionFailure {
                    denial: FilesystemMediaOwnerAdmissionDenial::Confinement(denial),
                    counters: counters.snapshot(),
                    effect_fate: namespace_effect_fate,
                    release: None,
                });
            }
        };
        let (namespace, namespace_directory, mutation_lease) = acquire_owned_mutation_lease(
            policy,
            namespace,
            namespace_directory,
            &boundary,
            &counters,
            namespace_effect_fate,
        )?;
        let (namespace, namespace_directory, mutation_lease, families, staging) =
            open_owned_artifact_directories(
                namespace,
                namespace_directory,
                mutation_lease,
                &boundary,
                &counters,
            )?;
        let store_root_publication_required = namespace.store_root_publication_required();
        let root_parent_publication_required = namespace.root_parent_publication_required();
        Ok(Self {
            boundary,
            namespace,
            namespace_directory,
            families,
            staging,
            mutation_lease,
            namespace_mutation_sequence: std::sync::Mutex::new(()),
            next_file_handle_generation: AtomicU64::new(5),
            next_operation_identity: AtomicU64::new(1),
            file_mutation_sequences: Default::default(),
            artifact_mutations: Arc::new(Default::default()),
            store_root_publication_required: AtomicBool::new(store_root_publication_required),
            root_parent_publication_required: AtomicBool::new(root_parent_publication_required),
            #[cfg(feature = "certification-test-authority")]
            fail_next_removal_directory_sync: AtomicBool::new(false),
        })
    }
}

trait NamespaceAdmissionPolicy: Copy {
    fn open_namespace(
        self,
        root: &Path,
        boundary: &super::super::fault_interposition::MediaFaultInterposer,
        counters: &Arc<super::super::operation_counters::MediaCounterCells>,
    ) -> Result<super::super::AdmittedStoreNamespace, ObservedMediaOwnerAdmissionFailure>;

    fn acquire_mutation_lease(
        self,
        namespace: &super::super::AdmittedStoreNamespace,
        directory: &super::super::NamespaceDirectoryHandle,
        boundary: &super::super::fault_interposition::MediaFaultInterposer,
    ) -> Result<
        MutationOwnershipLease,
        super::super::owner_admission_effect::MutationOwnershipAcquisitionFailure,
    >;
}

#[derive(Clone, Copy)]
struct OrdinaryNamespaceAdmission;

#[cfg(feature = "recovery-runtime-owner")]
#[derive(Clone, Copy)]
struct ExistingRecoveryNamespaceAdmission;

impl NamespaceAdmissionPolicy for OrdinaryNamespaceAdmission {
    fn open_namespace(
        self,
        root: &Path,
        boundary: &super::super::fault_interposition::MediaFaultInterposer,
        counters: &Arc<super::super::operation_counters::MediaCounterCells>,
    ) -> Result<super::super::AdmittedStoreNamespace, ObservedMediaOwnerAdmissionFailure> {
        map_namespace_admission(
            super::super::namespace_admission::create_or_open(root, boundary),
            counters,
        )
    }

    fn acquire_mutation_lease(
        self,
        namespace: &super::super::AdmittedStoreNamespace,
        directory: &super::super::NamespaceDirectoryHandle,
        boundary: &super::super::fault_interposition::MediaFaultInterposer,
    ) -> Result<
        MutationOwnershipLease,
        super::super::owner_admission_effect::MutationOwnershipAcquisitionFailure,
    > {
        MutationOwnershipLease::try_acquire_or_create(
            namespace.owner_identity(),
            directory,
            boundary,
        )
    }
}

#[cfg(feature = "recovery-runtime-owner")]
impl NamespaceAdmissionPolicy for ExistingRecoveryNamespaceAdmission {
    fn open_namespace(
        self,
        root: &Path,
        boundary: &super::super::fault_interposition::MediaFaultInterposer,
        counters: &Arc<super::super::operation_counters::MediaCounterCells>,
    ) -> Result<super::super::AdmittedStoreNamespace, ObservedMediaOwnerAdmissionFailure> {
        map_namespace_admission(
            super::super::namespace_admission::open_existing(root, boundary),
            counters,
        )
    }

    fn acquire_mutation_lease(
        self,
        namespace: &super::super::AdmittedStoreNamespace,
        directory: &super::super::NamespaceDirectoryHandle,
        boundary: &super::super::fault_interposition::MediaFaultInterposer,
    ) -> Result<
        MutationOwnershipLease,
        super::super::owner_admission_effect::MutationOwnershipAcquisitionFailure,
    > {
        MutationOwnershipLease::try_acquire_existing(
            namespace.owner_identity(),
            directory,
            boundary,
        )
    }
}

fn map_namespace_admission(
    result: Result<
        super::super::AdmittedStoreNamespace,
        super::super::owner_admission_effect::NamespaceAdmissionFailure,
    >,
    counters: &Arc<super::super::operation_counters::MediaCounterCells>,
) -> Result<super::super::AdmittedStoreNamespace, ObservedMediaOwnerAdmissionFailure> {
    result.map_err(|failure| ObservedMediaOwnerAdmissionFailure {
        denial: FilesystemMediaOwnerAdmissionDenial::Confinement(failure.denial()),
        counters: counters.snapshot(),
        effect_fate: failure.effect_fate(),
        release: None,
    })
}

fn acquire_owned_mutation_lease<P: NamespaceAdmissionPolicy>(
    policy: P,
    namespace: super::super::AdmittedStoreNamespace,
    directory: super::super::NamespaceDirectoryHandle,
    boundary: &super::super::fault_interposition::MediaFaultInterposer,
    counters: &Arc<super::super::operation_counters::MediaCounterCells>,
    namespace_effect_fate: super::super::owner_admission_effect::MediaOwnerAdmissionEffectFate,
) -> Result<
    (
        super::super::AdmittedStoreNamespace,
        super::super::NamespaceDirectoryHandle,
        MutationOwnershipLease,
    ),
    ObservedMediaOwnerAdmissionFailure,
> {
    match policy.acquire_mutation_lease(&namespace, &directory, boundary) {
        Ok(lease) => Ok((namespace, directory, lease)),
        Err(failure) => {
            let denial = failure.denial();
            if denial == MutationOwnershipDenial::Contended {
                counters.ownership_contended();
            }
            let effect_fate = namespace_effect_fate.combine(failure.effect_fate());
            let release = failure.release();
            drop(directory);
            drop(namespace);
            Err(ObservedMediaOwnerAdmissionFailure {
                denial: FilesystemMediaOwnerAdmissionDenial::Ownership(denial),
                counters: counters.snapshot(),
                effect_fate,
                release,
            })
        }
    }
}

fn open_owned_artifact_directories(
    mut namespace: super::super::AdmittedStoreNamespace,
    directory: super::super::NamespaceDirectoryHandle,
    mutation_lease: MutationOwnershipLease,
    boundary: &super::super::fault_interposition::MediaFaultInterposer,
    counters: &Arc<super::super::operation_counters::MediaCounterCells>,
) -> Result<
    (
        super::super::AdmittedStoreNamespace,
        super::super::NamespaceDirectoryHandle,
        MutationOwnershipLease,
        ArtifactFamilyDirectory,
        StagingDirectory,
    ),
    ObservedMediaOwnerAdmissionFailure,
> {
    let families = match namespace
        .open_role_directory(StoreNamespaceRelativeRole::FamiliesDirectory, boundary)
        .map(ArtifactFamilyDirectory::new)
    {
        Ok(families) => families,
        Err(denial) => {
            drop(directory);
            drop(namespace);
            let release = mutation_lease.release(boundary);
            return Err(artifact_directory_failure(denial, release, counters));
        }
    };
    let staging = match namespace
        .open_role_directory(StoreNamespaceRelativeRole::StagingDirectory, boundary)
        .map(StagingDirectory::new)
    {
        Ok(staging) => staging,
        Err(denial) => {
            drop(families);
            drop(directory);
            drop(namespace);
            let release = mutation_lease.release(boundary);
            return Err(artifact_directory_failure(denial, release, counters));
        }
    };
    Ok((namespace, directory, mutation_lease, families, staging))
}

fn artifact_directory_failure(
    denial: super::super::NamespaceConfinementDenial,
    release: super::super::OwnershipReleaseOutcome,
    counters: &Arc<super::super::operation_counters::MediaCounterCells>,
) -> ObservedMediaOwnerAdmissionFailure {
    ObservedMediaOwnerAdmissionFailure {
        denial: FilesystemMediaOwnerAdmissionDenial::Confinement(denial),
        counters: counters.snapshot(),
        effect_fate:
            super::super::owner_admission_effect::MediaOwnerAdmissionEffectFate::EffectPossible,
        release: Some(release),
    }
}
