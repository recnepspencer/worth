use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{
    ExtentChunkIntegrityValidation, ExtentManifestIntegrityValidation, PhysicalArtifactScope,
};

use super::super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::super::families::extent::{
    IntegrityAdmittedExtentChunkFrame, IntegrityAdmittedExtentManifest,
};
use super::super::{ObservedRecoverySource, RecoveryIntegrityIngressCounters};
use super::{recorded, rejected_integrity, RecoveryIntegrityIngressAttempt};

macro_rules! extent_source_binding {
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
    extent_source_binding!(
        bind_extent_manifest,
        ExtentManifestIntegrityValidation,
        IntegrityAdmittedExtentManifest<'media>,
        ExtentManifest
    );
    extent_source_binding!(
        bind_extent_chunk,
        ExtentChunkIntegrityValidation,
        IntegrityAdmittedExtentChunkFrame<'media>,
        ExtentChunk
    );
}
