use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_program::{
        ApplicationArtifactResourceCeiling, ApplicationCompositionInstance,
        ApplicationDerivedArtifact, ApplicationDerivedArtifactDeclaration, ApplicationFeature,
        ApplicationProgramDefinition,
    },
    application_schema::ApplicationSchema,
};

use super::WorthQueryInstalledApplicationProgram;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramArtifactPosture {
    Legacy,
    Installed(ApplicationDerivedArtifactDeclaration),
    Undeclared,
}

/// Installed evidence for one exact feature occurrence and derived artifact.
pub struct WorthQueryInstalledDerivedArtifact<Schema, Instance, Feature, Artifact> {
    ceiling: ApplicationArtifactResourceCeiling,
    marker: PhantomData<fn() -> (Schema, Instance, Feature, Artifact)>,
}

impl<Schema, Instance, Feature, Artifact>
    WorthQueryInstalledDerivedArtifact<Schema, Instance, Feature, Artifact>
{
    pub const fn resource_ceiling(&self) -> ApplicationArtifactResourceCeiling {
        self.ceiling
    }
}

impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn derived_artifact<Instance, Feature, Artifact>(
        &self,
    ) -> Option<WorthQueryInstalledDerivedArtifact<Schema, Instance, Feature, Artifact>>
    where
        Instance: ApplicationCompositionInstance,
        Feature: ApplicationFeature<Schema>,
        Artifact: ApplicationDerivedArtifact<Schema, Feature>,
    {
        let artifact = std::any::TypeId::of::<Artifact>();
        self.features
            .iter()
            .find(|feature| {
                feature.composition_instance() == Instance::PATH
                    && feature.identity() == Feature::IDENTITY
            })?
            .derived_artifacts()
            .iter()
            .find(|declared| declared.artifact_type() == artifact)
            .map(|declared| WorthQueryInstalledDerivedArtifact {
                ceiling: declared.resource_ceiling(),
                marker: PhantomData,
            })
    }

    #[doc(hidden)]
    pub fn artifact_posture(
        &self,
        instance: &str,
        feature_type: std::any::TypeId,
        producer_family: &str,
    ) -> WorthQueryProgramArtifactPosture {
        let Some(feature) = self.features.iter().find(|feature| {
            feature.composition_instance() == instance && feature.feature_type() == feature_type
        }) else {
            return match Program::DERIVED_ARTIFACT_GOVERNANCE {
                worth_query_declaration::facade::application_program::ApplicationDerivedArtifactGovernance::Compatible => WorthQueryProgramArtifactPosture::Legacy,
                worth_query_declaration::facade::application_program::ApplicationDerivedArtifactGovernance::Required => WorthQueryProgramArtifactPosture::Undeclared,
            };
        };
        if feature.derived_artifacts().is_empty() {
            return match Program::DERIVED_ARTIFACT_GOVERNANCE {
                worth_query_declaration::facade::application_program::ApplicationDerivedArtifactGovernance::Compatible => WorthQueryProgramArtifactPosture::Legacy,
                worth_query_declaration::facade::application_program::ApplicationDerivedArtifactGovernance::Required => WorthQueryProgramArtifactPosture::Undeclared,
            };
        }
        feature
            .derived_artifacts()
            .iter()
            .find(|artifact| artifact.producer_family() == producer_family)
            .copied()
            .map(WorthQueryProgramArtifactPosture::Installed)
            .unwrap_or(WorthQueryProgramArtifactPosture::Undeclared)
    }
}
