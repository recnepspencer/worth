use worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6;

use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::validation::{PhysicalArtifactScope, PhysicalIntegrityRejection};

use super::denial::{field_damage, FieldRange};

const STORE_IDENTITY: FieldRange = FieldRange::new(16, 16);
const RUNTIME_IDENTITY: FieldRange = FieldRange::new(32, 8);
const GENERATION_IDENTITY: FieldRange = FieldRange::new(40, 8);
const OPERATION_IDENTITY: FieldRange = FieldRange::new(48, 8);

pub(super) fn validate_expected_identity(
    scope: PhysicalArtifactScope,
    obligation: PhysicalWorkObligationV6,
) -> Result<(), PhysicalIntegrityRejection> {
    if obligation.store_identity() != scope.store_identity().bytes() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            STORE_IDENTITY,
            PhysicalFormatField::StoreIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    let expected = scope
        .physical_work_obligation_identity()
        .expect("physical-work family was checked before identity comparison");
    if obligation.identity().runtime() != expected.runtime() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            RUNTIME_IDENTITY,
            PhysicalFormatField::RuntimeIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if obligation.identity().generation() != expected.generation() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            GENERATION_IDENTITY,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if obligation.identity().operation() != expected.operation() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            OPERATION_IDENTITY,
            PhysicalFormatField::OperationIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    Ok(())
}
