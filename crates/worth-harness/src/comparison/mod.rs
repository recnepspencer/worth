use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::capture::{RunRecord, SnapshotRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComparisonMode {
    Exact,
    Semantic,
    Tolerance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonProfile {
    pub mode: ComparisonMode,
    pub include_extensions: bool,
    pub numeric_tolerance: Option<NumericTolerance>,
}

impl Default for ComparisonProfile {
    fn default() -> Self {
        Self {
            mode: ComparisonMode::Exact,
            include_extensions: true,
            numeric_tolerance: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NumericTolerance {
    pub absolute: f64,
    pub relative: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleComparisonOutcome {
    pub matched: bool,
    pub detail: String,
    pub fields: BTreeMap<String, Value>,
}

pub trait ComparisonOracle<Record> {
    fn compare_with_oracle(
        &self,
        left: &Record,
        right: &Record,
        profile: &ComparisonProfile,
    ) -> Result<Option<OracleComparisonOutcome>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct ComparisonOracleSuite<Record> {
    oracles: Vec<Box<dyn ComparisonOracle<Record>>>,
}

impl<Record> Default for ComparisonOracleSuite<Record> {
    fn default() -> Self {
        Self {
            oracles: Vec::new(),
        }
    }
}

impl<Record> ComparisonOracleSuite<Record> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_oracle(mut self, oracle: impl ComparisonOracle<Record> + 'static) -> Self {
        self.oracles.push(Box::new(oracle));
        self
    }

    pub fn evaluate(
        &self,
        left: &Record,
        right: &Record,
        profile: &ComparisonProfile,
    ) -> Result<Vec<OracleComparisonOutcome>, Box<dyn std::error::Error + Send + Sync>> {
        let mut outcomes = Vec::new();
        for oracle in &self.oracles {
            if let Some(outcome) = oracle.compare_with_oracle(left, right, profile)? {
                outcomes.push(outcome);
            }
        }
        Ok(outcomes)
    }
}

pub fn numbers_within_tolerance(left: f64, right: f64, tolerance: NumericTolerance) -> bool {
    let absolute_delta = (left - right).abs();
    if absolute_delta <= tolerance.absolute {
        return true;
    }
    if let Some(relative) = tolerance.relative {
        let baseline = right.abs().max(1.0);
        return absolute_delta / baseline <= relative;
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonMismatch {
    pub path: String,
    pub detail: String,
    pub severity: ComparisonSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonRecord {
    pub matched: bool,
    pub mismatches: Vec<ComparisonMismatch>,
}

impl ComparisonRecord {
    pub fn pass() -> Self {
        Self {
            matched: true,
            mismatches: Vec::new(),
        }
    }
}

fn mismatch(path: impl Into<String>, detail: impl Into<String>) -> ComparisonMismatch {
    ComparisonMismatch {
        path: path.into(),
        detail: detail.into(),
        severity: ComparisonSeverity::Error,
    }
}

pub fn compare_run_records<TargetId>(
    left: &RunRecord<TargetId>,
    right: &RunRecord<TargetId>,
    profile: &ComparisonProfile,
) -> ComparisonRecord
where
    TargetId: std::fmt::Debug + PartialEq,
{
    let mut mismatches = Vec::new();
    if left.status != right.status {
        mismatches.push(mismatch(
            "run.status",
            format!("left={:?} right={:?}", left.status, right.status),
        ));
    }
    if left.outcome != right.outcome {
        mismatches.push(mismatch(
            "run.outcome",
            format!("left={:?} right={:?}", left.outcome, right.outcome),
        ));
    }
    if left.budget_usage != right.budget_usage {
        mismatches.push(mismatch(
            "run.budget_usage",
            format!(
                "left={:?} right={:?}",
                left.budget_usage, right.budget_usage
            ),
        ));
    }
    if left.requested_targets != right.requested_targets {
        mismatches.push(mismatch(
            "run.requested_targets",
            format!(
                "left={:?} right={:?}",
                left.requested_targets, right.requested_targets
            ),
        ));
    }
    if left.target_statuses != right.target_statuses {
        mismatches.push(mismatch(
            "run.target_statuses",
            format!(
                "left={:?} right={:?}",
                left.target_statuses, right.target_statuses
            ),
        ));
    }
    if left.changed_targets != right.changed_targets {
        mismatches.push(mismatch(
            "run.changed_targets",
            format!(
                "left={:?} right={:?}",
                left.changed_targets, right.changed_targets
            ),
        ));
    }
    if left.summary != right.summary {
        mismatches.push(mismatch(
            "run.summary",
            format!("left={} right={}", left.summary, right.summary),
        ));
    }
    if left.time_marker != right.time_marker {
        mismatches.push(mismatch(
            "run.time_marker",
            format!("left={:?} right={:?}", left.time_marker, right.time_marker),
        ));
    }
    if left.feed_batch != right.feed_batch {
        mismatches.push(mismatch(
            "run.feed_batch",
            format!("left={:?} right={:?}", left.feed_batch, right.feed_batch),
        ));
    }
    if left.attachments != right.attachments {
        mismatches.push(mismatch(
            "run.attachments",
            format!("left={:?} right={:?}", left.attachments, right.attachments),
        ));
    }
    if profile.include_extensions && left.extensions != right.extensions {
        mismatches.push(mismatch(
            "run.extensions",
            format!("left={:?} right={:?}", left.extensions, right.extensions),
        ));
    }
    ComparisonRecord {
        matched: mismatches.is_empty(),
        mismatches,
    }
}

pub fn compare_snapshot_records<TargetId>(
    left: &SnapshotRecord<TargetId>,
    right: &SnapshotRecord<TargetId>,
    profile: &ComparisonProfile,
) -> ComparisonRecord
where
    TargetId: std::fmt::Debug + PartialEq,
{
    let mut mismatches = Vec::new();
    if left.observations != right.observations {
        mismatches.push(mismatch(
            "snapshot.observations",
            format!(
                "left={:?} right={:?}",
                left.observations, right.observations
            ),
        ));
    }
    if left.time_marker != right.time_marker {
        mismatches.push(mismatch(
            "snapshot.time_marker",
            format!("left={:?} right={:?}", left.time_marker, right.time_marker),
        ));
    }
    if left.attachments != right.attachments {
        mismatches.push(mismatch(
            "snapshot.attachments",
            format!("left={:?} right={:?}", left.attachments, right.attachments),
        ));
    }
    if profile.include_extensions && left.extensions != right.extensions {
        mismatches.push(mismatch(
            "snapshot.extensions",
            format!("left={:?} right={:?}", left.extensions, right.extensions),
        ));
    }
    ComparisonRecord {
        matched: mismatches.is_empty(),
        mismatches,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{
        numbers_within_tolerance, ComparisonMode, ComparisonOracle, ComparisonOracleSuite,
        ComparisonProfile, NumericTolerance, OracleComparisonOutcome,
    };

    struct EqualityOracle;

    impl ComparisonOracle<serde_json::Value> for EqualityOracle {
        fn compare_with_oracle(
            &self,
            left: &serde_json::Value,
            right: &serde_json::Value,
            _profile: &ComparisonProfile,
        ) -> Result<Option<OracleComparisonOutcome>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(OracleComparisonOutcome {
                matched: left == right,
                detail: "oracle equality".to_string(),
                fields: BTreeMap::new(),
            }))
        }
    }

    #[test]
    fn numeric_tolerance_supports_domain_comparison() {
        assert!(numbers_within_tolerance(
            10.0,
            10.005,
            NumericTolerance {
                absolute: 0.01,
                relative: Some(0.001),
            },
        ));
    }

    #[test]
    fn comparison_oracle_suite_collects_oracle_results() {
        let profile = ComparisonProfile {
            mode: ComparisonMode::Semantic,
            include_extensions: true,
            numeric_tolerance: None,
        };
        let outcomes = ComparisonOracleSuite::new()
            .with_oracle(EqualityOracle)
            .evaluate(&json!({"a": 1}), &json!({"a": 1}), &profile)
            .unwrap();
        assert_eq!(outcomes.len(), 1);
        assert!(outcomes[0].matched);
    }
}
