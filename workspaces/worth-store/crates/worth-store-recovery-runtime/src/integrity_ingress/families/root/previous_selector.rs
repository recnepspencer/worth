use worth_store_physical_format::{DurableRootSelector, RootSelectorRole};
use worth_store_physical_integrity::IntegrityValidatedPreviousRootSelector;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedPreviousRootSelector<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedPreviousRootSelector<'media>,
}

impl<'media> IntegrityAdmittedPreviousRootSelector<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedPreviousRootSelector<'media>,
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
            RootSelectorRole::Previous,
            self.validated.root_generation(),
            self.validated.linked_selector(),
            self.validated.linked_root_generation(),
        )
        .expect("sealed previous-selector fields preserve the format contract")
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

pub(crate) fn admit_previous_root_selector<'media>(
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedPreviousRootSelector<'media>,
) -> Result<IntegrityAdmittedPreviousRootSelector<'media>, RecoveryIntegrityIngressRejection> {
    IntegrityAdmittedPreviousRootSelector::bind(source, validated)
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedPreviousRootSelector<'media>,
    ) {
        let _ = admit_previous_root_selector(source, validated);
    }
    let _ = bind;
}
