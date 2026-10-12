#[path = "closeout/closeout.rs"]
mod closeout;
#[path = "closeout/closeout_artifacts.rs"]
mod closeout_artifacts;
#[path = "closeout/closeout_audits.rs"]
mod closeout_audits;
#[path = "closeout/closeout_dx.rs"]
mod closeout_dx;
#[path = "closeout/closeout_meta.rs"]
mod closeout_meta;
#[path = "closeout/closeout_oracles.rs"]
mod closeout_oracles;
#[path = "closeout/closeout_postures.rs"]
mod closeout_postures;
#[path = "closeout/closeout_receipts.rs"]
mod closeout_receipts;
#[path = "closeout/closeout_slopes.rs"]
mod closeout_slopes;
#[path = "phase4/phase4.rs"]
mod phase4;
#[path = "seeded/scenarios.rs"]
mod scenarios;
#[path = "seeded/support.rs"]
mod support;

use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::EffectLifecycleCounters;
use crate::identity::hash_parts;

use super::taxonomy::EffectFamily;

pub use closeout::certify_effect_execution_pipeline;
pub use closeout_artifacts::{
    EffectExecutionCertificationBundle, EffectExecutionCertificationLane,
    EffectExecutionCertificationOutputDigest, EffectExecutionCertificationRow,
};
pub use phase4::{
    certify_effect_lifecycle_phase4, EffectLifecyclePhase4CertificationBundle,
    EffectLifecyclePhase4CertificationRow, EffectLifecyclePhase4LaneKind,
    EffectLifecyclePhase4LaneOutcome,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectLifecycleSeededOutcomeClass {
    ScalarExecuted,
    BatchExecuted,
    Lowered,
    Advisory,
    RebindRequired,
    Deferred,
    Denied,
}

impl EffectLifecycleSeededOutcomeClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ScalarExecuted => "scalar_executed",
            Self::BatchExecuted => "batch_executed",
            Self::Lowered => "lowered",
            Self::Advisory => "advisory",
            Self::RebindRequired => "rebind_required",
            Self::Deferred => "deferred",
            Self::Denied => "denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleSeededCertificationRow {
    scenario_name: String,
    outcome_class: EffectLifecycleSeededOutcomeClass,
    basis_family: BasisFamily,
    effect_family: EffectFamily,
    batch_width: usize,
    support_discovery_digest: String,
    normalized_effect_intent_digest: Option<String>,
    effect_eligibility_digest: String,
    authority_scoped_effect_plan_digest: Option<String>,
    lowered_effect_execution_plan_digest: Option<String>,
    effect_execution_receipt_digest: Option<String>,
    failure_digest: Option<String>,
    counter_snapshot_digest: String,
    counters: EffectLifecycleCounters,
    row_digest: String,
}

impl EffectLifecycleSeededCertificationRow {
    pub(super) fn new(parts: EffectLifecycleSeededRowParts) -> Self {
        let counter_snapshot_digest = parts.counters.counter_for_reporting().to_string();
        let row_digest = hash_parts(&[
            "effect_lifecycle_seeded_certification_row_v1".to_string(),
            format!("scenario:{}", parts.scenario_name),
            format!("outcome:{}", parts.outcome_class.as_str()),
            format!("basis:{}", parts.basis_family.as_str()),
            format!("family:{}", parts.effect_family.as_str()),
            format!("batch_width:{}", parts.batch_width),
            format!("support:{}", parts.support_discovery_digest),
            format!(
                "normalized:{}",
                parts
                    .normalized_effect_intent_digest
                    .as_deref()
                    .unwrap_or("none")
            ),
            format!("eligibility:{}", parts.effect_eligibility_digest),
            format!(
                "authority_scoped:{}",
                parts
                    .authority_scoped_effect_plan_digest
                    .as_deref()
                    .unwrap_or("none")
            ),
            format!(
                "lowered:{}",
                parts
                    .lowered_effect_execution_plan_digest
                    .as_deref()
                    .unwrap_or("none")
            ),
            format!(
                "execution:{}",
                parts
                    .effect_execution_receipt_digest
                    .as_deref()
                    .unwrap_or("none")
            ),
            format!(
                "failure:{}",
                parts.failure_digest.as_deref().unwrap_or("none")
            ),
            format!("counters:{counter_snapshot_digest}"),
        ]);
        Self {
            scenario_name: parts.scenario_name,
            outcome_class: parts.outcome_class,
            basis_family: parts.basis_family,
            effect_family: parts.effect_family,
            batch_width: parts.batch_width,
            support_discovery_digest: parts.support_discovery_digest,
            normalized_effect_intent_digest: parts.normalized_effect_intent_digest,
            effect_eligibility_digest: parts.effect_eligibility_digest,
            authority_scoped_effect_plan_digest: parts.authority_scoped_effect_plan_digest,
            lowered_effect_execution_plan_digest: parts.lowered_effect_execution_plan_digest,
            effect_execution_receipt_digest: parts.effect_execution_receipt_digest,
            failure_digest: parts.failure_digest,
            counter_snapshot_digest,
            counters: parts.counters,
            row_digest,
        }
    }

    pub fn scenario_name(&self) -> &str {
        &self.scenario_name
    }

    pub fn outcome_class(&self) -> EffectLifecycleSeededOutcomeClass {
        self.outcome_class
    }

    pub fn effect_family(&self) -> EffectFamily {
        self.effect_family
    }

    #[cfg(test)]
    pub fn batch_width(&self) -> usize {
        self.batch_width
    }

    pub fn failure_digest(&self) -> Option<&str> {
        self.failure_digest.as_deref()
    }

    #[cfg(test)]
    pub fn counter_snapshot_digest(&self) -> &str {
        &self.counter_snapshot_digest
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecycleSeededCertificationBundle {
    seed: u64,
    rows: Vec<EffectLifecycleSeededCertificationRow>,
    seeded_sequence_digest: String,
    seed_replay_digest: String,
    certification_bundle_digest: String,
    replay_is_deterministic: bool,
}

impl EffectLifecycleSeededCertificationBundle {
    fn new(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        seed: u64,
        rows: Vec<EffectLifecycleSeededCertificationRow>,
    ) -> Self {
        let seeded_sequence_digest = rows_digest(&rows);
        let replay_rows = scenarios::seeded_rows(execution, seed, rows.len());
        let replay_rows_digest = rows_digest(&replay_rows);
        let replay_is_deterministic = seeded_sequence_digest == replay_rows_digest;
        let seed_replay_digest = hash_parts(&[
            "effect_lifecycle_seed_replay_v1".to_string(),
            format!("seed:{seed}"),
            format!("primary:{seeded_sequence_digest}"),
            format!("replay:{replay_rows_digest}"),
            format!("deterministic:{replay_is_deterministic}"),
        ]);
        let certification_bundle_digest = hash_parts(&[
            "effect_lifecycle_seeded_certification_bundle_v1".to_string(),
            format!("seed:{seed}"),
            format!("rows:{seeded_sequence_digest}"),
            format!("replay:{seed_replay_digest}"),
        ]);
        Self {
            seed,
            rows,
            seeded_sequence_digest,
            seed_replay_digest,
            certification_bundle_digest,
            replay_is_deterministic,
        }
    }

    pub fn rows(&self) -> &[EffectLifecycleSeededCertificationRow] {
        &self.rows
    }

    pub fn seeded_sequence_digest(&self) -> &str {
        &self.seeded_sequence_digest
    }

    pub fn seed_replay_digest(&self) -> &str {
        &self.seed_replay_digest
    }

    pub fn certification_bundle_digest(&self) -> &str {
        &self.certification_bundle_digest
    }

    #[cfg(test)]
    pub fn replay_is_deterministic(&self) -> bool {
        self.replay_is_deterministic
    }
}

pub fn certify_effect_lifecycle_seeded(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    seed: u64,
    scenario_count: usize,
) -> EffectLifecycleSeededCertificationBundle {
    let rows = scenarios::seeded_rows(
        execution,
        seed,
        scenario_count.max(scenarios::minimum_seeded_scenario_count()),
    );
    EffectLifecycleSeededCertificationBundle::new(execution, seed, rows)
}

fn rows_digest(rows: &[EffectLifecycleSeededCertificationRow]) -> String {
    hash_parts(
        &rows
            .iter()
            .map(|row| row.row_digest().to_string())
            .collect::<Vec<_>>(),
    )
}

pub(super) struct EffectLifecycleSeededRowParts {
    pub(super) scenario_name: String,
    pub(super) outcome_class: EffectLifecycleSeededOutcomeClass,
    pub(super) basis_family: BasisFamily,
    pub(super) effect_family: EffectFamily,
    pub(super) batch_width: usize,
    pub(super) support_discovery_digest: String,
    pub(super) normalized_effect_intent_digest: Option<String>,
    pub(super) effect_eligibility_digest: String,
    pub(super) authority_scoped_effect_plan_digest: Option<String>,
    pub(super) lowered_effect_execution_plan_digest: Option<String>,
    pub(super) effect_execution_receipt_digest: Option<String>,
    pub(super) failure_digest: Option<String>,
    pub(super) counters: EffectLifecycleCounters,
}
