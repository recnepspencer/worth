use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{durable_artifact_checksum, RootSelectorRole};

use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedPreviousRootSelector, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::selector_validation::validate_selector_envelope;

#[derive(Debug)]
pub enum PreviousRootSelectorIntegrityValidation<'media> {
    Intact(IntegrityValidatedPreviousRootSelector<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_previous_root_selector<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    PreviousRootSelectorIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let selector = match validate_selector_envelope(
        artifact,
        scope,
        PhysicalIntegrityArtifactFamily::PreviousRootSelector,
        RootSelectorRole::Previous,
    ) {
        Ok(selector) => selector,
        Err(rejection) => return rejected(rejection, byte_count),
    };
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated =
        IntegrityValidatedPreviousRootSelector::new(scope, selector, byte_range_checksum, artifact)
            .expect("validated previous selector satisfies the sealed-view contract");
    (
        PreviousRootSelectorIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::PreviousRootSelector,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    PreviousRootSelectorIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        PreviousRootSelectorIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::PreviousRootSelector,
            byte_count,
            rejection,
        ),
    )
}
