//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::WorthQueryAdmittedOperation::observe_inline_budget(census);
    let _typed=census.owner::<super::WorthQueryAdmittedApplicationOperation<(),(),(),()>>(
        "workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/operation_admission.rs",
        "WorthQueryAdmittedApplicationOperation",512);
    super::denial::inline_type_budget::collect(census);
    super::operation_scope_binding::inline_type_budget::collect(census);
}
