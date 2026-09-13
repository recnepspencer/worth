use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::IntegrityValidatedCheckpointStreamHeader;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{ObservedRecoverySource, RecoveryIntegrityIngressRejection};

pub(crate) struct IntegrityAdmittedCheckpointStreamHeader<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedCheckpointStreamHeader<'media>,
}

impl<'media> IntegrityAdmittedCheckpointStreamHeader<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCheckpointStreamHeader<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
    }

    pub(in crate::integrity_ingress) const fn source(&self) -> &ObservedRecoverySource<'media> {
        &self.source
    }

    pub(in crate::integrity_ingress) const fn checkpoint_identity(
        &self,
    ) -> PhysicalCheckpointIdentity {
        self.validated.checkpoint_identity()
    }

    pub(in crate::integrity_ingress) const fn validated(
        &self,
    ) -> &IntegrityValidatedCheckpointStreamHeader<'media> {
        &self.validated
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCheckpointStreamHeader<'media>,
    ) {
        let _ = IntegrityAdmittedCheckpointStreamHeader::bind(source, validated);
    }
    let _ = bind;
}
