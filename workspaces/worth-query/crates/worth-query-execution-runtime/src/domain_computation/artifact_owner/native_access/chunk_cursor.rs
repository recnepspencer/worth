use worth_query_installation::facade::WorthQueryArtifactBulkProjectionContract;

use super::row_batch::with_borrowed_rows;
use super::row_batch::WorthQueryArtifactRowProgress;
use super::thread_bound::WorthQueryArtifactThreadBound;
use super::{
    WorthQueryArtifactChunkRequest, WorthQueryArtifactNativeAccessAdmission,
    WorthQueryArtifactNativeAccessCounters, WorthQueryArtifactNativeAccessDenial,
    WorthQueryArtifactNativeAccessEvidence, WorthQueryArtifactProjectedChunkRequest,
    WorthQueryArtifactProjectionSink, WorthQueryArtifactProviderAccessDenial,
};

pub struct WorthQueryArtifactChunkCursor<'a> {
    admission: WorthQueryArtifactNativeAccessAdmission<'a>,
    request: WorthQueryArtifactChunkRequest,
    next_row: usize,
    row_count: usize,
}

impl<'a> WorthQueryArtifactChunkCursor<'a> {
    pub fn new(
        mut admission: WorthQueryArtifactNativeAccessAdmission<'a>,
        request: WorthQueryArtifactChunkRequest,
    ) -> Result<Self, WorthQueryArtifactNativeAccessDenial> {
        let row_count = admission.with_provider(|provider, session| provider.row_count(session))?;
        Ok(Self {
            admission,
            request,
            next_row: 0,
            row_count,
        })
    }

    pub fn next<T>(
        &mut self,
        consume: impl for<'view> FnOnce(super::WorthQueryArtifactBorrowedRowBatch<'view>) -> T,
    ) -> Result<Option<T>, WorthQueryArtifactNativeAccessDenial> {
        if self.next_row >= self.row_count {
            return Ok(None);
        }
        let width = self
            .request
            .chunk_rows()
            .min(self.row_count - self.next_row);
        let outcome = with_borrowed_rows(
            &mut self.admission,
            self.next_row,
            width,
            self.request.fields(),
            WorthQueryArtifactRowProgress::MustAdvance,
            consume,
        )?;
        self.admission.counters_mut().chunk_contacts += 1;
        self.next_row += outcome.row_count;
        Ok(Some(outcome.value))
    }

    pub fn evidence(&self) -> WorthQueryArtifactNativeAccessEvidence {
        self.admission.evidence()
    }
}

pub struct WorthQueryArtifactProjectedChunkCursor<'a> {
    admission: WorthQueryArtifactNativeAccessAdmission<'a>,
    request: WorthQueryArtifactProjectedChunkRequest,
    projection: WorthQueryArtifactBulkProjectionContract,
    next_row: usize,
    row_count: usize,
}

impl<'a> WorthQueryArtifactProjectedChunkCursor<'a> {
    pub fn new(
        mut admission: WorthQueryArtifactNativeAccessAdmission<'a>,
        request: WorthQueryArtifactProjectedChunkRequest,
        projection: WorthQueryArtifactBulkProjectionContract,
    ) -> Result<Self, WorthQueryArtifactNativeAccessDenial> {
        let row_count = admission.with_provider(|provider, session| provider.row_count(session))?;
        Ok(Self {
            admission,
            request,
            projection,
            next_row: 0,
            row_count,
        })
    }

    pub fn next<T>(
        &mut self,
        consume: impl for<'view> FnOnce(WorthQueryArtifactProjectedChunkView<'view>) -> T,
    ) -> Result<Option<T>, WorthQueryArtifactNativeAccessDenial> {
        if self.next_row >= self.row_count {
            return Ok(None);
        }
        let width = self
            .request
            .chunk_rows()
            .min(self.row_count - self.next_row);
        let mut sink = WorthQueryArtifactProjectionSink::new(
            self.projection.destination_fields().to_vec(),
            width,
            self.projection.destination_alignment(),
        )
        .map_err(|_| {
            self.admission.denial(
                super::WorthQueryArtifactNativeAccessDenialKind::AlignmentMismatch,
                "declared projection alignment exceeds the bounded destination sink",
            )
        })?;
        let start_row = self.next_row;
        let projection_identity = self.request.projection_identity().to_owned();
        let projected_rows = self.admission.with_provider(|provider, session| {
            provider.project_rows(session, &projection_identity, start_row, width, &mut sink)
        })?;
        if projected_rows != sink.row_count() || projected_rows > width {
            return Err(WorthQueryArtifactNativeAccessDenial::new(
                super::WorthQueryArtifactNativeAccessDenialKind::ProviderShapeMismatch,
                None,
                "provider projected row count does not match the bounded destination sink",
                self.admission.evidence().counters(),
            ));
        }
        if projected_rows == 0 {
            self.admission.counters_mut().projection_contacts += 1;
            return Err(provider_progress_denial(
                self.admission.evidence().counters(),
            ));
        }
        let result_bytes = sink.result_semantic_bytes();
        let capacity_bytes = sink.allocated_capacity_bytes();
        let value = consume(WorthQueryArtifactProjectedChunkView {
            start_row,
            sink: &sink,
            _thread_bound: WorthQueryArtifactThreadBound::new(),
        });
        self.admission
            .counters_mut()
            .accumulate(WorthQueryArtifactNativeAccessCounters {
                chunk_contacts: 1,
                projection_contacts: 1,
                rows_exposed: projected_rows,
                values_exposed: projected_rows.saturating_mul(sink.field_count()),
                result_bytes,
                peak_result_capacity_bytes: capacity_bytes,
                ..WorthQueryArtifactNativeAccessCounters::default()
            });
        self.next_row += projected_rows;
        Ok(Some(value))
    }

    pub fn evidence(&self) -> WorthQueryArtifactNativeAccessEvidence {
        self.admission.evidence()
    }
}

pub struct WorthQueryArtifactProjectedChunkView<'a> {
    start_row: usize,
    sink: &'a WorthQueryArtifactProjectionSink,
    _thread_bound: WorthQueryArtifactThreadBound,
}

impl WorthQueryArtifactProjectedChunkView<'_> {
    pub const fn start_row(&self) -> usize {
        self.start_row
    }

    pub fn row_count(&self) -> usize {
        self.sink.row_count()
    }

    pub fn row(&self, row: usize) -> Option<&[worth_foundational::facade::AspectValue]> {
        self.sink.row(row)
    }

    pub fn allocated_capacity_bytes(&self) -> usize {
        self.sink.allocated_capacity_bytes()
    }
}

fn provider_progress_denial(
    counters: WorthQueryArtifactNativeAccessCounters,
) -> WorthQueryArtifactNativeAccessDenial {
    let _ = WorthQueryArtifactProviderAccessDenial::Failed;
    WorthQueryArtifactNativeAccessDenial::new(
        super::WorthQueryArtifactNativeAccessDenialKind::ProviderDenied,
        None,
        "provider returned an empty non-terminal native projection chunk",
        counters,
    )
}
