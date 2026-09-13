use worth_store_physical_format::{DurableRootSelector, RootSelectorRole};
use worth_store_physical_integrity::{
    IntegrityValidatedCurrentRootSelector, UntrustedPhysicalArtifact,
};

use super::super::super::RecoveryIntegrityIngressRejection;

pub(crate) struct IntegrityAdmittedStagedCurrentSelector<'bytes> {
    source: &'bytes [u8],
    validated: IntegrityValidatedCurrentRootSelector<'bytes>,
}

impl<'bytes> IntegrityAdmittedStagedCurrentSelector<'bytes> {
    pub(in crate::integrity_ingress) fn bind(
        source: &'bytes [u8],
        validated: IntegrityValidatedCurrentRootSelector<'bytes>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        let input = UntrustedPhysicalArtifact::from_bounded_bytes(source);
        if !validated.matches_input(input) {
            return Err(RecoveryIntegrityIngressRejection::SourceIncarnationMismatch);
        }
        let admitted = Self { source, validated };
        if admitted.source != admitted.selector().encode().as_slice() {
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
        .expect("sealed staged current-selector fields preserve the format contract")
    }

    pub(crate) fn project(self) -> DurableRootSelector {
        self.selector()
    }
}
