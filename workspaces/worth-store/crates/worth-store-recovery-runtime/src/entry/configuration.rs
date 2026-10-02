use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryConfigurationDenial {
    ResidencyPolicyFormatMismatch {
        configured: PhysicalRecordFormatDeclaration,
        admitted: PhysicalRecordFormatDeclaration,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRecoveryStaticConfiguration {
    identity: [u8; 32],
    record_format: PhysicalRecordFormatDeclaration,
    residency_policy: AdmittedPhysicalRecordResidencyPolicy,
}

impl PhysicalRecoveryStaticConfiguration {
    pub fn current() -> Self {
        let record_format = PhysicalRecordFormatDeclaration::builder()
            .admit()
            .expect("the canonical physical record format is supported");
        Self::for_record_format(record_format)
    }

    pub fn for_record_format(record_format: PhysicalRecordFormatDeclaration) -> Self {
        let residency_policy = AdmittedPhysicalRecordResidencyPolicy::canonical(
            AdmittedPhysicalRecordFormat::admit(record_format),
        );
        Self::from_policy(record_format, residency_policy)
    }

    /// Bind a policy admitted for exactly this configured physical format.
    pub fn with_residency_policy(
        self,
        policy: AdmittedPhysicalRecordResidencyPolicy,
    ) -> Result<Self, PhysicalRecoveryConfigurationDenial> {
        if !policy.matches_format(AdmittedPhysicalRecordFormat::admit(self.record_format)) {
            return Err(
                PhysicalRecoveryConfigurationDenial::ResidencyPolicyFormatMismatch {
                    configured: self.record_format,
                    admitted: policy.record_format(),
                },
            );
        }
        Ok(Self::from_policy(self.record_format, policy))
    }

    fn from_policy(
        record_format: PhysicalRecordFormatDeclaration,
        residency_policy: AdmittedPhysicalRecordResidencyPolicy,
    ) -> Self {
        let mut digest = Sha256::new();
        digest.update(b"worth.store.physical.recovery.configuration@1");
        digest.update(record_format.canonical_identity_bytes());
        digest.update(residency_policy.canonical_identity_bytes());
        Self {
            identity: digest.finalize().into(),
            record_format,
            residency_policy,
        }
    }

    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub(crate) const fn record_format(&self) -> PhysicalRecordFormatDeclaration {
        self.record_format
    }

    pub(crate) const fn residency_policy(&self) -> AdmittedPhysicalRecordResidencyPolicy {
        self.residency_policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::PhysicalPageSizeClass;

    #[test]
    fn policy_geometry_mismatch_denies_at_configuration_construction() {
        let admitted = PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB16)
            .admit()
            .unwrap();
        let configured = PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap();
        let policy = AdmittedPhysicalRecordResidencyPolicy::canonical(
            AdmittedPhysicalRecordFormat::admit(admitted),
        );
        assert_eq!(
            PhysicalRecoveryStaticConfiguration::for_record_format(configured)
                .with_residency_policy(policy),
            Err(
                PhysicalRecoveryConfigurationDenial::ResidencyPolicyFormatMismatch {
                    configured,
                    admitted,
                }
            ),
        );
        let matching = AdmittedPhysicalRecordResidencyPolicy::canonical(
            AdmittedPhysicalRecordFormat::admit(configured),
        );
        let configuration = PhysicalRecoveryStaticConfiguration::for_record_format(configured)
            .with_residency_policy(matching)
            .expect("matching admitted geometry");
        assert_eq!(configuration.residency_policy(), matching);
    }
}
