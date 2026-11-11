//! Setup grants follow the lowerer/readmitter's real phase geometry.
use super::*;

pub(super) fn lower_peak(source: &AspectFieldPatch, plan: &LoweredAspectContractPlan) -> u64 {
    if source.is_empty() {
        return 0; // Creation returns its actual empty candidate before scope entry.
    }
    let rows = selection::select(source, plan, FieldAuthoringDomain::Entity).unwrap();
    let (retained, transient) =
        allocation::input(&rows, PortablePatchReadmissionPurpose::RecordCreation).unwrap();
    retained
        .checked_add(transient)
        .unwrap()
        .checked_add(allocation::backing::<selection::Selected<'_>>(source.len()).unwrap())
        .unwrap()
        // Each lowerer reservation uses the real native scope with R = ().
        .checked_add(scope_entry(|| ()))
        .unwrap()
}

pub(super) fn full_peak(source: &AspectFieldPatch, plan: &LoweredAspectContractPlan) -> u64 {
    let rows = selection::select(source, plan, FieldAuthoringDomain::Entity).unwrap();
    let (retained, _) =
        allocation::input(&rows, PortablePatchReadmissionPurpose::RecordCreation).unwrap();
    let candidate = PortableRecordAspectPatch::new(
        selection::build(&rows, PortablePatchReadmissionPurpose::RecordCreation).unwrap(),
    );
    lower_peak(source, plan).max(
        allocation::readmission_bytes(retained, &candidate, Some(plan))
            .unwrap()
            // Growing and later shrinking the same ticket both enter the
            // resizer's actual Result<(), Cause> native callback shape.
            .checked_add(scope_entry(|| Ok::<(), Cause>(())))
            .unwrap(),
    )
}

// Describe actual framework entry only, not a measured lowerer/payload peak.
// The two callers instantiate the exact public callback result types used by
// reserve_retained_memory and allocation::resize respectively.
fn scope_entry<R>(completed: impl FnOnce() -> R) -> u64 {
    let budget = SerialMemoryBudget::new(None);
    let host = host(&budget);
    let request = ExecutionRequest::serial(&host);
    let mut entry = None;
    request
        .run_native_with_checkpoint(|_| {
            entry = Some(budget.charged_memory_bytes());
            completed()
        })
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    entry.expect("the actual typed native scope was entered")
}
