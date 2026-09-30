use serde::{Deserialize, Deserializer, Serialize};

use crate::commit_strategies::data::canonical_digest::{
    commit_validation_summary_digest, lowering_summary_digest, preview_validation_cost_digest,
    runtime_invariant_catalog_digest,
};
use crate::commit_strategies::data::{
    CanonicalStrategyInputDigest, CanonicalStrategyOutputDigest, CommitStrategyDescriptorDigest,
    CommitStrategyId, LoweredStrategyCommitPlan, StrategyInputSchemaName,
    StrategyInputSchemaVersion, StrategyMutationProgramDigest, StrategyOutputSchemaName,
    StrategyPreviewValidationCostSummary,
};
use crate::config::data::RelationalRuntimeConfig;
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::schema::data::{
    schema_authority_snapshot_digest_bytes, DescriptorCanonicalBasisVersion,
    DescriptorSemanticsVersion,
};
use crate::transactions::data::CommitValidationSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StrategyReplayDescriptor {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    input_digest: CanonicalStrategyInputDigest,
    output_digest: CanonicalStrategyOutputDigest,
    mutation_program_digest: StrategyMutationProgramDigest,
    input_schema_name: StrategyInputSchemaName,
    input_schema_version: StrategyInputSchemaVersion,
    output_schema_name: StrategyOutputSchemaName,
    lowering_summary_digest: [u8; 32],
    preview_validation_summary_digest: Option<[u8; 32]>,
    preview_validation_cost_digest: Option<[u8; 32]>,
    validated_against_commit_id: Option<CommitId>,
    validated_against_version_id: Option<VersionId>,
    runtime_determinism_basis: StrategyRuntimeDeterminismBasis,
}

impl StrategyReplayDescriptor {
    pub fn from_lowered(
        lowered: &LoweredStrategyCommitPlan,
        runtime_config: &RelationalRuntimeConfig,
    ) -> Self {
        Self {
            strategy_id: lowered.lowering_provenance().strategy_id(),
            descriptor_digest: lowered.lowering_provenance().descriptor_digest(),
            input_digest: lowered.lowering_provenance().input_digest(),
            output_digest: lowered.lowering_provenance().output_digest(),
            mutation_program_digest: lowered.lowering_provenance().mutation_program_digest(),
            input_schema_name: lowered.request().canonical_input().schema_name().clone(),
            input_schema_version: lowered.request().canonical_input().schema_version(),
            output_schema_name: lowered.execution().output().schema_name().clone(),
            lowering_summary_digest: lowering_summary_digest(lowered.lowering_summary()),
            preview_validation_summary_digest: None,
            preview_validation_cost_digest: None,
            validated_against_commit_id: None,
            validated_against_version_id: None,
            runtime_determinism_basis: StrategyRuntimeDeterminismBasis::from_runtime_config(
                runtime_config,
            ),
        }
    }

    pub(super) fn with_preview_validation(
        mut self,
        preview_validation_summary: &CommitValidationSummary,
        preview_validation_cost: &StrategyPreviewValidationCostSummary,
        validated_against_commit_id: Option<CommitId>,
        validated_against_version_id: VersionId,
    ) -> Self {
        self.preview_validation_summary_digest =
            Some(commit_validation_summary_digest(preview_validation_summary));
        self.preview_validation_cost_digest =
            Some(preview_validation_cost_digest(preview_validation_cost));
        self.validated_against_commit_id = validated_against_commit_id;
        self.validated_against_version_id = Some(validated_against_version_id);
        self
    }

    pub fn strategy_id(&self) -> CommitStrategyId {
        self.strategy_id
    }

    pub fn descriptor_digest(&self) -> CommitStrategyDescriptorDigest {
        self.descriptor_digest
    }

    pub fn input_digest(&self) -> CanonicalStrategyInputDigest {
        self.input_digest
    }

    pub fn output_digest(&self) -> CanonicalStrategyOutputDigest {
        self.output_digest
    }

    pub fn mutation_program_digest(&self) -> StrategyMutationProgramDigest {
        self.mutation_program_digest
    }

    pub fn input_schema_name(&self) -> &StrategyInputSchemaName {
        &self.input_schema_name
    }

    pub fn input_schema_version(&self) -> StrategyInputSchemaVersion {
        self.input_schema_version
    }

    pub fn output_schema_name(&self) -> &StrategyOutputSchemaName {
        &self.output_schema_name
    }

    pub fn lowering_summary_digest(&self) -> &[u8; 32] {
        &self.lowering_summary_digest
    }

    pub fn preview_validation_summary_digest(&self) -> Option<&[u8; 32]> {
        self.preview_validation_summary_digest.as_ref()
    }

    pub fn preview_validation_cost_digest(&self) -> Option<&[u8; 32]> {
        self.preview_validation_cost_digest.as_ref()
    }

    pub fn validated_against_commit_id(&self) -> Option<CommitId> {
        self.validated_against_commit_id
    }

    pub fn validated_against_version_id(&self) -> Option<VersionId> {
        self.validated_against_version_id
    }

    pub fn runtime_determinism_basis(&self) -> &StrategyRuntimeDeterminismBasis {
        &self.runtime_determinism_basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyRuntimeDeterminismBasis {
    #[serde(deserialize_with = "deserialize_basis_version")]
    basis_version: u8,
    schema_registry_digest: [u8; 32],
    invariant_catalog_digest: [u8; 32],
    descriptor_semantics_version: DescriptorSemanticsVersion,
    descriptor_canonical_basis_version: DescriptorCanonicalBasisVersion,
}

fn deserialize_basis_version<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let version = u8::deserialize(deserializer)?;
    if version == 2 {
        Ok(version)
    } else {
        Err(serde::de::Error::custom(
            "unsupported runtime determinism basis version",
        ))
    }
}

impl StrategyRuntimeDeterminismBasis {
    pub fn from_runtime_config(runtime_config: &RelationalRuntimeConfig) -> Self {
        let schema_authority = runtime_config.schema.registry.authority_snapshot();
        Self {
            basis_version: 2,
            schema_registry_digest: schema_authority_snapshot_digest_bytes(&schema_authority),
            invariant_catalog_digest: runtime_invariant_catalog_digest(
                &runtime_config.schema.invariant_catalog,
            ),
            descriptor_semantics_version: runtime_config
                .schema
                .descriptor_semantics_policy
                .current_write_version(),
            descriptor_canonical_basis_version: runtime_config
                .schema
                .descriptor_canonical_basis_policy
                .current_write_version(),
        }
    }

    pub fn schema_registry_digest(&self) -> &[u8; 32] {
        &self.schema_registry_digest
    }

    pub fn invariant_catalog_digest(&self) -> &[u8; 32] {
        &self.invariant_catalog_digest
    }

    pub fn descriptor_semantics_version(&self) -> DescriptorSemanticsVersion {
        self.descriptor_semantics_version
    }

    pub fn descriptor_canonical_basis_version(&self) -> DescriptorCanonicalBasisVersion {
        self.descriptor_canonical_basis_version
    }
}

impl<'de> Deserialize<'de> for StrategyReplayDescriptor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStrategyReplayDescriptor {
            strategy_id: CommitStrategyId,
            descriptor_digest: CommitStrategyDescriptorDigest,
            input_digest: CanonicalStrategyInputDigest,
            output_digest: CanonicalStrategyOutputDigest,
            mutation_program_digest: StrategyMutationProgramDigest,
            input_schema_name: StrategyInputSchemaName,
            input_schema_version: StrategyInputSchemaVersion,
            output_schema_name: StrategyOutputSchemaName,
            lowering_summary_digest: [u8; 32],
            preview_validation_summary_digest: Option<[u8; 32]>,
            preview_validation_cost_digest: Option<[u8; 32]>,
            validated_against_commit_id: Option<CommitId>,
            validated_against_version_id: Option<VersionId>,
            runtime_determinism_basis: StrategyRuntimeDeterminismBasis,
        }

        let raw = RawStrategyReplayDescriptor::deserialize(deserializer)?;
        Ok(Self {
            strategy_id: raw.strategy_id,
            descriptor_digest: raw.descriptor_digest,
            input_digest: raw.input_digest,
            output_digest: raw.output_digest,
            mutation_program_digest: raw.mutation_program_digest,
            input_schema_name: raw.input_schema_name,
            input_schema_version: raw.input_schema_version,
            output_schema_name: raw.output_schema_name,
            lowering_summary_digest: raw.lowering_summary_digest,
            preview_validation_summary_digest: raw.preview_validation_summary_digest,
            preview_validation_cost_digest: raw.preview_validation_cost_digest,
            validated_against_commit_id: raw.validated_against_commit_id,
            validated_against_version_id: raw.validated_against_version_id,
            runtime_determinism_basis: raw.runtime_determinism_basis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::StrategyRuntimeDeterminismBasis;
    use crate::config::data::RelationalRuntimeConfig;

    #[test]
    fn replay_basis_rejects_pre_lease_and_unknown_versions() {
        let basis = StrategyRuntimeDeterminismBasis::from_runtime_config(
            &RelationalRuntimeConfig::default(),
        );
        let mut value = serde_json::to_value(&basis).expect("serialize basis");
        assert_eq!(value["basis_version"], 2);
        assert_eq!(
            serde_json::from_value::<StrategyRuntimeDeterminismBasis>(value.clone())
                .expect("current basis"),
            basis
        );

        value
            .as_object_mut()
            .expect("basis object")
            .remove("basis_version");
        assert!(serde_json::from_value::<StrategyRuntimeDeterminismBasis>(value.clone()).is_err());
        value["execution_model_digest"] = serde_json::json!("retired");
        assert!(serde_json::from_value::<StrategyRuntimeDeterminismBasis>(value.clone()).is_err());

        value["basis_version"] = serde_json::json!(3);
        value
            .as_object_mut()
            .expect("basis object")
            .remove("execution_model_digest");
        assert!(serde_json::from_value::<StrategyRuntimeDeterminismBasis>(value).is_err());
    }
}
