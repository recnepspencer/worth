use worth_foundational::facade::{AspectKey, AspectValue};
use worth_query_installation::facade::WorthQueryArtifactNativeLayoutReference;

use super::{
    WorthQueryArtifactProjectionSink, WorthQueryArtifactProviderAccessSession,
    WorthQueryArtifactProviderFieldSlice, WorthQueryArtifactProviderValueView,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryArtifactProviderAccessDenial {
    Unsupported,
    SessionMismatch,
    LayoutMismatch,
    AlignmentMismatch,
    BoundsExceeded,
    ShapeMismatch,
    NoProgress,
    Failed,
}

pub struct WorthQueryArtifactProviderBorrowedBatch<'a> {
    start_row: usize,
    row_count: usize,
    columns: Vec<(AspectKey, WorthQueryArtifactProviderFieldSlice<'a>)>,
}

impl<'a> WorthQueryArtifactProviderBorrowedBatch<'a> {
    pub fn new(
        start_row: usize,
        row_count: usize,
        columns: impl IntoIterator<Item = (AspectKey, WorthQueryArtifactProviderFieldSlice<'a>)>,
    ) -> Self {
        Self {
            start_row,
            row_count,
            columns: columns.into_iter().collect(),
        }
    }

    pub const fn start_row(&self) -> usize {
        self.start_row
    }

    pub const fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn source_bytes(&self) -> usize {
        self.columns
            .iter()
            .map(|(_, values)| values.physical_bytes())
            .sum()
    }

    pub fn into_columns(self) -> Vec<(AspectKey, WorthQueryArtifactProviderFieldSlice<'a>)> {
        self.columns
    }
}

pub trait WorthQueryArtifactNativeAccessProvider: Send {
    fn layout(&self) -> WorthQueryArtifactNativeLayoutReference;

    fn row_count(
        &self,
        session: &WorthQueryArtifactProviderAccessSession,
    ) -> Result<usize, WorthQueryArtifactProviderAccessDenial>;

    fn borrow_rows<'a>(
        &'a self,
        session: &WorthQueryArtifactProviderAccessSession,
        start_row: usize,
        max_rows: usize,
        fields: &[AspectKey],
    ) -> Result<WorthQueryArtifactProviderBorrowedBatch<'a>, WorthQueryArtifactProviderAccessDenial>;

    fn borrow_field<'a>(
        &'a self,
        _session: &WorthQueryArtifactProviderAccessSession,
        _start_row: usize,
        _max_rows: usize,
        _field: &AspectKey,
    ) -> Result<WorthQueryArtifactProviderFieldSlice<'a>, WorthQueryArtifactProviderAccessDenial>
    {
        Err(WorthQueryArtifactProviderAccessDenial::Unsupported)
    }

    fn project_rows(
        &self,
        _session: &WorthQueryArtifactProviderAccessSession,
        _projection_identity: &str,
        _start_row: usize,
        _max_rows: usize,
        _sink: &mut WorthQueryArtifactProjectionSink,
    ) -> Result<usize, WorthQueryArtifactProviderAccessDenial> {
        Err(WorthQueryArtifactProviderAccessDenial::Unsupported)
    }

    fn scalar<'a>(
        &'a self,
        _session: &WorthQueryArtifactProviderAccessSession,
        _row: usize,
        _field: &AspectKey,
    ) -> Result<WorthQueryArtifactProviderValueView<'a>, WorthQueryArtifactProviderAccessDenial>
    {
        Err(WorthQueryArtifactProviderAccessDenial::Unsupported)
    }
}

pub fn value_matches_contract(
    value: &AspectValue,
    contract: &worth_foundational::facade::AspectContract,
) -> bool {
    matches!(
        contract.shape(),
        worth_foundational::facade::AspectShape::Scalar(family)
            if value.value_family() == *family
    )
}
