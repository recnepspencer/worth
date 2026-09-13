use worth_store_physical_integrity::IntegrityValidatedCheckpointBindingCompaction;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{ObservedRecoverySource, RecoveryIntegrityIngressRejection};

pub(crate) struct IntegrityAdmittedCheckpointBindingCompaction<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedCheckpointBindingCompaction<'media>,
}

impl<'media> IntegrityAdmittedCheckpointBindingCompaction<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCheckpointBindingCompaction<'media>,
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

    pub(in crate::integrity_ingress) const fn validated(
        &self,
    ) -> &IntegrityValidatedCheckpointBindingCompaction<'media> {
        &self.validated
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCheckpointBindingCompaction<'media>,
    ) {
        let _ = IntegrityAdmittedCheckpointBindingCompaction::bind(source, validated);
    }
    let _ = bind;
}
