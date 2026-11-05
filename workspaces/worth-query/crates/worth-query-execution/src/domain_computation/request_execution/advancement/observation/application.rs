//! A fresh owner receipt is the observation source for commits and product publications.
use super::{
    WorthQueryAdvancementEventForTest, WorthQueryAdvancementEventKindForTest as Kind,
    WorthQueryAdvancementIdentityForTest as Identity,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
thread_local! {
    static MEMBER: RefCell<Option<Identity>> = const { RefCell::new(None) };
}
pub(in crate::domain_computation) struct AppliedIdentity {
    previous: Option<Identity>,
    owner_thread: PhantomData<Rc<()>>,
}
pub(in crate::domain_computation) fn apply_identity(
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
/// The mutation identity a committed application reports when no member identity is applied,
/// so the advancement observer never names the commit receipt.
pub(in crate::domain_computation) trait WorthQueryObservedCommit {
    fn observed_operation(&self) -> [u8; 32];
    fn observed_idempotency_key(&self) -> [u8; 32];
    fn observed_idempotency_intent(&self) -> [u8; 32];
}

pub(in crate::domain_computation) fn record_commit<
    Commit: WorthQueryObservedCommit + ?Sized,
>(
    receipt: &Commit,
) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    let identity = MEMBER
        .with(|member| member.borrow().clone())
        .unwrap_or_else(|| Identity::Mutation {
            operation: receipt.observed_operation(),
            key: receipt.observed_idempotency_key(),
            intent: receipt.observed_idempotency_intent(),
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
