use worth_store_buffer_pool::PhysicalOperationAllocationScope;
use worth_store_physical_backend::QualifiedFilesystemMedia;

mod record_serving;
mod retirement_residue;
mod work_runtime;

use record_serving::PhysicalRecordServingAssembly;
use work_runtime::prepare_work_runtime;

use crate::physical_runtime::{
    artifact_family::PhysicalArtifactFamilyRegistry,
    record_serving::{RecordAllocationFrontier, RecordServingOwner, RecordServingState},
    runtime::PhysicalRuntimeCore,
};

use super::{
    reopen_durability_basis, PhysicalResidencyOwner, PhysicalSignalConstructionFailure,
    PhysicalStoreInstanceParts,
};

pub(in crate::physical_runtime) struct PhysicalStoreInstanceFoundation {
    pub(in crate::physical_runtime) checkpoint_custody: super::OpenedCheckpointCustody,
    pub(in crate::physical_runtime) termination:
        crate::physical_runtime::lifecycle::LifecycleTerminationGuard,
    pub(in crate::physical_runtime) read_protection:
        crate::physical_runtime::stability::PhysicalReadProtectionOwner,
    pub(in crate::physical_runtime) media: QualifiedFilesystemMedia,
    pub(in crate::physical_runtime) core: PhysicalRuntimeCore,
    pub(in crate::physical_runtime) bootstrap: RecordServingState,
    pub(in crate::physical_runtime) allocation_frontier: RecordAllocationFrontier,
    pub(in crate::physical_runtime) residency: PhysicalResidencyOwner,
    pub(in crate::physical_runtime) work_profile:
        crate::physical_runtime::PhysicalWorkProfileDeclaration,
    pub(in crate::physical_runtime) durability:
        crate::physical_runtime::durability::PhysicalDurabilityRuntimeOwner,
}

pub(in crate::physical_runtime) struct PhysicalStoreInstanceConstructionFailure {
    termination: crate::physical_runtime::lifecycle::LifecycleTerminationGuard,
    read_protection: crate::physical_runtime::stability::PhysicalReadProtectionOwner,
    media: QualifiedFilesystemMedia,
    core: PhysicalRuntimeCore,
    residency: PhysicalResidencyOwner,
    durability: crate::physical_runtime::durability::PhysicalDurabilityRuntimeOwner,
    cause: PhysicalSignalConstructionFailure,
}

impl PhysicalStoreInstanceParts {
    pub(in crate::physical_runtime) fn from_record_admission(
        foundation: PhysicalStoreInstanceFoundation,
    ) -> Result<Self, PhysicalStoreInstanceConstructionFailure> {
        let PhysicalStoreInstanceFoundation {
            checkpoint_custody,
            termination,
            read_protection,
            media,
            core,
            bootstrap,
            allocation_frontier,
            residency,
            work_profile,
            durability,
        } = foundation;
        let frame_ports = residency.ports().clone();
        let runtime_identity = core.runtime_identity();
        let lifecycle_generation = core.lifecycle_generation();
        let record_owner = RecordServingOwner::new();
        let prepared_work =
            match prepare_work_runtime(&media, &core, work_profile, durability.observation()) {
                Ok(prepared) => prepared,
                Err(cause) => {
                    return Err(PhysicalStoreInstanceConstructionFailure {
                        termination,
                        read_protection,
                        media,
                        core,
                        residency,
                        durability,
                        cause,
                    })
                }
            };
        let signal_profile = prepared_work.signal_profile();
        let reopen_grant = match std::num::NonZeroU64::new(
            residency.available_recovery_operation_bytes(),
        )
        .ok_or(())
        .and_then(|bytes| {
            residency
                .ports()
                .begin_operation(PhysicalOperationAllocationScope::Recovery, bytes)
                .map_err(|_| ())
        }) {
            Ok(grant) => grant,
            Err(()) => {
                return Err(PhysicalStoreInstanceConstructionFailure {
                    termination,
                    read_protection,
                    media,
                    core,
                    residency,
                    durability,
                    cause: PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                        super::PhysicalDurabilityStateReopenFailure::Wal(
                            crate::physical_runtime::PhysicalWalOpenFailure::ReopenAllocationRejected,
                        ),
                    ),
                });
            }
        };
        let durability_reopen = match reopen_durability_basis(
            &media,
            runtime_identity,
            signal_profile,
            &durability,
            bootstrap.format.declaration(),
            &reopen_grant,
            &checkpoint_custody,
        ) {
            Ok(reopened) => reopened,
            Err(failure) => {
                return Err(PhysicalStoreInstanceConstructionFailure {
                    termination,
                    read_protection,
                    media,
                    core,
                    residency,
                    durability,
                    cause: PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                        failure,
                    ),
                })
            }
        };
        let retained_wal_tail = durability
            .observation()
            .checkpoint_policy()
            .retained_wal_tail_limit()
            .get()
            .get();
        let publication_retention =
            match crate::physical_runtime::record_serving::AdmittedPublicationRetention::admit(
                durability_reopen.wal(),
                &bootstrap.publication_overheads,
                crate::physical_runtime::durability::PhysicalRetentionProfile::store_default()
                    .covering_retained_wal_tail(retained_wal_tail),
                &reopen_grant,
            ) {
                Ok(admitted) => admitted,
                Err(()) => {
                    return Err(PhysicalStoreInstanceConstructionFailure {
                    termination,
                    read_protection,
                    media,
                    core,
                    residency,
                    durability,
                    cause: PhysicalSignalConstructionFailure::DurabilityStateReopenRejected(
                        super::PhysicalDurabilityStateReopenFailure::PublicationRetentionRejected,
                    ),
                });
                }
            };
        // Keep this grant through publication-retention prevalidation, then
        // release it before ordinary serving installs operation grants.
        drop(reopen_grant);
        let checkpoint_custody_origin = durability_reopen.checkpoint_custody_origin();
        let reopened = durability_reopen.install(durability);
        prepared_work.admit_publication_residue(
            retirement_residue::PublicationResidueAdmission::classify(&bootstrap, &reopened),
        );
        let installed_work = prepared_work.install(media);
        let lifecycle_state = core.lifecycle_state();
        // Recovered Serving carries the original Store-issued ceiling, while
        // this owner's pool independently enforces the current tighter limit.
        let selected_recovery_allocation = residency.recovery_allocation_admission();
        let record_serving = PhysicalRecordServingAssembly::new(
            bootstrap,
            checkpoint_custody_origin,
            checkpoint_custody.recovered,
            allocation_frontier,
            frame_ports,
            selected_recovery_allocation,
            lifecycle_generation,
            signal_profile,
            lifecycle_state,
        )
        .install(
            &installed_work,
            &reopened,
            &record_owner,
            read_protection.registry(),
            publication_retention,
        );

        Ok(Self {
            termination,
            read_protection,
            work_admission: installed_work.admission,
            work_runtime: installed_work.runtime,
            scheduler_admission: installed_work.scheduler,
            record_owner,
            artifact_families: PhysicalArtifactFamilyRegistry::install(),
            record_work: installed_work.record_work,
            core,
            format: record_serving.format,
            access: record_serving.access,
            publication: record_serving.publication,
            checkpoint: record_serving.checkpoint,
            root_protocol_counters: record_serving.root_protocol_counters,
            residency,
            durability: reopened.durability,
        })
    }
}

impl PhysicalStoreInstanceConstructionFailure {
    pub(in crate::physical_runtime) fn abort(
        self,
    ) -> (
        crate::physical_runtime::RuntimeIdentity,
        crate::physical_runtime::MediaShutdownOutcome<crate::physical_runtime::AbortedRuntime>,
        PhysicalSignalConstructionFailure,
    ) {
        let identity = self.core.runtime_identity();
        let _read_protection = self.read_protection.close();
        let _residency = self.residency.close();
        drop(self.termination);
        drop(self.durability);
        let release = self.media.close();
        let terminal =
            crate::physical_runtime::MediaShutdownOutcome::new(self.core.abort(), release);
        (identity, terminal, self.cause)
    }
}
