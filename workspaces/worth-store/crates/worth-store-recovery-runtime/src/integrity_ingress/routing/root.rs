use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{
    CurrentRootSelectorIntegrityValidation, PhysicalArtifactScope,
    PreviousRootSelectorIntegrityValidation, RootManifestIntegrityValidation,
    RootRoutingBlockIntegrityValidation,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::root::{
    IntegrityAdmittedCurrentRootSelector, IntegrityAdmittedPreviousRootSelector,
    IntegrityAdmittedRootManifest, IntegrityAdmittedRootRoutingBlock,
};
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

macro_rules! root_source_binding {
    ($name:ident, $validation:ident, $wrapper:ty, $variant:ident) => {
        pub(crate) fn $name(
            observed: &'media ObservedRecoveryArtifact,
            expected_scope: PhysicalArtifactScope,
            validation: $validation<'media>,
            counters: &mut RecoveryIntegrityIngressCounters,
        ) -> RecoveryIntegrityIngressAttempt<'media> {
            match validation {
                $validation::Intact(validated) => recorded(
                    expected_scope,
                    <$wrapper>::bind(
                        ObservedRecoverySource::complete(observed, expected_scope),
                        validated,
                    )
                    .map(Self::$variant),
                    counters,
                ),
                $validation::Rejected(rejection) => {
                    rejected_integrity(expected_scope, rejection, counters)
                }
            }
        }
    };
}

impl<'media> IntegrityAdmittedRecoveryArtifact<'media> {
    root_source_binding!(
        bind_current_selector,
        CurrentRootSelectorIntegrityValidation,
        IntegrityAdmittedCurrentRootSelector<'media>,
        CurrentSelector
    );
    root_source_binding!(
        bind_previous_selector,
        PreviousRootSelectorIntegrityValidation,
        IntegrityAdmittedPreviousRootSelector<'media>,
        PreviousSelector
    );
    root_source_binding!(
        bind_root_manifest,
        RootManifestIntegrityValidation,
        IntegrityAdmittedRootManifest<'media>,
        RootManifest
    );
    root_source_binding!(
        bind_root_routing_block,
        RootRoutingBlockIntegrityValidation,
        IntegrityAdmittedRootRoutingBlock<'media>,
        RootRoutingBlock
    );
}
