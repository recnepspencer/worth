use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};

use super::merge_descriptor::StrategyMergeDescriptor;
use super::replay_descriptor::StrategyReplayDescriptor;
use crate::commit_strategies::data::canonical_digest::{
    commit_validation_summary_digest, lowering_summary_digest, preview_validation_cost_digest,
};
use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact, CommitStrategyDescriptor,
    LoweredStrategyCommitPlan, StrategyCallerProvenance, StrategyLoweringProvenance,
    StrategyLoweringSummary, StrategyPreviewValidationCostSummary, StrategyRequestOrigin,
};
use crate::config::data::RelationalRuntimeConfig;
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::transactions::data::CommitValidationSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StrategyCommitArtifactBundle {
    pub(super) lowering_provenance: StrategyLoweringProvenance,
    pub(super) lowering_summary: StrategyLoweringSummary,
    pub(super) canonical_input: CanonicalStrategyInputArtifact,
    pub(super) merge_descriptor: StrategyMergeDescriptor,
    pub(super) replay_descriptor: StrategyReplayDescriptor,
    pub(super) preview_validation_summary: Option<CommitValidationSummary>,
    pub(super) preview_validation_cost: Option<StrategyPreviewValidationCostSummary>,
    pub(super) validated_against_commit_id: Option<CommitId>,
    pub(super) validated_against_version_id: Option<VersionId>,
}

impl StrategyCommitArtifactBundle {
    /// Consume only the pre-reserved original leaf grants before any public
    /// clone or exposure. A failed attachment restores its SAME unused grant.
    pub(crate) fn attach_readmission_memory(
        &mut self,
        input: &mut Option<worth_execution::ExecutionMemoryReservation>,
        targets: &mut Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), &'static str> {
        if let Err(unused) = self
            .canonical_input
            .retain_canonical_bytes_memory(input.take())
        {
            *input = unused;
            return Err("strategy input backing is not exclusively owned for readmission");
        }
        if let Err(unused) = self
            .merge_descriptor
            .retain_intent_scope_targets_memory(targets.take())
        {
            *targets = unused;
            return Err("strategy locator backing is not exclusively owned for readmission");
        }
        Ok(())
    }

    pub fn from_lowered(
        lowered: &LoweredStrategyCommitPlan,
        descriptor: &CommitStrategyDescriptor,
        runtime_config: &RelationalRuntimeConfig,
    ) -> Self {
        Self {
            lowering_provenance: lowered.lowering_provenance(),
            lowering_summary: lowered.lowering_summary().clone(),
            canonical_input: lowered.request().canonical_input().clone(),
            merge_descriptor: StrategyMergeDescriptor::from_descriptor_and_lowered(
                descriptor, lowered,
            ),
            replay_descriptor: StrategyReplayDescriptor::from_lowered(lowered, runtime_config),
            preview_validation_summary: None,
            preview_validation_cost: None,
            validated_against_commit_id: None,
            validated_against_version_id: None,
        }
    }

    pub fn with_preview_validation(
        mut self,
        preview_validation_summary: CommitValidationSummary,
        preview_validation_cost: StrategyPreviewValidationCostSummary,
        validated_against_commit_id: Option<CommitId>,
        validated_against_version_id: VersionId,
    ) -> Self {
        self.replay_descriptor = self.replay_descriptor.with_preview_validation(
            &preview_validation_summary,
            &preview_validation_cost,
            validated_against_commit_id,
            validated_against_version_id,
        );
        self.preview_validation_summary = Some(preview_validation_summary);
        self.preview_validation_cost = Some(preview_validation_cost);
        self.validated_against_commit_id = validated_against_commit_id;
        self.validated_against_version_id = Some(validated_against_version_id);
        self
    }

    pub fn lowering_provenance(&self) -> StrategyLoweringProvenance {
        self.lowering_provenance
    }

    pub fn lowering_summary(&self) -> &StrategyLoweringSummary {
        &self.lowering_summary
    }

    pub fn canonical_input(&self) -> &CanonicalStrategyInputArtifact {
        &self.canonical_input
    }

    pub fn merge_descriptor(&self) -> &StrategyMergeDescriptor {
        &self.merge_descriptor
    }

    pub fn replay_descriptor(&self) -> &StrategyReplayDescriptor {
        &self.replay_descriptor
    }

    pub fn preview_validation_summary(&self) -> Option<&CommitValidationSummary> {
        self.preview_validation_summary.as_ref()
    }

    pub fn preview_validation_cost(&self) -> Option<StrategyPreviewValidationCostSummary> {
        self.preview_validation_cost
    }

    pub fn validated_against_commit_id(&self) -> Option<CommitId> {
        self.validated_against_commit_id
    }

    pub fn validated_against_version_id(&self) -> Option<VersionId> {
        self.validated_against_version_id
    }

    pub fn replay_request(&self) -> CanonicalStrategyCommitRequest {
        CanonicalStrategyCommitRequest::new(
            self.replay_descriptor.strategy_id(),
            self.replay_descriptor.descriptor_digest(),
            self.canonical_input.clone(),
            StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Replay,
                actor_identity: None,
                correlation_id: None,
            },
        )
    }

    pub(super) fn validate_consistency(&self) -> Result<(), &'static str> {
        validate_replay_descriptor_matches_lowering(self)?;
        validate_canonical_input_matches_replay_descriptor(self)?;
        validate_merge_descriptor_matches_replay_descriptor(self)?;
        validate_preview_validation_matches_replay_descriptor(self)?;
        validate_validation_basis_matches_replay_descriptor(self)
    }
}

impl<'de> Deserialize<'de> for StrategyCommitArtifactBundle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStrategyCommitArtifactBundle {
            lowering_provenance: StrategyLoweringProvenance,
            lowering_summary: StrategyLoweringSummary,
            canonical_input: CanonicalStrategyInputArtifact,
            merge_descriptor: StrategyMergeDescriptor,
            replay_descriptor: StrategyReplayDescriptor,
            preview_validation_summary: Option<CommitValidationSummary>,
            preview_validation_cost: Option<StrategyPreviewValidationCostSummary>,
            validated_against_commit_id: Option<CommitId>,
            validated_against_version_id: Option<VersionId>,
        }

        let raw = RawStrategyCommitArtifactBundle::deserialize(deserializer)?;
        let bundle = Self {
            lowering_provenance: raw.lowering_provenance,
            lowering_summary: raw.lowering_summary,
            canonical_input: raw.canonical_input,
            merge_descriptor: raw.merge_descriptor,
            replay_descriptor: raw.replay_descriptor,
            preview_validation_summary: raw.preview_validation_summary,
            preview_validation_cost: raw.preview_validation_cost,
            validated_against_commit_id: raw.validated_against_commit_id,
            validated_against_version_id: raw.validated_against_version_id,
        };
        bundle.validate_consistency().map_err(D::Error::custom)?;
        Ok(bundle)
    }
}

fn validate_replay_descriptor_matches_lowering(
    bundle: &StrategyCommitArtifactBundle,
) -> Result<(), &'static str> {
    if bundle.lowering_provenance.strategy_id() != bundle.replay_descriptor.strategy_id() {
        return Err("strategy replay descriptor strategy id does not match lowering provenance");
    }
    if bundle.lowering_provenance.descriptor_digest()
        != bundle.replay_descriptor.descriptor_digest()
    {
        return Err(
            "strategy replay descriptor descriptor digest does not match lowering provenance",
        );
    }
    if bundle.lowering_provenance.input_digest() != bundle.replay_descriptor.input_digest() {
        return Err("strategy replay descriptor input digest does not match lowering provenance");
    }
    if bundle.lowering_provenance.output_digest() != bundle.replay_descriptor.output_digest() {
        return Err("strategy replay descriptor output digest does not match lowering provenance");
    }
    if bundle.lowering_provenance.mutation_program_digest()
        != bundle.replay_descriptor.mutation_program_digest()
    {
        return Err(
            "strategy replay descriptor mutation program digest does not match lowering provenance",
        );
    }
    Ok(())
}

fn validate_canonical_input_matches_replay_descriptor(
    bundle: &StrategyCommitArtifactBundle,
) -> Result<(), &'static str> {
    if bundle.canonical_input.digest() != bundle.replay_descriptor.input_digest() {
        return Err(
            "strategy canonical input artifact digest does not match strategy replay descriptor",
        );
    }
    if bundle.canonical_input.schema_name() != bundle.replay_descriptor.input_schema_name() {
        return Err(
            "strategy canonical input schema name does not match strategy replay descriptor",
        );
    }
    if bundle.canonical_input.schema_version() != bundle.replay_descriptor.input_schema_version() {
        return Err(
            "strategy canonical input schema version does not match strategy replay descriptor",
        );
    }
    if lowering_summary_digest(&bundle.lowering_summary)
        != *bundle.replay_descriptor.lowering_summary_digest()
    {
        return Err("strategy lowering summary does not match strategy replay descriptor digest");
    }
    Ok(())
}

fn validate_merge_descriptor_matches_replay_descriptor(
    bundle: &StrategyCommitArtifactBundle,
) -> Result<(), &'static str> {
    if bundle.merge_descriptor.strategy_id() != bundle.lowering_provenance.strategy_id() {
        return Err("strategy merge descriptor strategy id does not match lowering provenance");
    }
    if bundle.merge_descriptor.descriptor_digest() != bundle.lowering_provenance.descriptor_digest()
    {
        return Err(
            "strategy merge descriptor descriptor digest does not match lowering provenance",
        );
    }
    if bundle.merge_descriptor.lowering_summary_digest()
        != bundle.replay_descriptor.lowering_summary_digest()
    {
        return Err(
            "strategy merge descriptor lowering summary digest does not match strategy replay descriptor digest",
        );
    }
    Ok(())
}

fn validate_preview_validation_matches_replay_descriptor(
    bundle: &StrategyCommitArtifactBundle,
) -> Result<(), &'static str> {
    match (
        bundle.preview_validation_summary.as_ref(),
        bundle.replay_descriptor.preview_validation_summary_digest(),
    ) {
        (Some(summary), Some(expected_digest))
            if commit_validation_summary_digest(summary) == *expected_digest => {}
        (None, None) => {}
        _ => return Err(
            "strategy preview validation summary does not match strategy replay descriptor digest",
        ),
    }
    match (
        bundle.preview_validation_cost.as_ref(),
        bundle.replay_descriptor.preview_validation_cost_digest(),
    ) {
        (Some(summary), Some(expected_digest))
            if preview_validation_cost_digest(summary) == *expected_digest => {}
        (None, None) => {}
        _ => {
            return Err(
                "strategy preview validation cost does not match strategy replay descriptor digest",
            )
        }
    }
    Ok(())
}

fn validate_validation_basis_matches_replay_descriptor(
    bundle: &StrategyCommitArtifactBundle,
) -> Result<(), &'static str> {
    if bundle.validated_against_version_id
        != bundle.replay_descriptor.validated_against_version_id()
    {
        return Err(
            "strategy validated-against version id does not match strategy replay descriptor",
        );
    }
    if bundle.validated_against_commit_id != bundle.replay_descriptor.validated_against_commit_id()
    {
        return Err(
            "strategy validated-against commit id does not match strategy replay descriptor",
        );
    }
    Ok(())
}
