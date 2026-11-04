use worth_foundational::facade::{AspectKey, AspectShape};

use super::thread_bound::WorthQueryArtifactThreadBound;
use super::{
    WorthQueryArtifactNativeAccessAdmission, WorthQueryArtifactNativeAccessCounters,
    WorthQueryArtifactNativeAccessDenial, WorthQueryArtifactNativeFieldSlice,
    WorthQueryArtifactNativeValueView, WorthQueryArtifactProviderAccessDenial,
    WorthQueryArtifactProviderFieldSlice,
};

pub struct WorthQueryArtifactBorrowedRowBatch<'a> {
    start_row: usize,
    row_count: usize,
    columns: Vec<(AspectKey, WorthQueryArtifactProviderFieldSlice<'a>)>,
    _thread_bound: WorthQueryArtifactThreadBound,
}

pub(crate) struct WorthQueryArtifactBorrowedRowsOutcome<T> {
    pub(crate) value: T,
    pub(crate) row_count: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum WorthQueryArtifactRowProgress {
    MayBeEmpty,
    MustAdvance,
}

pub(crate) fn with_borrowed_rows<T>(
    admission: &mut WorthQueryArtifactNativeAccessAdmission<'_>,
    start_row: usize,
    max_rows: usize,
    fields: &[AspectKey],
    progress: WorthQueryArtifactRowProgress,
    consume: impl for<'view> FnOnce(WorthQueryArtifactBorrowedRowBatch<'view>) -> T,
) -> Result<WorthQueryArtifactBorrowedRowsOutcome<T>, WorthQueryArtifactNativeAccessDenial> {
    let layout = admission.native_contract().layout().clone();
    let requested = fields.to_vec();
    let (value, row_count, increment) = admission.with_provider(|provider, session| {
        let batch = provider.borrow_rows(session, start_row, max_rows, &requested)?;
        let row_count = batch.row_count();
        let source_bytes = batch.source_bytes();
        if batch.start_row() != start_row || row_count > max_rows {
            return Err(WorthQueryArtifactProviderAccessDenial::BoundsExceeded);
        }
        if matches!(progress, WorthQueryArtifactRowProgress::MustAdvance) && row_count == 0 {
            return Err(WorthQueryArtifactProviderAccessDenial::NoProgress);
        }
        let columns = batch.into_columns();
        if columns.len() != requested.len() {
            return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
        }
        for ((field, values), expected) in columns.iter().zip(&requested) {
            if field != expected || values.len() != row_count {
                return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            }
            let Some(contract) = layout
                .fields()
                .iter()
                .find(|contract| contract.aspect().key() == field)
            else {
                return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            };
            let shape_matches = match contract.aspect().shape() {
                AspectShape::Scalar(family) => values.matches_scalar_family(*family),
                AspectShape::Struct(_) => values.is_struct(),
                AspectShape::Opaque(_) | AspectShape::Reference(_) => false,
            };
            if !shape_matches {
                return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            }
        }
        let increment = WorthQueryArtifactNativeAccessCounters {
            row_batch_contacts: 1,
            rows_exposed: row_count,
            values_exposed: row_count.saturating_mul(columns.len()),
            source_bytes,
            ..WorthQueryArtifactNativeAccessCounters::default()
        };
        Ok((
            consume(WorthQueryArtifactBorrowedRowBatch::new(
                start_row, row_count, columns,
            )),
            row_count,
            increment,
        ))
    })?;
    admission.counters_mut().accumulate(increment);
    Ok(WorthQueryArtifactBorrowedRowsOutcome { value, row_count })
}

impl<'a> WorthQueryArtifactBorrowedRowBatch<'a> {
    pub(crate) fn new(
        start_row: usize,
        row_count: usize,
        columns: Vec<(AspectKey, WorthQueryArtifactProviderFieldSlice<'a>)>,
    ) -> Self {
        Self {
            start_row,
            row_count,
            columns,
            _thread_bound: WorthQueryArtifactThreadBound::new(),
        }
    }

    pub const fn start_row(&self) -> usize {
        self.start_row
    }

    pub const fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn rows(&self) -> impl ExactSizeIterator<Item = WorthQueryArtifactBorrowedRow<'_>> {
        (0..self.row_count).map(|row| WorthQueryArtifactBorrowedRow { batch: self, row })
    }

    pub fn field_slice(&self, field: &AspectKey) -> Option<WorthQueryArtifactNativeFieldSlice<'a>> {
        self.columns
            .iter()
            .find(|(candidate, _)| candidate == field)
            .map(|(_, values)| WorthQueryArtifactNativeFieldSlice::from_provider(*values))
    }
}

pub struct WorthQueryArtifactBorrowedRow<'a> {
    batch: &'a WorthQueryArtifactBorrowedRowBatch<'a>,
    row: usize,
}

impl<'a> WorthQueryArtifactBorrowedRow<'a> {
    pub fn absolute_row(&self) -> usize {
        self.batch.start_row + self.row
    }

    pub fn field(&self, field: &AspectKey) -> Option<WorthQueryArtifactNativeValueView<'a>> {
        self.batch.field_slice(field)?.value(self.row)
    }
}
