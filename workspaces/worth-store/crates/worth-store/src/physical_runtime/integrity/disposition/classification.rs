use worth_store_physical_integrity::{
    PhysicalIntegrityObservationOutcome, PhysicalIntegrityRejection,
    PhysicalIntegrityValidationRecord,
};

use super::authority::{
    DamagedPhysicalAuthorityObservation, IntactPhysicalAuthorityObservation,
    StoreAuthoritativeArtifactOwnerTruth,
};
use super::derived::{DamagedPhysicalDerivedDisposition, IntactPhysicalDerivedObservation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalArtifactRoleDisposition {
    IntactAuthority(IntactPhysicalAuthorityObservation),
    IntactDerived(IntactPhysicalDerivedObservation),
    DamagedAuthority(DamagedPhysicalAuthorityObservation),
    DamagedDerived(DamagedPhysicalDerivedDisposition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalArtifactDisposition {
    validator_outcome: PhysicalIntegrityObservationOutcome,
    owner_role: Option<PhysicalArtifactRoleDisposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerDispositionProjectionDenial {
    ScopeSubstitution,
    DamageRequiresOwnerTruth,
    NonDamageCannotReceiveOwnerRole,
}

pub(super) fn project_intact_authority(
    owner_truth: StoreAuthoritativeArtifactOwnerTruth,
    validation: PhysicalIntegrityValidationRecord,
) -> Result<PhysicalArtifactDisposition, OwnerDispositionProjectionDenial> {
    let scope = owner_truth.scope();
    if !validation.matches_scope(scope) {
        return Err(OwnerDispositionProjectionDenial::ScopeSubstitution);
    }
    Ok(PhysicalArtifactDisposition::with_owner_role(
        PhysicalIntegrityObservationOutcome::Intact(scope),
        PhysicalArtifactRoleDisposition::IntactAuthority(IntactPhysicalAuthorityObservation::new(
            scope,
        )),
    ))
}

pub(super) fn project_damaged_authority(
    owner_truth: StoreAuthoritativeArtifactOwnerTruth,
    rejection: PhysicalIntegrityRejection,
) -> Result<PhysicalArtifactDisposition, OwnerDispositionProjectionDenial> {
    let PhysicalIntegrityRejection::Damaged(localization) = rejection else {
        return Err(OwnerDispositionProjectionDenial::NonDamageCannotReceiveOwnerRole);
    };
    if localization.scope() != owner_truth.scope() {
        return Err(OwnerDispositionProjectionDenial::ScopeSubstitution);
    }
    Ok(PhysicalArtifactDisposition::with_owner_role(
        PhysicalIntegrityObservationOutcome::Rejected(rejection),
        PhysicalArtifactRoleDisposition::DamagedAuthority(
            DamagedPhysicalAuthorityObservation::new(localization),
        ),
    ))
}

pub(super) fn project_rejection_without_owner_truth(
    rejection: PhysicalIntegrityRejection,
) -> Result<PhysicalArtifactDisposition, OwnerDispositionProjectionDenial> {
    if matches!(rejection, PhysicalIntegrityRejection::Damaged(_)) {
        return Err(OwnerDispositionProjectionDenial::DamageRequiresOwnerTruth);
    }
    Ok(PhysicalArtifactDisposition {
        validator_outcome: PhysicalIntegrityObservationOutcome::Rejected(rejection),
        owner_role: None,
    })
}

impl PhysicalArtifactDisposition {
    const fn with_owner_role(
        validator_outcome: PhysicalIntegrityObservationOutcome,
        owner_role: PhysicalArtifactRoleDisposition,
    ) -> Self {
        Self {
            validator_outcome,
            owner_role: Some(owner_role),
        }
    }

    pub const fn validator_outcome(self) -> PhysicalIntegrityObservationOutcome {
        self.validator_outcome
    }

    pub const fn owner_role(self) -> Option<PhysicalArtifactRoleDisposition> {
        self.owner_role
    }
}
