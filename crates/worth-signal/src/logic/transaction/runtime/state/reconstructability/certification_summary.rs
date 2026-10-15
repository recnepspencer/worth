use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::data::error::SignalError;

use super::super::merge::canonical_digest;
use super::certification::{
    TemporalCertificationFailure, TemporalCertificationFamily, TemporalCertificationRecord,
    REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalCertificationSummary {
    pub required_family_count: u32,
    pub provided_record_count: u32,
    pub passed_family_count: u32,
    pub failed_family_count: u32,
    pub missing_family_count: u32,
    pub duplicate_family_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalCertificationBundle {
    pub schema_version: String,
    pub records: Vec<TemporalCertificationRecord>,
    pub summary: TemporalCertificationSummary,
    pub bundle_digest: String,
    pub passed: bool,
    pub failures: Vec<TemporalCertificationFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TemporalCertificationBundleMismatchClass {
    SchemaVersionMismatch,
    BundleDigestMismatch,
    PassStatusMismatch,
    SummaryMismatch,
    FailureSetMismatch,
    RecordSetMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporalCertificationBundleParityReport {
    pub proof_schema_version: String,
    pub expected: TemporalCertificationBundle,
    pub replayed: TemporalCertificationBundle,
    pub parity: bool,
    pub mismatch_classes: Vec<TemporalCertificationBundleMismatchClass>,
}

pub const TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION: &str =
    "worth-signal-temporal-certification-bundle-v1";
pub const TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION: &str =
    "worth-signal-temporal-certification-bundle-parity-v1";

pub fn temporal_certification_bundle(
    records: impl IntoIterator<Item = TemporalCertificationRecord>,
) -> TemporalCertificationBundle {
    let mut records = records.into_iter().collect::<Vec<_>>();
    records.sort_by_key(|record| record.family);
    let by_family = index_records(&records);
    let failures = collect_certification_failures(&records, &by_family);
    let summary = certification_summary(&records, &by_family, &failures);
    let digest = canonical_digest(&TemporalCertificationBundleDigestBasis {
        schema_version: TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION,
        records: &records,
    });
    let passed = failures.is_empty();
    TemporalCertificationBundle {
        schema_version: TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION.to_owned(),
        records,
        summary,
        bundle_digest: digest,
        passed,
        failures,
    }
}

fn index_records(
    records: &[TemporalCertificationRecord],
) -> BTreeMap<TemporalCertificationFamily, Vec<&TemporalCertificationRecord>> {
    let mut by_family: BTreeMap<TemporalCertificationFamily, Vec<&TemporalCertificationRecord>> =
        BTreeMap::new();
    for record in records {
        by_family.entry(record.family).or_default().push(record);
    }
    by_family
}

fn collect_certification_failures(
    records: &[TemporalCertificationRecord],
    by_family: &BTreeMap<TemporalCertificationFamily, Vec<&TemporalCertificationRecord>>,
) -> Vec<TemporalCertificationFailure> {
    let mut failures = Vec::new();
    for family in REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES {
        match by_family.get(&family) {
            None => failures.push(TemporalCertificationFailure::MissingRequiredFamily { family }),
            Some(records_for_family) if records_for_family.len() > 1 => {
                failures.push(TemporalCertificationFailure::DuplicateFamily {
                    family,
                    count: records_for_family.len() as u32,
                });
            }
            Some(_) => {}
        }
    }

    for record in records {
        if record.artifact.certification_digest.is_empty() {
            failures.push(TemporalCertificationFailure::EmptyCertificationDigest {
                family: record.family,
            });
        }
        if !record.passed {
            failures.push(TemporalCertificationFailure::FailedFamily {
                family: record.family,
            });
        }
        if let Some(parity) = record.parity.as_ref()
            && !parity.parity
        {
            failures.push(TemporalCertificationFailure::ParityMismatch {
                family: record.family,
                mismatch_classes: parity.mismatch_classes.clone(),
            });
        }
    }
    failures
}

fn certification_summary(
    records: &[TemporalCertificationRecord],
    by_family: &BTreeMap<TemporalCertificationFamily, Vec<&TemporalCertificationRecord>>,
    failures: &[TemporalCertificationFailure],
) -> TemporalCertificationSummary {
    let failed_families = failures
        .iter()
        .map(TemporalCertificationFailure::family)
        .collect::<BTreeSet<_>>();
    let passed_family_count = REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES
        .iter()
        .filter(|family| {
            by_family
                .get(family)
                .is_some_and(|records_for_family| records_for_family.len() == 1)
                && !failed_families.contains(family)
        })
        .count() as u32;
    let missing_family_count = REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES
        .iter()
        .filter(|family| !by_family.contains_key(family))
        .count() as u32;
    let duplicate_family_count = by_family
        .values()
        .filter(|records_for_family| records_for_family.len() > 1)
        .count() as u32;
    let failed_family_count = failed_families.len() as u32;
    TemporalCertificationSummary {
        required_family_count: REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES.len() as u32,
        provided_record_count: records.len() as u32,
        passed_family_count,
        failed_family_count,
        missing_family_count,
        duplicate_family_count,
    }
}

impl TemporalCertificationBundle {
    pub fn ensure_passed(&self) -> Result<(), SignalError> {
        if self.passed {
            return Ok(());
        }
        Err(SignalError::invalid_input(format!(
            "temporal certification bundle failed with {} failure(s)",
            self.failures.len()
        )))
    }
}

pub fn temporal_certification_bundle_parity_report(
    expected: &TemporalCertificationBundle,
    replayed: &TemporalCertificationBundle,
) -> TemporalCertificationBundleParityReport {
    let mut mismatch_classes = Vec::new();
    if expected.schema_version != replayed.schema_version {
        mismatch_classes.push(TemporalCertificationBundleMismatchClass::SchemaVersionMismatch);
    }
    if expected.bundle_digest != replayed.bundle_digest {
        mismatch_classes.push(TemporalCertificationBundleMismatchClass::BundleDigestMismatch);
    }
    if expected.passed != replayed.passed {
        mismatch_classes.push(TemporalCertificationBundleMismatchClass::PassStatusMismatch);
    }
    if expected.summary != replayed.summary {
        mismatch_classes.push(TemporalCertificationBundleMismatchClass::SummaryMismatch);
    }
    if expected.failures != replayed.failures {
        mismatch_classes.push(TemporalCertificationBundleMismatchClass::FailureSetMismatch);
    }
    if expected.records != replayed.records {
        mismatch_classes.push(TemporalCertificationBundleMismatchClass::RecordSetMismatch);
    }
    TemporalCertificationBundleParityReport {
        proof_schema_version: TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION.to_owned(),
        expected: expected.clone(),
        replayed: replayed.clone(),
        parity: mismatch_classes.is_empty(),
        mismatch_classes,
    }
}

#[derive(Debug, Serialize)]
struct TemporalCertificationBundleDigestBasis<'a> {
    schema_version: &'static str,
    records: &'a [TemporalCertificationRecord],
}
