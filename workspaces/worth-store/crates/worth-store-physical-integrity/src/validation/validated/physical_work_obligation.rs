use worth_store_physical_format::physical_work_obligation::{
    physical_work_obligation_v6_incarnation_digest, PhysicalWorkObligationOperationCode,
    PhysicalWorkObligationTargetCode, PhysicalWorkObligationV6,
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
};
use worth_store_physical_format::PhysicalWorkObligationIdentity;

use super::super::{
    PhysicalArtifactScope, PhysicalIntegrityValidationDigest, PhysicalIntegrityValidationMechanism,
    PhysicalIntegrityValidationRecord, UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub struct IntegrityValidatedPhysicalWorkObligation<'media> {
    scope: PhysicalArtifactScope,
    obligation: PhysicalWorkObligationV6,
    validation_record: PhysicalIntegrityValidationRecord,
    inspected: UntrustedPhysicalArtifact<'media>,
}

impl<'media> IntegrityValidatedPhysicalWorkObligation<'media> {
    pub(crate) fn new(
        scope: PhysicalArtifactScope,
        obligation: PhysicalWorkObligationV6,
        inspected: UntrustedPhysicalArtifact<'media>,
    ) -> Option<Self> {
        if inspected.byte_count() != PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64
            || inspected.byte_count() != scope.byte_range().length()
            || obligation.store_identity() != scope.store_identity().bytes()
            || Some(obligation.identity()) != scope.physical_work_obligation_identity()
        {
            return None;
        }
        let validation_record = PhysicalIntegrityValidationRecord::from_validated_scope(
            scope,
            PhysicalIntegrityValidationDigest::sha256(scope.physical_work_exact_scope_digest()?),
            PhysicalIntegrityValidationDigest::sha256(
                physical_work_obligation_v6_incarnation_digest(
                    inspected
                        .bytes()
                        .try_into()
                        .expect("physical-work byte count was checked before fingerprinting"),
                ),
            ),
            PhysicalIntegrityValidationMechanism::Sha256V1,
        )?;
        Some(Self {
            scope,
            obligation,
            validation_record,
            inspected,
        })
    }

    pub const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub const fn identity(&self) -> PhysicalWorkObligationIdentity {
        self.obligation.identity()
    }

    pub const fn operation_code(&self) -> PhysicalWorkObligationOperationCode {
        self.obligation.operation_code()
    }

    pub const fn target(&self) -> PhysicalWorkObligationTargetCode {
        self.obligation.target()
    }

    pub const fn payload_digest(&self) -> Option<[u8; 32]> {
        self.obligation.payload_digest()
    }

    pub const fn into_validation_record(self) -> PhysicalIntegrityValidationRecord {
        self.validation_record
    }

    /// Matches the exact immutable slice incarnation inspected by validation.
    /// It exposes no bytes and grants no decoder authority.
    pub fn matches_input(&self, input: UntrustedPhysicalArtifact<'media>) -> bool {
        self.inspected.same_incarnation(input)
    }
}
