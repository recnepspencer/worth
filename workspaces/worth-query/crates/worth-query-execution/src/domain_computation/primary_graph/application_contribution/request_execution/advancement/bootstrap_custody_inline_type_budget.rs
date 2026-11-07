//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<WorthQueryBootstrapAdvancementPhase<'static>>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/primary_graph/application_contribution/request_execution/advancement/bootstrap_custody.rs", "WorthQueryBootstrapAdvancementPhase", 128 * 1024);
        owner.field::<ExecutionRequest<'static, 'static>>("request");
        owner.field::<std::marker::PhantomData<std::rc::Rc<()>>>("_thread");
    }
}
