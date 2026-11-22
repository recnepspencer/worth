use worth_foundational::facade::AspectKey;
use worth_query_installation::facade::WorthQueryArtifactNativeLayoutReference;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactRowBatchRequest {
    layout: WorthQueryArtifactNativeLayoutReference,
    fields: Vec<AspectKey>,
    start_row: usize,
    max_rows: usize,
}

impl WorthQueryArtifactRowBatchRequest {
    pub fn new(
        layout: WorthQueryArtifactNativeLayoutReference,
        fields: impl IntoIterator<Item = AspectKey>,
        max_rows: usize,
    ) -> Self {
        Self {
            layout,
            fields: fields.into_iter().collect(),
            start_row: 0,
            max_rows,
        }
    }

    pub fn starting_at(mut self, start_row: usize) -> Self {
        self.start_row = start_row;
        self
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }

    pub fn fields(&self) -> &[AspectKey] {
        &self.fields
    }

    pub const fn start_row(&self) -> usize {
        self.start_row
    }

    pub const fn max_rows(&self) -> usize {
        self.max_rows
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactFieldSliceRequest {
    layout: WorthQueryArtifactNativeLayoutReference,
    field: AspectKey,
    start_row: usize,
    max_rows: usize,
}

impl WorthQueryArtifactFieldSliceRequest {
    pub fn new(
        layout: WorthQueryArtifactNativeLayoutReference,
        field: AspectKey,
        max_rows: usize,
    ) -> Self {
        Self {
            layout,
            field,
            start_row: 0,
            max_rows,
        }
    }

    pub fn starting_at(mut self, start_row: usize) -> Self {
        self.start_row = start_row;
        self
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }

    pub fn field(&self) -> &AspectKey {
        &self.field
    }

    pub const fn start_row(&self) -> usize {
        self.start_row
    }

    pub const fn max_rows(&self) -> usize {
        self.max_rows
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactChunkRequest {
    layout: WorthQueryArtifactNativeLayoutReference,
    fields: Vec<AspectKey>,
    chunk_rows: usize,
}

impl WorthQueryArtifactChunkRequest {
    pub fn new(
        layout: WorthQueryArtifactNativeLayoutReference,
        fields: impl IntoIterator<Item = AspectKey>,
        chunk_rows: usize,
    ) -> Self {
        Self {
            layout,
            fields: fields.into_iter().collect(),
            chunk_rows,
        }
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }

    pub fn fields(&self) -> &[AspectKey] {
        &self.fields
    }

    pub const fn chunk_rows(&self) -> usize {
        self.chunk_rows
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactProjectedChunkRequest {
    layout: WorthQueryArtifactNativeLayoutReference,
    projection_identity: String,
    chunk_rows: usize,
}

impl WorthQueryArtifactProjectedChunkRequest {
    pub fn new(
        layout: WorthQueryArtifactNativeLayoutReference,
        projection_identity: impl Into<String>,
        chunk_rows: usize,
    ) -> Self {
        Self {
            layout,
            projection_identity: projection_identity.into(),
            chunk_rows,
        }
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }

    pub fn projection_identity(&self) -> &str {
        &self.projection_identity
    }

    pub const fn chunk_rows(&self) -> usize {
        self.chunk_rows
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactScalarFallbackRequest {
    layout: WorthQueryArtifactNativeLayoutReference,
    fields: Vec<AspectKey>,
}

impl WorthQueryArtifactScalarFallbackRequest {
    pub fn new(
        layout: WorthQueryArtifactNativeLayoutReference,
        fields: impl IntoIterator<Item = AspectKey>,
    ) -> Self {
        Self {
            layout,
            fields: fields.into_iter().collect(),
        }
    }

    pub fn layout(&self) -> &WorthQueryArtifactNativeLayoutReference {
        &self.layout
    }

    pub fn fields(&self) -> &[AspectKey] {
        &self.fields
    }
}
