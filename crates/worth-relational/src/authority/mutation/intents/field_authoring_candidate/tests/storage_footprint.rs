//! Setup grants follow the lowerer/readmitter's real phase geometry.
use super::*;

pub(super) fn lower_peak(source: &AspectFieldPatch, plan: &LoweredAspectContractPlan) -> u64 {
    let rows = selection::select(source, plan, FieldAuthoringDomain::Entity).unwrap();
    let (retained, transient) =
        allocation::input(&rows, PortablePatchReadmissionPurpose::RecordCreation).unwrap();
    retained
        .checked_add(transient)
        .unwrap()
        .checked_add(allocation::backing::<selection::Selected<'_>>(source.len()).unwrap())
        .unwrap()
}

pub(super) fn full_peak(source: &AspectFieldPatch, plan: &LoweredAspectContractPlan) -> u64 {
    let rows = selection::select(source, plan, FieldAuthoringDomain::Entity).unwrap();
    let (retained, _) =
        allocation::input(&rows, PortablePatchReadmissionPurpose::RecordCreation).unwrap();
    let candidate = PortableRecordAspectPatch::new(
        selection::build(&rows, PortablePatchReadmissionPurpose::RecordCreation).unwrap(),
    );
    lower_peak(source, plan)
        .max(allocation::readmission_bytes(retained, &candidate, Some(plan)).unwrap())
}
