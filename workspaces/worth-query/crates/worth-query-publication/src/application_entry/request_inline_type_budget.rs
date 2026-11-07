//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryApplicationRequest<'static, 'static, 'static, ()>>("workspaces/worth-query/crates/worth-query-publication/src/application_entry/request.rs", "WorthQueryApplicationRequest", 128 * 1024);
        owner.field::<&'static WorthQueryPrimaryGraphApplicationRuntime<()>>("application");
        owner.field::<&'static WorthQueryAuthenticatedExternalPrincipal<()>>("principal");
        owner.field::<&'static WorthQueryRequestScope>("scope");
        owner.field::<worth_query_execution::facade::product::WorthQueryProductBranch>("branch");
    }
    super::program_outputs::inline_type_budget::collect(census);
}
