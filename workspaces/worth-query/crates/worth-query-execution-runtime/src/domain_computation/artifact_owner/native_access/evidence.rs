use worth_foundational::facade::AspectKey;
use worth_query_installation::facade::WorthQueryArtifactNativeLayoutReference;

use super::WorthQueryArtifactNativeAccessCounters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactNativeAccessBound {
    RowBatch {
        start_row: usize,
        max_rows: usize,
    },
    FieldSlice {
        start_row: usize,
        max_rows: usize,
    },
    Chunk {
        chunk_rows: usize,
    },
    Projection {
        projection_identity: String,
        chunk_rows: usize,
    },
    ScalarFallback {
        max_calls_per_admission: usize,
        max_call_amplification: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactNativeAccessEvidence {
    artifact_occurrence_identity: String,
    basis_identity: String,
    provider_session_identity: String,
    layout: WorthQueryArtifactNativeLayoutReference,
    requested_fields: Vec<AspectKey>,
    access_bound: WorthQueryArtifactNativeAccessBound,
    borrow_generation: u64,
    counters: WorthQueryArtifactNativeAccessCounters,
}

impl WorthQueryArtifactNativeAccessEvidence {
    pub fn new(
        artifact_occurrence_identity: String,
        basis_identity: String,
        provider_session_identity: String,
        layout: WorthQueryArtifactNativeLayoutReference,
        requested_fields: Vec<AspectKey>,
        access_bound: WorthQueryArtifactNativeAccessBound,
        borrow_generation: u64,
        counters: WorthQueryArtifactNativeAccessCounters,
    ) -> Self {
        Self {
            artifact_occurrence_identity,
            basis_identity,
            provider_session_identity,
            layout,
            requested_fields,
            access_bound,
            borrow_generation,
            counters,
        }
    }

    pub fn artifact_occurrence_identity(&self) -> &str {
        &self.artifact_occurrence_identity
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }

    pub fn requested_fields(&self) -> &[AspectKey] {
        &self.requested_fields
    }

    pub fn access_bound(&self) -> &WorthQueryArtifactNativeAccessBound {
        &self.access_bound
    }

    pub const fn borrow_generation(&self) -> u64 {
        self.borrow_generation
    }

    pub const fn counters(&self) -> WorthQueryArtifactNativeAccessCounters {
        self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactNativeAccessOutcome<T> {
    value: T,
    evidence: WorthQueryArtifactNativeAccessEvidence,
}

impl<T> WorthQueryArtifactNativeAccessOutcome<T> {
    pub fn new(value: T, evidence: WorthQueryArtifactNativeAccessEvidence) -> Self {
        Self { value, evidence }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn evidence(&self) -> &WorthQueryArtifactNativeAccessEvidence {
        &self.evidence
    }
}
