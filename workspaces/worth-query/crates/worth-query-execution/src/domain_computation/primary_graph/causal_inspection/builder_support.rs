use super::builder::CausalInspection;
use super::request::{CausalInspectionExplanationFamily, CausalInspectionRichness};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionSupport {
    rows: Vec<CausalInspectionSupportRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionSupportRow {
    explanation_family: CausalInspectionExplanationFamily,
    default_richness: CausalInspectionRichness,
    posture: CausalInspectionSupportRowPosture,
    note: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalInspectionSupportRowPosture {
    Supported,
    Advisory,
    Deferred,
}

impl CausalInspectionSupportRowPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Advisory => "advisory",
            Self::Deferred => "deferred",
        }
    }
}

impl CausalInspection {
    pub fn support() -> CausalInspectionSupport {
        CausalInspectionSupport {
            rows: vec![
                CausalInspectionSupportRow::new(
                    CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
                    CausalInspectionRichness::ReferenceOnly,
                    CausalInspectionSupportRowPosture::Supported,
                    "reference-only cross-runtime causal inspection is the common path",
                ),
                CausalInspectionSupportRow::new(
                    CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
                    CausalInspectionRichness::MaterializedDetail,
                    CausalInspectionSupportRowPosture::Advisory,
                    "materialized detail narrows until bridge envelope materialization",
                ),
                CausalInspectionSupportRow::new(
                    CausalInspectionExplanationFamily::DurableCausalArchive,
                    CausalInspectionRichness::ReferenceOnly,
                    CausalInspectionSupportRowPosture::Deferred,
                    "durable causal archives are later-milestone debt",
                ),
                CausalInspectionSupportRow::new(
                    CausalInspectionExplanationFamily::StoreBackedReplayReconstruction,
                    CausalInspectionRichness::ReferenceOnly,
                    CausalInspectionSupportRowPosture::Deferred,
                    "store-backed replay reconstruction is later-milestone debt",
                ),
            ],
        }
    }
}

impl CausalInspectionSupport {
    pub fn rows(&self) -> &[CausalInspectionSupportRow] {
        &self.rows
    }

    pub fn explain(&self) -> CausalInspectionSupportExplanation {
        CausalInspectionSupportExplanation {
            supported_row_count: self
                .rows
                .iter()
                .filter(|row| row.posture == CausalInspectionSupportRowPosture::Supported)
                .count(),
            advisory_row_count: self
                .rows
                .iter()
                .filter(|row| row.posture == CausalInspectionSupportRowPosture::Advisory)
                .count(),
            deferred_row_count: self
                .rows
                .iter()
                .filter(|row| row.posture == CausalInspectionSupportRowPosture::Deferred)
                .count(),
        }
    }
}

impl CausalInspectionSupportRow {
    fn new(
        explanation_family: CausalInspectionExplanationFamily,
        default_richness: CausalInspectionRichness,
        posture: CausalInspectionSupportRowPosture,
        note: &'static str,
    ) -> Self {
        Self {
            explanation_family,
            default_richness,
            posture,
            note,
        }
    }

    pub fn explanation_family(&self) -> CausalInspectionExplanationFamily {
        self.explanation_family
    }

    pub fn default_richness(&self) -> CausalInspectionRichness {
        self.default_richness
    }

    pub fn posture(&self) -> CausalInspectionSupportRowPosture {
        self.posture
    }

    pub fn note(&self) -> &'static str {
        self.note
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalInspectionSupportExplanation {
    supported_row_count: usize,
    advisory_row_count: usize,
    deferred_row_count: usize,
}

impl CausalInspectionSupportExplanation {
    pub fn supported_row_count(&self) -> usize {
        self.supported_row_count
    }

    pub fn advisory_row_count(&self) -> usize {
        self.advisory_row_count
    }

    pub fn deferred_row_count(&self) -> usize {
        self.deferred_row_count
    }
}
