//! Observer of the coupled commit patches a post-commit rebase walks.
use std::cell::Cell;
thread_local! { static WALKED: Cell<u64> = const { Cell::new(0) }; }
pub(super) fn record() {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    WALKED.with(|walked| walked.set(walked.get().saturating_add(1)));
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn coupled_commit_patches_walked_on_this_thread_for_test(&self) -> u64 {
        WALKED.with(Cell::get)
    }
}

// Law 2, scope: passive thread records end with the owned observation lifetime.
thread_local! { static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    WALKED.with(|walked| walked.set(0));
}
