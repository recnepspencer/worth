use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;

use crate::{
    PhysicalIntegrityObservationCounters, PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityRejection, PhysicalIntegrityScrubInspection, PhysicalIntegrityScrubWindow,
    UnknownPhysicalIntegrityCause, UnknownPhysicalIntegrityPosture,
};

/// Inspects exactly one immutable physical granule. The result is observation,
/// never an admission to an owner decoder or evidence of Store completeness.
pub fn inspect_physical_integrity_window(
    window: PhysicalIntegrityScrubWindow<'_>,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    let scope = window.scope();
    let input = window.artifact();
    macro_rules! inspect {
        ($validator:ident, $validation:ident) => {{
            let (result, counters) = crate::$validator(input, scope);
            let outcome = match result {
                crate::$validation::Intact(_) => PhysicalIntegrityObservationOutcome::Intact(scope),
                crate::$validation::Rejected(rejection) => {
                    PhysicalIntegrityObservationOutcome::Rejected(rejection)
                }
            };
            (PhysicalIntegrityScrubInspection::new(outcome), counters)
        }};
    }
    use PhysicalIntegrityArtifactFamily as Family;
    match scope.artifact_family() {
        Family::BootstrapCatalog => super::bootstrap_inspection::inspect(input, scope),
        Family::CurrentRootSelector => inspect!(
            validate_current_root_selector,
            CurrentRootSelectorIntegrityValidation
        ),
        Family::PreviousRootSelector => inspect!(
            validate_previous_root_selector,
            PreviousRootSelectorIntegrityValidation
        ),
        Family::RootManifest => inspect!(validate_root_manifest, RootManifestIntegrityValidation),
        Family::RootRoutingBlock => inspect!(
            validate_root_routing_block,
            RootRoutingBlockIntegrityValidation
        ),
        Family::SegmentMembership => inspect!(
            validate_segment_membership_block,
            SegmentMembershipBlockIntegrityValidation
        ),
        Family::PageFrame => inspect!(validate_inline_page, InlinePageIntegrityValidation),
        Family::ExtentManifest => {
            inspect!(validate_extent_manifest, ExtentManifestIntegrityValidation)
        }
        Family::FreeSpaceHeader => inspect!(
            validate_free_space_header,
            FreeSpaceHeaderIntegrityValidation
        ),
        Family::FreeSpaceMembershipBlock => inspect!(
            validate_free_space_membership_block,
            FreeSpaceMembershipBlockIntegrityValidation
        ),
        Family::PhysicalWorkObligation => inspect!(
            validate_physical_work_obligation,
            PhysicalWorkObligationIntegrityValidation
        ),
        Family::WalFrame => inspect!(validate_wal_frame, WalFrameIntegrityValidation),
        Family::CheckpointStreamHeader => inspect!(
            validate_checkpoint_stream_header,
            CheckpointStreamHeaderIntegrityValidation
        ),
        Family::CheckpointDirtyBasis => inspect!(
            validate_checkpoint_dirty_basis,
            CheckpointDirtyBasisIntegrityValidation
        ),
        Family::CheckpointBindingCompaction => inspect!(
            validate_checkpoint_binding_compaction,
            CheckpointBindingCompactionIntegrityValidation
        ),
        Family::CheckpointBinding => inspect!(
            validate_checkpoint_binding,
            CheckpointBindingIntegrityValidation
        ),
        // A footer envelope alone cannot establish its stream's selective aggregates.
        Family::ExtentChunk | Family::CheckpointFooter | Family::NamespaceIdentity => {
            let rejection =
                PhysicalIntegrityRejection::Unknown(UnknownPhysicalIntegrityPosture::new(
                    scope,
                    UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable,
                ));
            (
                PhysicalIntegrityScrubInspection::new(
                    PhysicalIntegrityObservationOutcome::Rejected(rejection),
                ),
                PhysicalIntegrityObservationCounters::one_rejected(
                    scope.artifact_family(),
                    input.byte_count(),
                    rejection,
                ),
            )
        }
    }
}
