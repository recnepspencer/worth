#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadExecutionEngine {
    QueryRuntimeCurrent,
    QueryRuntimeBranch,
    QueryRuntimeHistorical,
    QueryRuntimePreviewDerived,
}

impl WorthQueryReadExecutionEngine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryRuntimeCurrent => "query-runtime-current",
            Self::QueryRuntimeBranch => "query-runtime-branch",
            Self::QueryRuntimeHistorical => "query-runtime-historical",
            Self::QueryRuntimePreviewDerived => "query-runtime-preview-derived",
        }
    }
}
