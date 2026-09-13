use std::num::NonZeroU64;

use worth_store_physical_format::{
    physical_work_obligation::{
        PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
        PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
    },
    store_namespace::StableStoreIdentity,
    PhysicalWorkObligationIdentity,
};
use worth_store_physical_integrity::{
    validate_physical_work_obligation, IntegrityValidatedPhysicalWorkObligation,
    PhysicalArtifactScope, PhysicalByteRange, PhysicalWorkObligationIntegrityValidation,
    UntrustedPhysicalArtifact,
};

use super::{
    observation::{PhysicalWorkRecoveryAdmissionCounters, PhysicalWorkRecoveryIngressRejection},
    PhysicalWorkRecoveryLocator,
};

pub(super) struct IntegrityAdmittedPhysicalWorkProjection {
    scope: PhysicalArtifactScope,
    identity: PhysicalWorkObligationIdentity,
    operation: PhysicalWorkObligationOperationCode,
    target: PhysicalWorkObligationTargetCode,
    payload_digest: Option<[u8; 32]>,
}

pub(super) fn scope_from_pending_name(
    store: StableStoreIdentity,
    file_name: &str,
) -> Result<PhysicalArtifactScope, PhysicalWorkRecoveryIngressRejection> {
    let identity = identity_from_pending_name(file_name)
        .ok_or(PhysicalWorkRecoveryIngressRejection::InvalidPendingName)?;
    let range = PhysicalByteRange::new(0, PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64)
        .expect("physical-work v6 has a fixed nonzero record length");
    Ok(PhysicalArtifactScope::physical_work_obligation(
        store, identity, range,
    ))
}

pub(super) fn admit_bounded_obligation(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    counters: &mut PhysicalWorkRecoveryAdmissionCounters,
) -> Result<PhysicalWorkRecoveryLocator, PhysicalWorkRecoveryIngressRejection> {
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(bytes);
    let (validation, _) = validate_physical_work_obligation(input, scope);
    let validated = match validation {
        PhysicalWorkObligationIntegrityValidation::Intact(validated) => validated,
        PhysicalWorkObligationIntegrityValidation::Rejected(rejection) => {
            counters.rejected_before_owner_interpretation();
            return Err(PhysicalWorkRecoveryIngressRejection::Integrity(rejection));
        }
    };
    project_validated_obligation(input, validated, counters)
}

fn project_validated_obligation<'media>(
    input: UntrustedPhysicalArtifact<'media>,
    validated: IntegrityValidatedPhysicalWorkObligation<'media>,
    counters: &mut PhysicalWorkRecoveryAdmissionCounters,
) -> Result<PhysicalWorkRecoveryLocator, PhysicalWorkRecoveryIngressRejection> {
    let projection = bind_validated_projection(input, validated).map_err(|rejection| {
        counters.rejected_before_owner_interpretation();
        rejection
    })?;
    counters.owner_interpretation();
    let locator = PhysicalWorkRecoveryLocator::from_integrity_admitted(projection)
        .ok_or(PhysicalWorkRecoveryIngressRejection::OwnerProjectionRejected)?;
    counters.admitted();
    Ok(locator)
}

fn bind_validated_projection<'media>(
    input: UntrustedPhysicalArtifact<'media>,
    validated: IntegrityValidatedPhysicalWorkObligation<'media>,
) -> Result<IntegrityAdmittedPhysicalWorkProjection, PhysicalWorkRecoveryIngressRejection> {
    if !validated.matches_input(input) {
        return Err(PhysicalWorkRecoveryIngressRejection::SourceIncarnationMismatch);
    }
    Ok(IntegrityAdmittedPhysicalWorkProjection {
        scope: validated.scope(),
        identity: validated.identity(),
        operation: validated.operation_code(),
        target: validated.target(),
        payload_digest: validated.payload_digest(),
    })
}

impl IntegrityAdmittedPhysicalWorkProjection {
    pub(super) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub(super) const fn identity(&self) -> PhysicalWorkObligationIdentity {
        self.identity
    }

    pub(super) const fn operation(&self) -> PhysicalWorkObligationOperationCode {
        self.operation
    }

    pub(super) const fn target(&self) -> PhysicalWorkObligationTargetCode {
        self.target
    }

    pub(super) const fn payload_digest(&self) -> Option<[u8; 32]> {
        self.payload_digest
    }
}

fn identity_from_pending_name(file_name: &str) -> Option<PhysicalWorkObligationIdentity> {
    let body = file_name
        .strip_prefix("effect-")?
        .strip_suffix(".pending")?;
    let mut parts = body.split('-');
    let runtime = parse_identity_part(parts.next()?)?;
    let generation = parse_identity_part(parts.next()?)?;
    let operation = parse_identity_part(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some(PhysicalWorkObligationIdentity::new(
        runtime, generation, operation,
    ))
}

fn parse_identity_part(value: &str) -> Option<NonZeroU64> {
    if value.len() != 16
        || !value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return None;
    }
    NonZeroU64::new(u64::from_str_radix(value, 16).ok()?)
}

#[cfg(test)]
mod tests;
