use super::{
    WorthQueryArtifactBulkProjectionContract, WorthQueryArtifactChunkContract,
    WorthQueryArtifactNativeLayoutContract, WorthQueryArtifactRowBatchPosture,
    WorthQueryArtifactScalarFallbackPosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactAccessPathContract {
    Denied,
    Native(WorthQueryArtifactNativeAccessContract),
}

impl WorthQueryArtifactAccessPathContract {
    pub const fn denied() -> Self {
        Self::Denied
    }

    pub fn native(contract: WorthQueryArtifactNativeAccessContract) -> Self {
        Self::Native(contract)
    }

    pub fn native_contract(&self) -> Option<&WorthQueryArtifactNativeAccessContract> {
        match self {
            Self::Denied => None,
            Self::Native(contract) => Some(contract),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactNativeAccessContract {
    layout: WorthQueryArtifactNativeLayoutContract,
    row_batch: WorthQueryArtifactRowBatchPosture,
    chunks: Option<WorthQueryArtifactChunkContract>,
    bulk_projections: Vec<WorthQueryArtifactBulkProjectionContract>,
    scalar_fallback: WorthQueryArtifactScalarFallbackPosture,
}

impl WorthQueryArtifactNativeAccessContract {
    pub fn new(
        layout: WorthQueryArtifactNativeLayoutContract,
        row_batch: WorthQueryArtifactRowBatchPosture,
        chunks: Option<WorthQueryArtifactChunkContract>,
        bulk_projections: impl IntoIterator<Item = WorthQueryArtifactBulkProjectionContract>,
        scalar_fallback: WorthQueryArtifactScalarFallbackPosture,
    ) -> Self {
        let mut bulk_projections = bulk_projections.into_iter().collect::<Vec<_>>();
        bulk_projections.sort_by(|left, right| left.identity().cmp(right.identity()));
        Self {
            layout,
            row_batch,
            chunks,
            bulk_projections,
            scalar_fallback,
        }
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutContract {
        &self.layout
    }

    pub const fn row_batch(&self) -> WorthQueryArtifactRowBatchPosture {
        self.row_batch
    }

    pub const fn chunks(&self) -> Option<WorthQueryArtifactChunkContract> {
        self.chunks
    }

    pub fn bulk_projections(&self) -> &[WorthQueryArtifactBulkProjectionContract] {
        &self.bulk_projections
    }

    pub const fn scalar_fallback(&self) -> WorthQueryArtifactScalarFallbackPosture {
        self.scalar_fallback
    }
}
