use super::super::readmission::{
    RecoveryLayoutReadmissionAdmissionDenial, RecoveryLayoutReadmissionClass,
    RecoveryLayoutReadmissionWitness,
};
use worth_store_authority::StoreCurrentAuthorityWitness;

use crate::LayoutCorruptionClassification;

use super::super::denial::CorruptionDenial;
use super::super::quarantine::{
    classify_quarantine_authority, LayoutQuarantineAuthorityClass, LayoutQuarantineWitness,
};
use super::super::readmission::{ImportReadmissionRequirement, QuarantineReadmissionRequirement};
use super::outcome::LayoutCorruptionOutcome;
use super::LayoutCorruptionAssessment;

impl LayoutCorruptionAssessment {
    pub fn assess_derived_projection(
        &self,
        classification: LayoutCorruptionClassification,
    ) -> LayoutCorruptionOutcome {
        LayoutCorruptionOutcome::rebuild_required(
            classification,
            super::LayoutCorruptionCounterSnapshot::rebuild_classification(),
        )
    }

    pub fn assess_quarantine_observation(
        &self,
        family: crate::AdmittedPhysicalArtifactFamily,
        observation_identity: super::super::readmission::RecoveryLayoutReadmissionIdentity,
        observation_class: RecoveryLayoutReadmissionClass,
    ) -> LayoutCorruptionOutcome {
        match classify_quarantine_authority(observation_class) {
            LayoutQuarantineAuthorityClass::DerivedProjectionDamage => self
                .assess_derived_projection(
                    LayoutCorruptionClassification::derived_projection_rebuild_to_parity(),
                ),
            LayoutQuarantineAuthorityClass::AuthoritativeQuarantineRequired => {
                LayoutCorruptionOutcome::quarantined(
                    LayoutQuarantineWitness::from_observation(
                        family,
                        observation_identity,
                        observation_class,
                    ),
                    super::LayoutCorruptionCounterSnapshot::quarantine_observation(),
                )
            }
        }
    }

    pub fn require_import_readmission(
        &self,
        target: crate::AdmittedPhysicalArtifactFamily,
        witness: RecoveryLayoutReadmissionWitness,
    ) -> LayoutCorruptionOutcome {
        LayoutCorruptionOutcome::import_readmission_required(
            ImportReadmissionRequirement::new(target, witness.identity().clone()),
            super::LayoutCorruptionCounterSnapshot::terminal_import(),
        )
    }

    pub fn require_observation_bound_recovery_readmission(
        &self,
        required: LayoutCorruptionOutcome,
        current_store_authority: &StoreCurrentAuthorityWitness,
        current_security_scope: &worth_store_security::StoreCurrentSecurityScopeWitnessSet,
    ) -> Result<LayoutCorruptionOutcome, CorruptionDenial> {
        match required.into_quarantined() {
            Ok((quarantine, counters)) => require_quarantine_readmission(
                quarantine,
                current_store_authority,
                current_security_scope,
                counters,
            ),
            Err(other) => Err(
                CorruptionDenial::ReadmissionInputDoesNotMatchRequiredOutcome {
                    required: other.class(),
                },
            ),
        }
    }
}

fn require_quarantine_readmission(
    quarantine: LayoutQuarantineWitness,
    current_store_authority: &StoreCurrentAuthorityWitness,
    current_security_scope: &worth_store_security::StoreCurrentSecurityScopeWitnessSet,
    counters: super::LayoutCorruptionCounterSnapshot,
) -> Result<LayoutCorruptionOutcome, CorruptionDenial> {
    let family = quarantine.family();
    let observation_identity = quarantine.observation_identity();
    let observation_class = quarantine.observation_class();
    let admitted_family = quarantine.admitted_family();
    let current_security_identity = current_security_scope.key_scope().identity();
    if admitted_family.security_identity() != current_security_identity {
        return Err(CorruptionDenial::SecurityScopeReadmissionMismatch {
            family,
            required: admitted_family.security_identity(),
            current: current_security_identity,
        });
    }
    match super::super::readmission::layout_readmission()
        .admit_quarantine(
            admitted_family.family_id(),
            observation_identity,
            observation_class,
            current_store_authority,
            current_security_scope,
        )
        .into_result()
    {
        Ok(witness) => match witness.class() {
            RecoveryLayoutReadmissionClass::QuarantineRecovery => {
                Ok(LayoutCorruptionOutcome::quarantine_readmission_required(
                    QuarantineReadmissionRequirement::new(quarantine, witness.identity().clone()),
                    counters.with_observation_bound_readmission(),
                ))
            }
            RecoveryLayoutReadmissionClass::NoForegroundAuthority => {
                Err(CorruptionDenial::NoForegroundReadAuthority { family })
            }
            class => Err(CorruptionDenial::UnexpectedQuarantineReadmissionClass { family, class }),
        },
        Err(RecoveryLayoutReadmissionAdmissionDenial::NoForegroundAuthority) => {
            Err(CorruptionDenial::NoForegroundReadAuthority { family })
        }
        Err(_) => {
            Err(CorruptionDenial::QuarantineObservationReadmissionEvidenceRequired { family })
        }
    }
}
