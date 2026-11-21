//! A fresh owner receipt is the observation source for commits and product publications.
use super::{
    WorthQueryAdvancementEventForTest, WorthQueryAdvancementEventKindForTest as Kind,
    WorthQueryAdvancementIdentityForTest as Identity,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
thread_local! {
    static MEMBER: RefCell<Option<Identity>> = const { RefCell::new(None) };
    static CALLER_PASSES: RefCell<Vec<(Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>, u64)>> =
        const { RefCell::new(Vec::new()) };
    static CALLER_PASS_CLEANUP: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn clear_caller_passes() {
    CALLER_PASS_CLEANUP.with(|registered| registered.set(false));
    CALLER_PASSES.with(|records| records.borrow_mut().clear());
}

pub fn record_caller_pass_for_test<R>(
    result: &Result<R, crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial>,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CALLER_PASS_CLEANUP.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_caller_passes);
    }
    CALLER_PASSES.with(|passes| {
        passes.borrow_mut().push((
            result.as_ref().map(|_| ()).map_err(|denial| denial.kind()),
            worth_signal::facade::observed_signal_request_work_on_this_thread_for_test(),
        ))
    });
}

pub fn caller_pass_reports_on_this_thread_for_test(
) -> Vec<(
    Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind>,
    u64,
)> {
    CALLER_PASSES.with(|records| std::mem::take(&mut *records.borrow_mut()))
}
pub struct AppliedIdentity {
    previous: Option<Identity>,
    owner_thread: PhantomData<Rc<()>>,
}
pub fn apply_identity(
    identity: Identity,
) -> AppliedIdentity {
    AppliedIdentity {
        previous: MEMBER.with(|member| member.replace(Some(identity))),
        owner_thread: PhantomData,
    }
}
impl Drop for AppliedIdentity {
    fn drop(&mut self) {
        MEMBER.with(|member| {
            member.replace(self.previous.take());
        });
    }
}
pub fn record_commit(
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    let identity = MEMBER
        .with(|member| member.borrow().clone())
        .unwrap_or_else(|| {
            let binding = receipt.idempotency_binding();
            Identity::Mutation {
                operation: *receipt.installed_operation(),
                key: *binding.key_identity(),
                intent: *binding.intent_identity(),
            }
        });
    super::record_event(WorthQueryAdvancementEventForTest {
        kind: Kind::Commit,
        identity: identity.clone(),
    });
    // A fresh application's closed receipt certifies its landed product publication.
    super::record_event(WorthQueryAdvancementEventForTest {
        kind: Kind::Publication,
        identity,
    });
}
