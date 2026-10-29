pub use worth_query::facade::domain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactNativeLane {
    BulkRows,
    FieldSlice,
    ChunkedRows,
    ScalarFallback,
    SummaryProjection,
    ProvenanceProjection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArtifactNativeCandidate {
    pub id: u64,
    pub score: f64,
}

impl ArtifactNativeCandidate {
    pub const fn new(id: u64, score: f64) -> Self {
        Self { id, score }
    }

    pub const fn id(self) -> u64 {
        self.id
    }

    pub const fn score(self) -> f64 {
        self.score
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArtifactNativeValues {
    Candidates(Vec<ArtifactNativeCandidate>),
    CandidateIds(Vec<u64>),
    Signatures(Vec<u64>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactNativeSuccess {
    pub lane: ArtifactNativeLane,
    pub values: ArtifactNativeValues,
    pub chunk_capacity_bytes: Vec<usize>,
    pub evidence: domain::WorthQueryArtifactNativeAccessEvidence,
}

impl ArtifactNativeSuccess {
    pub fn new(
        lane: ArtifactNativeLane,
        values: ArtifactNativeValues,
        chunk_capacity_bytes: Vec<usize>,
        evidence: domain::WorthQueryArtifactNativeAccessEvidence,
    ) -> Self {
        Self {
            lane,
            values,
            chunk_capacity_bytes,
            evidence,
        }
    }

    pub const fn lane(&self) -> ArtifactNativeLane {
        self.lane
    }

    pub fn values(&self) -> &ArtifactNativeValues {
        &self.values
    }

    pub fn chunk_capacity_bytes(&self) -> &[usize] {
        &self.chunk_capacity_bytes
    }

    pub fn evidence(&self) -> &domain::WorthQueryArtifactNativeAccessEvidence {
        &self.evidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactNativeDenial {
    pub kind: domain::WorthQueryArtifactNativeAccessDenialKind,
    pub counters: domain::WorthQueryArtifactNativeAccessCounters,
}

impl ArtifactNativeDenial {
    pub const fn new(denial: &domain::WorthQueryArtifactNativeAccessDenial) -> Self {
        Self {
            kind: denial.kind(),
            counters: denial.counters(),
        }
    }

    pub const fn kind(&self) -> domain::WorthQueryArtifactNativeAccessDenialKind {
        self.kind
    }

    pub const fn counters(&self) -> domain::WorthQueryArtifactNativeAccessCounters {
        self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArtifactNativeObservation {
    Success(ArtifactNativeSuccess),
    Denied(ArtifactNativeDenial),
}
