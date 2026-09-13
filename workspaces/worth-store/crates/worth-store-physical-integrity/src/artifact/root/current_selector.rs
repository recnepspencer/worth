use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{durable_artifact_checksum, RootSelectorRole};

use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedCurrentRootSelector, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::selector_validation::validate_selector_envelope;

#[derive(Debug)]
pub enum CurrentRootSelectorIntegrityValidation<'media> {
    Intact(IntegrityValidatedCurrentRootSelector<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_current_root_selector<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    CurrentRootSelectorIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let selector = match validate_selector_envelope(
        artifact,
        scope,
        PhysicalIntegrityArtifactFamily::CurrentRootSelector,
        RootSelectorRole::Current,
    ) {
        Ok(selector) => selector,
        Err(rejection) => return rejected(rejection, byte_count),
    };
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated =
        IntegrityValidatedCurrentRootSelector::new(scope, selector, byte_range_checksum, artifact)
            .expect("validated current selector satisfies the sealed-view contract");
    (
        CurrentRootSelectorIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::CurrentRootSelector,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CurrentRootSelectorIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        CurrentRootSelectorIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::CurrentRootSelector,
            byte_count,
            rejection,
        ),
    )
}
