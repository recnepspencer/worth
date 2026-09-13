use worth_store_physical_format::{
    DurablePhysicalRootManifest, DurableRootSelector, PhysicalRecordFormatDeclaration,
};

use super::{
    PhysicalRootCandidateDenial, PhysicalRootManifestDenial, PhysicalRootSlotObservation,
    PhysicalRootSourceCandidate,
};

/// Describes an already interpreted selector/root pair to C.8 source
/// precedence. This is not integrity admission: the recovery runtime owns
/// source-bound validation before it projects these descriptive values.
pub fn observe_structured_physical_root_candidate(
    selector: DurableRootSelector,
    manifest: DurablePhysicalRootManifest,
    manifest_format: PhysicalRecordFormatDeclaration,
) -> PhysicalRootSlotObservation {
    match PhysicalRootSourceCandidate::from_structured_observation(
        selector,
        manifest,
        manifest_format,
    ) {
        Ok(candidate) => PhysicalRootSlotObservation::Candidate(candidate),
        Err(denial) => PhysicalRootSlotObservation::RootRejected {
            denial: match denial {
                PhysicalRootCandidateDenial::RootFormatMismatch => {
                    PhysicalRootManifestDenial::FormatMismatch
                }
                PhysicalRootCandidateDenial::RootGenerationMismatch => {
                    PhysicalRootManifestDenial::GenerationMismatch
                }
                _ => unreachable!("structured root observation only checks root bindings"),
            },
            selector,
        },
    }
}
