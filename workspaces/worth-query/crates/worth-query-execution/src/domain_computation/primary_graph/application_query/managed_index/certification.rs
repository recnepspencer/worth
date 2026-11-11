//! Cold certification evidence; ordinary execution never constructs these reports.
use super::nodes::pages::inspection::PageInspection;
use super::WorthQueryManagedIndexStop;
use worth_execution::{ExecutionWorkCeiling, SerialRequest, WorkCeilingDenial};
use worth_relational::facade::runtime::ManagedIndexSourceProgram;

/// One Source program across the registry's currently admitted branch images.
/// Counts immutable provider node pages, excluding directory and Source-row storage.
/// Historical images held only by old read pins are outside this observation.
#[derive(Debug)]
pub struct WorthQueryCertificationManagedIndexPages {
    pub program: ManagedIndexSourceProgram,
    pub retained_roots: usize,
    pub distinct_pages: usize,
    pub page_references: usize,
}
pub(in crate::domain_computation::primary_graph) struct ProgramPages {
    pub program: ManagedIndexSourceProgram,
    pub roots: usize,
    pub pages: PageInspection,
}
impl ProgramPages {
    pub fn finish(self) -> WorthQueryCertificationManagedIndexPages {
        WorthQueryCertificationManagedIndexPages {
            program: self.program,
            retained_roots: self.roots,
            distinct_pages: self.pages.distinct(),
            page_references: self.pages.references,
        }
    }
}

/// Whole original SDK invoice, including charged node visits and provider work.
/// This is the SDK's accepted charged work, not a second counter or a production cost contract.
/// Supply the original request so memory, cancellation and deadline remain caller-owned.
pub fn measure_certification_request<T>(
    original: &SerialRequest,
    operation: impl FnOnce() -> T,
) -> Result<(T, u64), WorkCeilingDenial> {
    let (result, report) =
        ExecutionWorkCeiling::new(None)
            .run_with_checkpoint(None, Some(original), |_| operation())?;
    Ok((result, report.charged_work()))
}

/// Typed failure rather than an incomplete or silently zero page count.
#[derive(Debug)]
pub enum WorthQueryCertificationManagedIndexPagesDenial {
    MissingProgram,
    Material(WorthQueryManagedIndexStop),
}
