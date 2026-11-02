//! Actual component cleanup tests carrier/final-poll protocol, not Source rows.
use super::*;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCloseScope, WorthQueryProductBranchOwnerCleanup,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::branch::{DeletedRelationalBranch, RelationalBranchIdentity};

struct Probe {
    runtime: u64,
    outer: Mutex<()>,
    entry: Mutex<Option<Arc<CleanupEntry>>>,
    notifications: AtomicUsize,
}
impl WorthQueryProductRetentionFollower for Probe {
    fn history_released(&self) {}
    fn manages_native_sources(&self, identity: &RelationalBranchIdentity) -> bool {
        // An observing receiver for this exact owner, not a claimed Source installation.
        identity.runtime_instance_id() == self.runtime
    }
    fn native_branch_deleted(&self, deleted: &DeletedRelationalBranch) {
        assert_eq!(deleted.identity().runtime_instance_id(), self.runtime);
        assert!(
            self.outer.try_lock().is_ok(),
            "notification is beyond caller coordination"
        );
        let entry = self.entry.lock().unwrap().as_ref().unwrap().clone();
        assert!(
            entry.record.try_lock().is_ok(),
            "notification is beyond actual cleanup entry mutex"
        );
        self.notifications.fetch_add(1, Ordering::SeqCst);
    }
}
struct Hook(WorthQueryProductBranchOwnerCleanupRegistry);
impl Drop for Hook {
    fn drop(&mut self) {
        self.0.replace_after_component_progress_hook(None);
    }
}
fn with_cleanup(
    fork_signal: bool,
    body: impl FnOnce(WorthQueryProductRuntime, WorthQueryProductBranchOwnerCleanup, Arc<Probe>),
) {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let branch = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|c| {
                    let c = c.fork_relational();
                    if fork_signal {
                        c.fork_signal()
                    } else {
                        c.reuse_exact_signal_basis()
                    }
                })
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let mut runtime = world.application.runtime.product_runtime.clone();
    // Isolate this protocol observer from the real provider's follower. The World
    // reference, Native/Signal components and deletion ports are the real owners.
    runtime.owner_cleanup = WorthQueryProductBranchOwnerCleanupRegistry::new(None);
    let probe = Arc::new(Probe {
        runtime: runtime
            .source
            .authoritative_source_profile()
            .runtime_instance_id(),
        outer: Mutex::new(()),
        entry: Mutex::new(None),
        notifications: AtomicUsize::new(0),
    });
    let follower: Arc<dyn WorthQueryProductRetentionFollower> = probe.clone();
    runtime
        .owner_cleanup
        .follow_product_retention(Arc::downgrade(&follower));
    let pending = runtime
        .begin_product_branch_close(branch, WorthQueryProductBranchCloseScope::Workspace)
        .unwrap();
    let (_, cleanup) = pending.into_cleanup();
    *probe.entry.lock().unwrap() = lock(&runtime.owner_cleanup.state)
        .entries
        .values()
        .next()
        .cloned();
    body(runtime, cleanup, probe);
}

#[test]
fn managed_source_retirement_partial_failure_notifies_only_beyond_outer_and_entry_guards() {
    with_cleanup(true, |runtime, cleanup, probe| {
        let memory = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
        let cancel = CancellationSource::new();
        let host = SerialRequest::from_memory(memory.clone(), cancel.token(), None);
        let token = cancel.clone();
        runtime
            .owner_cleanup
            .replace_after_component_progress_hook(Some(Box::new(move || token.cancel())));
        let hook = Hook(runtime.owner_cleanup.clone());
        let mut retirements = OwnerCleanupRetirements::default();
        let outer = probe.outer.lock().unwrap();
        let failed = cleanup
            .retry_in_request_with_retirements(&ExecutionRequest::serial(&host), &mut retirements)
            .expect_err("Native completed but the next Signal checkpoint must stop");
        assert_eq!(failed.denial(), WorthQueryProductBranchOwnerCleanupDenial::Execution(
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Interrupted(
                crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption::Cancelled)));
        assert_eq!(probe.notifications.load(Ordering::SeqCst), 0);
        let cleanup = failed.into_cleanup();
        assert!(matches!(
            cleanup.pending_work().as_slice(),
            [WorthQueryProductBranchOwnerCleanupWork::SignalBranch { .. }]
        ));
        assert!(
            memory.charged_memory_bytes() > 0,
            "original request owns actual deferred metadata"
        );
        drop(outer);
        drop(retirements);
        assert_eq!(probe.notifications.load(Ordering::SeqCst), 1);
        assert_eq!(memory.charged_memory_bytes(), 0);
        drop(hook);
        assert_eq!(cleanup.retry().unwrap().retired_component_count(), 2);
        assert_eq!(
            probe.notifications.load(Ordering::SeqCst),
            1,
            "Native deletion is not repeated"
        );
        assert!(runtime.pending_owner_cleanup().is_empty());
    });
}

#[test]
fn managed_source_retirement_completed_cleanup_survives_actual_final_poll_stop() {
    with_cleanup(false, |runtime, cleanup, probe| {
        let memory = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
        let cancel = CancellationSource::new();
        let host = SerialRequest::from_memory(memory.clone(), cancel.token(), None);
        let token = cancel.clone();
        runtime
            .owner_cleanup
            .replace_after_component_progress_hook(Some(Box::new(move || token.cancel())));
        let _hook = Hook(runtime.owner_cleanup.clone());
        let mut retirements = OwnerCleanupRetirements::default();
        let outer = probe.outer.lock().unwrap();
        let completed = cleanup
            .retry_in_request_with_retirements(&ExecutionRequest::serial(&host), &mut retirements)
            .expect("actual completion cannot become an unavailable retry handle after final poll");
        assert!(cancel.token().is_cancelled());
        assert_eq!(completed.retired_component_count(), 1);
        assert!(runtime.pending_owner_cleanup().is_empty());
        assert_eq!(probe.notifications.load(Ordering::SeqCst), 0);
        drop(outer);
        drop(retirements);
        assert_eq!(probe.notifications.load(Ordering::SeqCst), 1);
        assert_eq!(memory.charged_memory_bytes(), 0);
    });
}
