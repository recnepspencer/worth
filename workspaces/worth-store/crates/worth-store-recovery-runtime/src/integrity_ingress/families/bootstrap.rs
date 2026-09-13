use worth_store_physical_format::{CurrentRootCatalogGeneration, PhysicalRecordFormatDeclaration};
use worth_store_physical_integrity::IntegrityValidatedBootstrapCatalog;

use super::super::admission::require_observed_recovery_source;
use super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedBootstrapCatalog<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedBootstrapCatalog<'media>,
}

pub(crate) struct BootstrapCatalogProjection {
    pub record_format: PhysicalRecordFormatDeclaration,
    pub current_root_generation: CurrentRootCatalogGeneration,
}

impl<'media> IntegrityAdmittedBootstrapCatalog<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedBootstrapCatalog<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project(
        &self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> BootstrapCatalogProjection {
        counters.record_owner_projection();
        BootstrapCatalogProjection {
            record_format: self.validated.record_format(),
            current_root_generation: self.validated.current_root_generation(),
        }
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedBootstrapCatalog<'media>,
    ) {
        let _ = IntegrityAdmittedBootstrapCatalog::bind(source, validated);
    }
    let _ = bind;
}
