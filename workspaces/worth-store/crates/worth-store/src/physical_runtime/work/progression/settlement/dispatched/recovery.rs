use worth_store_physical_backend::{
    CompletedRecoveryStagingWrite, IndeterminateRecoveryStagingWrite,
    RecoveryStagingIndeterminatePhysical,
};

use super::DispatchedPhysicalWork;
use crate::physical_runtime::work::PhysicalWorkOperationFamily;

impl DispatchedPhysicalWork {
    pub(in crate::physical_runtime) fn matches_recovery_staging(
        &self,
        physical: &CompletedRecoveryStagingWrite,
    ) -> bool {
        let coordinate = physical.coordinate();
        let common = Some(coordinate) == self.coordinate()
            && self.payload_digest == Some(physical.payload_digest())
            && physical.byte_count() == u64::from(coordinate.length())
            && matches!(
                self.intent().operation(),
                PhysicalWorkOperationFamily::ArtifactRangeWrite
                    | PhysicalWorkOperationFamily::RootPublication
            );
        if !common {
            return false;
        }
        match physical.disposition() {
            worth_store_physical_backend::RecoveryStagingWriteDisposition::Created => physical
                .created()
                .is_some_and(|created| self.matches_new_artifact_binding(created, coordinate)),
            worth_store_physical_backend::RecoveryStagingWriteDisposition::AlreadyMaterialized => {
                let Some(verified) = physical.verified() else {
                    return false;
                };
                verified.store() == self.intent().identity().store()
                    && verified.owner()
                        == self.admitted.authority().media_owner_observation().owner()
                    && verified.coordinate() == coordinate
                    && verified.completed_bytes() == u64::from(coordinate.length())
            }
            worth_store_physical_backend::RecoveryStagingWriteDisposition::CompletedFromExactPrefix => {
                self.matches_completed_prefix(physical, coordinate)
            }
        }
    }

    pub(in crate::physical_runtime) fn matches_recovery_staging_indeterminate(
        &self,
        physical: &IndeterminateRecoveryStagingWrite,
    ) -> bool {
        self.coordinate().is_some_and(|coordinate| {
            if physical.artifact() != coordinate.artifact() {
                return false;
            }
            match physical.evidence() {
                RecoveryStagingIndeterminatePhysical::NewArtifact(new_artifact) => {
                    self.matches_new_artifact_indeterminate_binding(new_artifact, coordinate)
                }
                RecoveryStagingIndeterminatePhysical::Append {
                    prefix_verified,
                    append,
                } => self.matches_append_indeterminate_binding(
                    physical,
                    coordinate,
                    *prefix_verified,
                    append,
                ),
            }
        })
    }

    fn matches_completed_prefix(
        &self,
        physical: &CompletedRecoveryStagingWrite,
        coordinate: worth_store_physical_format::RecordFrameCoordinate,
    ) -> bool {
        let expected = u64::from(coordinate.length());
        let prefix_bytes = physical
            .prefix_verified()
            .map_or(0, |prefix| prefix.completed_bytes());
        if let Some(prefix) = physical.prefix_verified() {
            if prefix.store() != self.intent().identity().store()
                || prefix.owner() != self.admitted.authority().media_owner_observation().owner()
                || prefix.coordinate().artifact() != coordinate.artifact()
                || prefix.coordinate().offset() != 0
                || prefix.coordinate().length() as u64 != prefix_bytes
                || prefix_bytes >= expected
                || prefix.completed_bytes() != prefix_bytes
            {
                return false;
            }
        }
        let Some(appended) = physical.appended() else {
            return false;
        };
        appended.store() == self.intent().identity().store()
            && appended.owner() == self.admitted.authority().media_owner_observation().owner()
            && appended.range().offset() == prefix_bytes
            && appended.range().byte_count() == expected.saturating_sub(prefix_bytes)
    }

    fn matches_append_indeterminate_binding(
        &self,
        physical: &IndeterminateRecoveryStagingWrite,
        coordinate: worth_store_physical_format::RecordFrameCoordinate,
        prefix_verified: Option<worth_store_physical_backend::CompletedArtifactRangeRead>,
        append: &worth_store_physical_backend::IndeterminateArtifactAppend,
    ) -> bool {
        let expected = u64::from(coordinate.length());
        let prefix_bytes = prefix_verified.map_or(0, |prefix| prefix.completed_bytes());
        if let Some(prefix) = prefix_verified {
            if prefix.store() != self.intent().identity().store()
                || prefix.owner() != self.admitted.authority().media_owner_observation().owner()
                || prefix.coordinate().artifact() != coordinate.artifact()
                || prefix.coordinate().offset() != 0
                || prefix.coordinate().length() as u64 != prefix_bytes
                || prefix_bytes >= expected
                || prefix.completed_bytes() != prefix_bytes
            {
                return false;
            }
        }
        append.store() == self.intent().identity().store()
            && append.owner() == self.admitted.authority().media_owner_observation().owner()
            && append.range().offset() == prefix_bytes
            && append.range().byte_count() == expected.saturating_sub(prefix_bytes)
            && append.completed_bytes() <= append.range().byte_count()
            && Some(physical.payload_digest()) == self.payload_digest
    }
}
