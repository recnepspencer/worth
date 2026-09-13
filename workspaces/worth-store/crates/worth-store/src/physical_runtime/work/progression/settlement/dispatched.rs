use worth_signal::facade::{ResourceAttemptId, ResourceRequestHandle};
use worth_store_physical_backend::{
    ArtifactTreePublicationEffect, BackendQueueExecutionPlanBinding, CompletedArtifactAppend,
    CompletedArtifactMetadataRead, CompletedArtifactNewWrite, CompletedArtifactRangeRead,
    CompletedArtifactRangeWrite, IndeterminateArtifactAppend, IndeterminateArtifactNewWrite,
    IndeterminateArtifactRangeWrite,
};
use worth_store_physical_format::RecordFrameCoordinate;

use super::super::{AdmittedPhysicalWork, PhysicalSignalReadinessEvidence};
use crate::physical_runtime::work::{
    PhysicalPublicationEffect, PhysicalRootPublicationWorkAction, PhysicalWorkOperationFamily,
};

mod inspection_binding;
#[cfg(feature = "recovery-runtime-owner")]
mod recovery;

pub struct DispatchedPhysicalWork {
    pub(in crate::physical_runtime::work::progression) inspection_source:
        Option<worth_store_physical_backend::ArtifactTreeFile>,
    pub(in crate::physical_runtime::work::progression) admitted: AdmittedPhysicalWork,
    pub(in crate::physical_runtime::work::progression) signal: PhysicalSignalReadinessEvidence,
    pub(in crate::physical_runtime::work::progression) effect_activity:
        Option<crate::physical_runtime::work::submission::PhysicalEffectActivity>,
    pub(in crate::physical_runtime::work::progression) scheduler_capacity: Option<
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacityLease,
    >,
    pub(in crate::physical_runtime::work::progression) scheduler_binding:
        BackendQueueExecutionPlanBinding,
    pub(in crate::physical_runtime::work::progression) payload_digest: Option<[u8; 32]>,
}

impl DispatchedPhysicalWork {
    pub(in crate::physical_runtime::work) fn take_effect_activity(
        &mut self,
    ) -> crate::physical_runtime::work::submission::PhysicalEffectActivity {
        self.effect_activity
            .take()
            .expect("dispatched work owns one active-effect enrollment")
    }

    pub(super) fn release_scheduler_capacity(&mut self) {
        drop(self.scheduler_capacity.take());
    }

    pub const fn intent(&self) -> &crate::physical_runtime::work::PhysicalWorkIntent {
        self.admitted.intent()
    }

    pub const fn signal_request(&self) -> ResourceRequestHandle {
        self.signal.signal_request
    }

    pub const fn request_attempt(&self) -> ResourceAttemptId {
        self.signal.attempt
    }

    pub(in crate::physical_runtime) const fn signal_evidence(
        &self,
    ) -> &PhysicalSignalReadinessEvidence {
        &self.signal
    }

    pub const fn scheduler_binding(&self) -> BackendQueueExecutionPlanBinding {
        self.scheduler_binding
    }

    pub fn coordinate(&self) -> Option<RecordFrameCoordinate> {
        let [coordinate] = self.intent().scope().coordinates() else {
            return None;
        };
        Some(*coordinate)
    }

    pub(in crate::physical_runtime) fn matches_metadata(
        &self,
        physical: CompletedArtifactMetadataRead,
    ) -> bool {
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && self.intent().scope().artifact_target() == Some(physical.artifact())
            && self.intent().operation() == PhysicalWorkOperationFamily::ArtifactMetadataRead
    }

    pub(in crate::physical_runtime) fn matches_read(
        &self,
        physical: CompletedArtifactRangeRead,
    ) -> bool {
        let coordinate = physical.coordinate();
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && (Some(coordinate) == self.coordinate()
                || self.intent().scope().artifact_target() == Some(coordinate.artifact()))
            && physical.completed_bytes() <= u64::from(coordinate.length())
            && self.intent().operation() == PhysicalWorkOperationFamily::ArtifactRangeRead
    }

    pub(in crate::physical_runtime) fn matches_write(
        &self,
        physical: &CompletedArtifactRangeWrite,
    ) -> bool {
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && Some(physical.coordinate()) == self.coordinate()
            && physical.completed_bytes() == u64::from(physical.coordinate().length())
            && self.payload_digest == Some(physical.payload_digest())
            && crate::physical_runtime::work::execution::settlement::durability_satisfies(
                self.intent().durability(),
                physical.durability(),
            )
    }

    pub(in crate::physical_runtime) fn matches_indeterminate(
        &self,
        physical: IndeterminateArtifactRangeWrite,
    ) -> bool {
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && Some(physical.coordinate()) == self.coordinate()
            && physical.completed_bytes() <= u64::from(physical.coordinate().length())
            && self.payload_digest == Some(physical.payload_digest())
            && self.intent().operation() != PhysicalWorkOperationFamily::ArtifactRangeRead
    }

    pub(in crate::physical_runtime) fn matches_new_artifact(
        &self,
        physical: &CompletedArtifactNewWrite,
        coordinate: RecordFrameCoordinate,
    ) -> bool {
        self.matches_new_artifact_binding(physical, coordinate)
            && self.intent().operation() == PhysicalWorkOperationFamily::ArtifactPublication
    }

    fn matches_new_artifact_binding(
        &self,
        physical: &CompletedArtifactNewWrite,
        coordinate: RecordFrameCoordinate,
    ) -> bool {
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && Some(coordinate) == self.coordinate()
            && physical.range().byte_count() == u64::from(coordinate.length())
            && physical.completed_bytes() == u64::from(coordinate.length())
            && self.payload_digest == Some(physical.payload_digest())
            && physical.create_operation() != physical.write_operation()
    }

    pub(in crate::physical_runtime) fn matches_wal_append(
        &self,
        physical: &CompletedArtifactAppend,
    ) -> bool {
        let Some(scope) = self.intent().scope().wal_append_target() else {
            return false;
        };
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && physical.range().offset() == scope.offset()
            && physical.range().byte_count() == scope.byte_count()
            && self.payload_digest == Some(physical.payload_digest())
            && self.intent().operation() == PhysicalWorkOperationFamily::WalAppend
            && matches!(
                scope.disposition(),
                crate::physical_runtime::PhysicalWalFrameWriteDisposition::AppendExistingSegment
            )
    }

    pub(in crate::physical_runtime) fn matches_wal_segment_create(
        &self,
        physical: &CompletedArtifactNewWrite,
    ) -> bool {
        let Some(scope) = self.intent().scope().wal_append_target() else {
            return false;
        };
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && physical.range().byte_count() == scope.byte_count()
            && physical.completed_bytes() == scope.byte_count()
            && self.payload_digest == Some(physical.payload_digest())
            && self.intent().operation() == PhysicalWorkOperationFamily::WalAppend
            && matches!(
                scope.disposition(),
                crate::physical_runtime::PhysicalWalFrameWriteDisposition::CreateSegment
            )
            && physical.create_operation() != physical.write_operation()
    }

    pub(in crate::physical_runtime) fn matches_wal_append_indeterminate(
        &self,
        physical: &IndeterminateArtifactAppend,
    ) -> bool {
        let Some(scope) = self.intent().scope().wal_append_target() else {
            return false;
        };
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && physical.range().offset() == scope.offset()
            && physical.range().byte_count() == scope.byte_count()
            && physical.completed_bytes() <= scope.byte_count()
            && self.payload_digest == Some(physical.payload_digest())
            && self.intent().operation() == PhysicalWorkOperationFamily::WalAppend
            && matches!(
                scope.disposition(),
                crate::physical_runtime::PhysicalWalFrameWriteDisposition::AppendExistingSegment
            )
    }

    pub(in crate::physical_runtime) fn matches_wal_segment_create_indeterminate(
        &self,
        physical: &IndeterminateArtifactNewWrite,
    ) -> bool {
        let Some(scope) = self.intent().scope().wal_append_target() else {
            return false;
        };
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && physical.range().byte_count() == scope.byte_count()
            && physical.completed_bytes() <= scope.byte_count()
            && self.payload_digest == Some(physical.payload_digest())
            && self.intent().operation() == PhysicalWorkOperationFamily::WalAppend
            && matches!(
                scope.disposition(),
                crate::physical_runtime::PhysicalWalFrameWriteDisposition::CreateSegment
            )
    }

    pub(in crate::physical_runtime) fn matches_wal_barrier(
        &self,
        physical: &crate::physical_runtime::work::CompletedPhysicalWalBarrier,
    ) -> bool {
        self.matches_wal_barrier_binding(
            physical.physical().store(),
            physical.physical().owner(),
            physical.artifact(),
            physical.physical().effect(),
        )
    }

    pub(in crate::physical_runtime) fn matches_wal_barrier_indeterminate(
        &self,
        physical: &crate::physical_runtime::work::IndeterminatePhysicalWalBarrier,
    ) -> bool {
        self.matches_wal_barrier_binding(
            physical.physical().store(),
            physical.physical().owner(),
            physical.artifact(),
            physical.physical().effect(),
        )
    }

    fn matches_wal_barrier_binding(
        &self,
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        owner: worth_store_physical_backend::MediaOwnerIdentity,
        artifact: &worth_store_physical_backend::ArtifactTreeFile,
        effect: &ArtifactTreePublicationEffect,
    ) -> bool {
        store == self.intent().identity().store()
            && owner == self.admitted.authority().media_owner_observation().owner()
            && self.intent().scope().wal_barrier_target().is_some()
            && self.intent().operation() == PhysicalWorkOperationFamily::DurabilityBarrier
            && matches!(
                effect,
                ArtifactTreePublicationEffect::FileSynchronization(observed)
                    if observed == artifact
            )
    }

    pub(in crate::physical_runtime) fn matches_new_artifact_indeterminate(
        &self,
        physical: &IndeterminateArtifactNewWrite,
        coordinate: RecordFrameCoordinate,
    ) -> bool {
        self.matches_new_artifact_indeterminate_binding(physical, coordinate)
            && self.intent().operation() == PhysicalWorkOperationFamily::ArtifactPublication
    }

    fn matches_new_artifact_indeterminate_binding(
        &self,
        physical: &IndeterminateArtifactNewWrite,
        coordinate: RecordFrameCoordinate,
    ) -> bool {
        physical.store() == self.intent().identity().store()
            && physical.owner() == self.admitted.authority().media_owner_observation().owner()
            && Some(coordinate) == self.coordinate()
            && physical.range().byte_count() == u64::from(coordinate.length())
            && physical.completed_bytes() <= u64::from(coordinate.length())
            && self.payload_digest == Some(physical.payload_digest())
    }

    pub(in crate::physical_runtime) fn matches_publication_effect(
        &self,
        physical: &crate::physical_runtime::work::CompletedPhysicalPublicationEffect,
    ) -> bool {
        physical.physical().store() == self.intent().identity().store()
            && physical.physical().owner()
                == self.admitted.authority().media_owner_observation().owner()
            && declared_publication_effect_matches(
                self.intent(),
                physical.artifact(),
                physical.effect(),
            )
            && publication_effect_matches(physical.effect(), physical.physical().effect())
    }

    pub(in crate::physical_runtime) fn matches_publication_effect_indeterminate(
        &self,
        physical: &crate::physical_runtime::work::IndeterminatePhysicalPublicationEffect,
    ) -> bool {
        physical.physical().store() == self.intent().identity().store()
            && physical.physical().owner()
                == self.admitted.authority().media_owner_observation().owner()
            && declared_publication_effect_matches(
                self.intent(),
                physical.artifact(),
                physical.effect(),
            )
            && publication_effect_matches(physical.effect(), physical.physical().effect())
    }
}

fn declared_publication_effect_matches(
    intent: &crate::physical_runtime::PhysicalWorkIntent,
    artifact: worth_store_physical_format::RecordArtifactFile,
    effect: PhysicalPublicationEffect,
) -> bool {
    match intent.operation() {
        PhysicalWorkOperationFamily::ArtifactPublication => {
            intent.scope().artifact_target() == Some(artifact)
        }
        PhysicalWorkOperationFamily::RootPublication => {
            if let Some(scope) = intent.scope().root_publication_target() {
                return match scope.action() {
                    PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact {
                        artifact: expected,
                    } => {
                        expected == artifact
                            && effect == PhysicalPublicationEffect::SynchronizeArtifact
                    }
                    PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog => {
                        scope.publication().catalog_candidate() == artifact
                            && effect == PhysicalPublicationEffect::ReplaceCatalog
                    }
                    PhysicalRootPublicationWorkAction::SynchronizeParentNamespace => {
                        artifact
                            == worth_store_physical_format::RecordArtifactFile::BootstrapCatalog
                            && effect == PhysicalPublicationEffect::SynchronizeRecordFamily
                    }
                };
            }
            intent.scope().artifact_target() == Some(artifact)
                && matches!(
                    effect,
                    PhysicalPublicationEffect::SynchronizeArtifact
                        | PhysicalPublicationEffect::ReplaceCatalog
                        | PhysicalPublicationEffect::SynchronizeRecordFamily
                )
        }
        _ => false,
    }
}

fn publication_effect_matches(
    declared: PhysicalPublicationEffect,
    observed: &ArtifactTreePublicationEffect,
) -> bool {
    matches!(
        (declared, observed),
        (
            PhysicalPublicationEffect::SynchronizeArtifact,
            ArtifactTreePublicationEffect::FileSynchronization(_)
        ) | (
            PhysicalPublicationEffect::SynchronizeArtifactParent
                | PhysicalPublicationEffect::SynchronizeRecordFamily,
            ArtifactTreePublicationEffect::DirectorySynchronization(_)
        ) | (
            PhysicalPublicationEffect::ReplaceCatalog,
            ArtifactTreePublicationEffect::Replacement { .. }
                | ArtifactTreePublicationEffect::RootProtocolReplacement { .. }
        )
    )
}
