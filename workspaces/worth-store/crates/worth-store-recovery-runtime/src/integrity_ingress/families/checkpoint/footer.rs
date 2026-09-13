use worth_store_physical_integrity::IntegrityValidatedCheckpointFooter;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{ObservedRecoverySource, RecoveryIntegrityIngressRejection};

pub(crate) struct IntegrityAdmittedCheckpointFooter<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedCheckpointFooter<'media>,
}

impl<'media> IntegrityAdmittedCheckpointFooter<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCheckpointFooter<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
    }

    pub(in crate::integrity_ingress) const fn validated(
        &self,
    ) -> &IntegrityValidatedCheckpointFooter<'media> {
        &self.validated
    }

    pub(in crate::integrity_ingress) const fn source(&self) -> &ObservedRecoverySource<'media> {
        &self.source
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCheckpointFooter<'media>,
    ) {
        let _ = IntegrityAdmittedCheckpointFooter::bind(source, validated);
    }
    let _ = bind;
}
