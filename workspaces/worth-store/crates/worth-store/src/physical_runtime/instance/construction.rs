use worth_store_physical_backend::QualifiedFilesystemMedia;

mod record_serving;
mod work_runtime;

use record_serving::PhysicalRecordServingAssembly;
use work_runtime::prepare_work_runtime;

use crate::physical_runtime::{
    record_serving::{RecordAllocationFrontier, RecordServingOwner, RecordServingState},
    runtime::PhysicalRuntimeCore,
};

use super::{
    reopen_durability_basis, PhysicalResidencyOwner, PhysicalSignalConstructionFailure,
    PhysicalStoreInstanceParts,
};

pub(in crate::physical_runtime) struct PhysicalStoreInstanceFoundation {
    pub(in crate::physical_runtime) termination:
        crate::physical_runtime::lifecycle::LifecycleTerminationGuard,
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
            termination,
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
        let prepared_work = match prepare_work_runtime(
            &media,
            &core,
            work_profile,
            durability.observation(),
            !bootstrap.publication_residue.is_empty(),
        ) {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err(PhysicalStoreInstanceConstructionFailure {
                    termination,
                    media,
                    core,
                    residency,
                    durability,
                    cause,
                })
            }
        };
        let signal_profile = prepared_work.signal_profile();
        let durability_reopen =
            match reopen_durability_basis(&media, runtime_identity, signal_profile, &durability) {
                Ok(reopened) => reopened,
                Err(failure) => {
                    return Err(PhysicalStoreInstanceConstructionFailure {
                        termination,
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
        let reopened = durability_reopen.install(durability);
        let installed_work = prepared_work.install(media);
        let lifecycle_state = core.lifecycle_state();
        let record_serving = PhysicalRecordServingAssembly::new(
            bootstrap,
            allocation_frontier,
            frame_ports,
            lifecycle_generation,
            signal_profile,
            lifecycle_state,
        )
        .install(&installed_work, &reopened);

        Ok(Self {
            termination,
            work_admission: installed_work.admission,
            work_runtime: installed_work.runtime,
            scheduler_admission: installed_work.scheduler,
            record_owner,
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
        let _residency = self.residency.close();
        drop(self.termination);
        drop(self.durability);
        let release = self.media.close();
        let terminal =
            crate::physical_runtime::MediaShutdownOutcome::new(self.core.abort(), release);
        (identity, terminal, self.cause)
    }
}
