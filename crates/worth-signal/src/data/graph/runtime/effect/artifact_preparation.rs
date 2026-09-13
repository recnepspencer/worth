//! Materialized artifact storage carried into exclusive output publication.
use crate::data::error::SignalError;
use crate::data::trace::{ColdArtifactRecord, HotArtifactWrite, RuntimeArtifactState};
use crate::diagnostics::policy::ArtifactRetentionPolicy;
use crate::logic::evaluation::EvaluationWork;

use super::SignalGraph;

#[derive(Debug, Default)]
pub(super) struct PreparedEffectArtifactWrite {
    pub(super) runtime: Option<RuntimeArtifactState>,
    pub(super) retained: Option<ColdArtifactRecord>,
}

#[cfg(test)]
#[path = "artifact_preparation_tests.rs"]
mod tests;

#[derive(Debug, Default)]
enum PreparedArtifactWork {
    #[default]
    None,
    Materialized,
    Bypassed,
}

impl SignalGraph {
    /// Allocation precedes graph publication. The packet owns the materialized
    /// record; retention admission must still account for this draft storage.
    pub(super) fn prepare_effect_artifact_write(
        &mut self,
        artifact_write: Option<HotArtifactWrite>,
        allowance: &mut EvaluationWork<'_>,
    ) -> Result<PreparedEffectArtifactWrite, SignalError> {
        let Some(write) = artifact_write else {
            return Ok(PreparedEffectArtifactWrite::default());
        };
        let (retained, work) = if super::vocabulary::runtime_policy_omits_cold_artifacts(self) {
            (None, PreparedArtifactWork::Bypassed)
        } else if let Some(intent) = write.cold_intent {
            let policy = self.installed_runtime_policy().retention_budget();
            let retain =
                matches!(
                    policy.explanation_retention,
                    ArtifactRetentionPolicy::Retain
                ) || matches!(policy.provenance_retention, ArtifactRetentionPolicy::Retain);
            let retained = if retain {
                // Materialization moves all payload owners. Only inline labels
                // may need new Vec storage; their String payloads are not cloned.
                allowance.reserve(intent.labels.len().checked_add(8))?;
                intent.materialize_record()
            } else {
                None
            };
            let work = if retained.is_some() {
                PreparedArtifactWork::Materialized
            } else if !retain {
                PreparedArtifactWork::Bypassed
            } else {
                PreparedArtifactWork::None
            };
            (retained, work)
        } else {
            (None, PreparedArtifactWork::None)
        };
        self.record_prepared_artifact_work(work);
        Ok(PreparedEffectArtifactWrite {
            runtime: write.runtime,
            retained,
        })
    }

    /// These counters report performed preparation even if later admission
    /// rejects the packet. They confer no output publication authority.
    fn record_prepared_artifact_work(&mut self, work: PreparedArtifactWork) {
        let Some(mut telemetry) = self.telemetry_mut() else {
            return;
        };
        match work {
            PreparedArtifactWork::None => {}
            PreparedArtifactWork::Materialized => {
                telemetry
                    .storage
                    .hot_write_cold_record_materialization_count += 1;
                telemetry.storage.eager_cold_artifact_materialization_count += 1;
            }
            PreparedArtifactWork::Bypassed => {
                telemetry.storage.hot_write_cold_bypass_count += 1;
                telemetry.storage.deferred_cold_artifact_bypass_count += 1;
            }
        }
    }
}
