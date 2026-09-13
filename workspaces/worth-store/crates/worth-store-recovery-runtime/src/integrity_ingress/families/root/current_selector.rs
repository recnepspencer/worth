use worth_store_physical_format::{DurableRootSelector, RootSelectorRole};
use worth_store_physical_integrity::IntegrityValidatedCurrentRootSelector;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedCurrentRootSelector<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedCurrentRootSelector<'media>,
}

impl<'media> IntegrityAdmittedCurrentRootSelector<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCurrentRootSelector<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        let admitted = Self { source, validated };
        if admitted.source.input()?.bytes() != admitted.selector().encode().as_slice() {
            return Err(RecoveryIntegrityIngressRejection::NonCanonicalEncoding);
        }
        Ok(admitted)
    }

    fn selector(&self) -> DurableRootSelector {
        DurableRootSelector::new(
            self.validated.scope().store_identity(),
            self.validated.record_format(),
            self.validated.selector_identity(),
            RootSelectorRole::Current,
            self.validated.root_generation(),
            self.validated.linked_selector(),
            self.validated.linked_root_generation(),
        )
        .expect("sealed current-selector fields preserve the format contract")
    }

    pub(crate) fn project(self) -> DurableRootSelector {
        self.selector()
    }

    pub(crate) fn project_for_recovery(
        &self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> DurableRootSelector {
        counters.record_owner_projection();
        self.selector()
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
    }
}

pub(crate) fn admit_current_root_selector<'media>(
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedCurrentRootSelector<'media>,
) -> Result<IntegrityAdmittedCurrentRootSelector<'media>, RecoveryIntegrityIngressRejection> {
    IntegrityAdmittedCurrentRootSelector::bind(source, validated)
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedCurrentRootSelector<'media>,
    ) {
        let _ = admit_current_root_selector(source, validated);
    }
    let _ = bind;
}
