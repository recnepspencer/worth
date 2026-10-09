use std::collections::BTreeSet;
use worth_foundational::{
    PhysicalArtifactFamily, PhysicalArtifactGeneration, PhysicalArtifactIdentity, PhysicalByteRange,
};

use super::{
    OfflineIntegrityObservationCounters, OfflineIntegrityObservationLimits,
    OfflineIntegrityOutcome, OfflineIntegrityProtocolContext, OFFLINE_OBSERVER_ROLE_IDENTITY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityReportCompleteness {
    Complete,
    BoundExhausted,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfflineArtifactFamily {
    Declared(PhysicalArtifactFamily),
    OriginalDropReservation,
    DedupeQuarantine,
    Unrecognized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineBlobReclaimSourceKind {
    FailedIngest,
    ReleasedGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfflineArtifactDuplicateEvidence {
    PhysicalAlias { first_path: Box<str> },
    SemanticIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineArtifactObservation {
    relative_path: Box<str>,
    family: OfflineArtifactFamily,
    identity: PhysicalArtifactIdentity,
    generation: PhysicalArtifactGeneration,
    range: Option<PhysicalByteRange>,
    outcome: OfflineIntegrityOutcome,
    expected_point_page_touches: Option<u64>,
    index_family: Option<&'static str>,
    blob_reclaim_source_kind: Option<OfflineBlobReclaimSourceKind>,
    duplicates: Vec<OfflineArtifactDuplicateEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineIntegrityReport {
    protocol_context: OfflineIntegrityProtocolContext,
    store_identity: Option<Box<str>>,
    declared_limits: OfflineIntegrityObservationLimits,
    counters: OfflineIntegrityObservationCounters,
    completeness: OfflineIntegrityReportCompleteness,
    artifacts: Vec<OfflineArtifactObservation>,
    selected_root: Option<OfflineSelectedRootWitness>,
    selected_records: BTreeSet<Box<str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OfflineSelectedRootWitness {
    pub(crate) generation: u64,
    pub(crate) reference: u64,
}

impl OfflineArtifactObservation {
    pub(crate) fn new(
        relative_path: impl Into<Box<str>>,
        family: OfflineArtifactFamily,
        identity: PhysicalArtifactIdentity,
        generation: PhysicalArtifactGeneration,
        range: Option<PhysicalByteRange>,
        outcome: OfflineIntegrityOutcome,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            family,
            identity,
            generation,
            range,
            outcome,
            expected_point_page_touches: None,
            index_family: None,
            blob_reclaim_source_kind: None,
            duplicates: Vec::new(),
        }
    }

    pub(crate) fn with_duplicate(mut self, duplicate: OfflineArtifactDuplicateEvidence) -> Self {
        self.duplicates.push(duplicate);
        self
    }

    pub(crate) fn with_outcome(mut self, outcome: OfflineIntegrityOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub(crate) fn with_expected_point_page_touches(
        mut self,
        touches: u64,
        index_family: &'static str,
    ) -> Self {
        self.expected_point_page_touches = Some(touches);
        self.index_family = Some(index_family);
        self
    }

    pub(crate) fn with_blob_reclaim_source_kind(
        mut self,
        kind: OfflineBlobReclaimSourceKind,
    ) -> Self {
        self.blob_reclaim_source_kind = Some(kind);
        self
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }
    pub const fn family(&self) -> OfflineArtifactFamily {
        self.family
    }
    pub const fn identity(&self) -> &PhysicalArtifactIdentity {
        &self.identity
    }
    pub const fn generation(&self) -> PhysicalArtifactGeneration {
        self.generation
    }
    pub const fn range(&self) -> Option<PhysicalByteRange> {
        self.range
    }
    pub const fn outcome(&self) -> &OfflineIntegrityOutcome {
        &self.outcome
    }
    /// Independent selected-tree depth, emitted only for a complete intact
    /// B-tree closure; it is not a Store counter or device-I/O estimate.
    pub const fn expected_point_page_touches(&self) -> Option<u64> {
        self.expected_point_page_touches
    }
    pub const fn index_family(&self) -> Option<&'static str> {
        self.index_family
    }
    pub const fn blob_reclaim_source_kind(&self) -> Option<OfflineBlobReclaimSourceKind> {
        self.blob_reclaim_source_kind
    }
    pub fn duplicates(&self) -> &[OfflineArtifactDuplicateEvidence] {
        &self.duplicates
    }
}

impl From<PhysicalArtifactFamily> for OfflineArtifactFamily {
    fn from(value: PhysicalArtifactFamily) -> Self {
        Self::Declared(value)
    }
}

impl PartialEq<PhysicalArtifactFamily> for OfflineArtifactFamily {
    fn eq(&self, other: &PhysicalArtifactFamily) -> bool {
        matches!(self, Self::Declared(family) if family == other)
    }
}

impl OfflineArtifactFamily {
    pub const fn declared(self) -> Option<PhysicalArtifactFamily> {
        match self {
            Self::Declared(family) => Some(family),
            Self::OriginalDropReservation | Self::DedupeQuarantine => None,
            Self::Unrecognized => None,
        }
    }
}

impl OfflineIntegrityReport {
    pub(crate) fn new(
        protocol_context: OfflineIntegrityProtocolContext,
        store_identity: Option<Box<str>>,
        declared_limits: OfflineIntegrityObservationLimits,
        counters: OfflineIntegrityObservationCounters,
        completeness: OfflineIntegrityReportCompleteness,
        artifacts: Vec<OfflineArtifactObservation>,
        selected_root: Option<OfflineSelectedRootWitness>,
        selected_records: BTreeSet<Box<str>>,
    ) -> Self {
        // Reconciliation may discover source uncertainty after acquisition.
        // Reflect it in completeness without inventing an additional read.
        let completeness = if completeness == OfflineIntegrityReportCompleteness::Complete
            && artifacts.iter().any(|artifact| {
                matches!(
                    artifact.outcome(),
                    OfflineIntegrityOutcome::Indeterminate(_)
                )
            }) {
            OfflineIntegrityReportCompleteness::Indeterminate
        } else {
            completeness
        };
        Self {
            protocol_context,
            store_identity,
            declared_limits,
            counters,
            completeness,
            artifacts,
            selected_root,
            selected_records,
        }
    }

    pub const fn protocol_context(&self) -> &OfflineIntegrityProtocolContext {
        &self.protocol_context
    }
    pub const fn role_identity(&self) -> &'static str {
        OFFLINE_OBSERVER_ROLE_IDENTITY
    }
    pub fn store_identity(&self) -> Option<&str> {
        self.store_identity.as_deref()
    }
    pub const fn declared_limits(&self) -> OfflineIntegrityObservationLimits {
        self.declared_limits
    }
    pub const fn counters(&self) -> &OfflineIntegrityObservationCounters {
        &self.counters
    }
    pub const fn completeness(&self) -> OfflineIntegrityReportCompleteness {
        self.completeness
    }
    pub fn artifacts(&self) -> &[OfflineArtifactObservation] {
        &self.artifacts
    }
    pub(crate) const fn selected_root(&self) -> Option<OfflineSelectedRootWitness> {
        self.selected_root
    }
    pub(crate) fn selected_record(&self, identity: &str) -> bool {
        self.selected_records.contains(identity)
    }

    pub(crate) fn counters_mut(&mut self) -> &mut OfflineIntegrityObservationCounters {
        &mut self.counters
    }
}

#[cfg(test)]
mod selected_comparison_tests {
    use super::*;
    use crate::{encode_offline_selected_integrity_observation, PhysicalIntegrityComparisonLimits};

    const SELECTED: &str = "blob-record:222222222222222222222222222222220100000000000000";
    const HISTORICAL: &str = "blob-record:333333333333333333333333333333330200000000000000";

    #[test]
    fn selected_inventory_keeps_older_placement_generation_and_excludes_historical_row() {
        let artifact = |identity: &str, path: &str| {
            OfflineArtifactObservation::new(
                path,
                PhysicalArtifactFamily::BlobChunkFrame.into(),
                PhysicalArtifactIdentity::new(identity.to_owned()).unwrap(),
                PhysicalArtifactGeneration::encoded(3).unwrap(),
                None,
                OfflineIntegrityOutcome::Intact,
            )
        };
        let mut selected_records = BTreeSet::new();
        selected_records.insert(SELECTED.into());
        let report = OfflineIntegrityReport::new(
            OfflineIntegrityProtocolContext::new(
                "offline-observer",
                "offline-process",
                "offline-run",
                "old-placement-new-root",
            )
            .unwrap(),
            Some("11111111111111111111111111111111".into()),
            OfflineIntegrityObservationLimits::new(16, 8192, 6, 4, 0, 1000, 8192).unwrap(),
            OfflineIntegrityObservationCounters::default(),
            OfflineIntegrityReportCompleteness::Complete,
            vec![
                artifact(SELECTED, "families/records/arenas/old.data"),
                artifact(HISTORICAL, "families/records/arenas/historical.data"),
            ],
            Some(OfflineSelectedRootWitness {
                generation: 9,
                reference: 9,
            }),
            selected_records,
        );
        let wire = encode_offline_selected_integrity_observation(
            &report,
            PhysicalIntegrityComparisonLimits::default(),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&wire).unwrap();
        assert_eq!(value["selected_root"]["generation"], 9);
        assert_eq!(value["artifacts"].as_array().unwrap().len(), 1);
        assert_eq!(
            value["artifacts"][0]["record"],
            SELECTED.strip_prefix("blob-record:").unwrap()
        );
        assert_eq!(
            value["artifacts"][0]["physical_path"],
            "families/records/arenas/old.data"
        );
    }
}
