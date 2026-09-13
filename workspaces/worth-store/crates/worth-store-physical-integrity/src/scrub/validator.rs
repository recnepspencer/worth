use crate::*;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily as Family;

/// Constant-space validation context for consecutive bounded windows. Extent
/// chunks require their manifest first; checkpoint records require one complete
/// ordered header-to-footer sequence. Missing context is Unknown, never Intact.
#[derive(Default)]
pub struct PhysicalIntegrityScrubValidator {
    extent: Option<IntegrityValidatedExtentMembership>,
    checkpoint: Option<crate::artifact::checkpoint::CheckpointInspectionAggregate>,
}

impl PhysicalIntegrityScrubValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self, scope: PhysicalArtifactScope) {
        if scope.artifact_family() == Family::ExtentManifest {
            self.extent = None;
        }
        if scope.checkpoint_identity().is_some()
            || scope.artifact_family() == Family::CheckpointStreamHeader
        {
            self.checkpoint = None;
        }
    }

    pub fn inspect(
        &mut self,
        window: PhysicalIntegrityScrubWindow<'_>,
    ) -> (
        PhysicalIntegrityScrubInspection,
        PhysicalIntegrityObservationCounters,
    ) {
        let scope = window.scope();
        let input = window.artifact();
        match scope.artifact_family() {
            Family::ExtentManifest => {
                self.extent = None;
                let (result, counters) = validate_extent_manifest(input, scope);
                let outcome = match result {
                    ExtentManifestIntegrityValidation::Intact(manifest) => {
                        self.extent = Some(manifest.membership());
                        PhysicalIntegrityObservationOutcome::Intact(scope)
                    }
                    ExtentManifestIntegrityValidation::Rejected(rejection) => {
                        PhysicalIntegrityObservationOutcome::Rejected(rejection)
                    }
                };
                (PhysicalIntegrityScrubInspection::new(outcome), counters)
            }
            Family::ExtentChunk => {
                let Some(membership) = self.extent else {
                    return inspect_physical_integrity_window(window);
                };
                let coordinate = scope.extent_chunk_coordinate().expect("chunk scope");
                if scope.store_identity() != membership.scope().store_identity()
                    || coordinate.extent_cell() != membership.extent_cell()
                    || coordinate.record() != membership.record()
                {
                    return inspect_physical_integrity_window(window);
                }
                let (result, counters) = validate_extent_chunk_membership(input, scope, membership);
                let outcome = match result {
                    ExtentChunkIntegrityValidation::Intact(_) => {
                        PhysicalIntegrityObservationOutcome::Intact(scope)
                    }
                    ExtentChunkIntegrityValidation::Rejected(rejection) => {
                        PhysicalIntegrityObservationOutcome::Rejected(rejection)
                    }
                };
                (PhysicalIntegrityScrubInspection::new(outcome), counters)
            }
            Family::CheckpointStreamHeader
            | Family::CheckpointDirtyBasis
            | Family::CheckpointBindingCompaction
            | Family::CheckpointBinding
            | Family::CheckpointFooter => {
                crate::artifact::checkpoint::inspect_checkpoint_window(&mut self.checkpoint, window)
            }
            _ => inspect_physical_integrity_window(window),
        }
    }
}
