mod denial;
mod identity;

use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::physical_work_obligation::decode_physical_work_obligation_v6;

use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedPhysicalWorkObligation, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use denial::{format_denial, input_length, invalid_scope_length, wrong_scope};
use identity::validate_expected_identity;

#[derive(Debug)]
pub enum PhysicalWorkObligationIntegrityValidation<'media> {
    Intact(IntegrityValidatedPhysicalWorkObligation<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_physical_work_obligation<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    PhysicalWorkObligationIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::PhysicalWorkObligation {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = invalid_scope_length(scope) {
        return rejected(rejection, byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let obligation = match decode_physical_work_obligation_v6(artifact.bytes()) {
        Ok(obligation) => obligation,
        Err(denial) => return rejected(format_denial(scope, artifact.bytes(), denial), byte_count),
    };
    if let Err(rejection) = validate_expected_identity(scope, obligation) {
        return rejected(rejection, byte_count);
    }
    let validated = IntegrityValidatedPhysicalWorkObligation::new(scope, obligation, artifact)
        .expect("validated physical-work obligation satisfies the sealed-view contract");
    (
        PhysicalWorkObligationIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    PhysicalWorkObligationIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        PhysicalWorkObligationIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::PhysicalWorkObligation,
            byte_count,
            rejection,
        ),
    )
}
