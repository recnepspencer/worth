//! A fresh owner receipt is the observation source for commits and product publications.
use super::{
    WorthQueryAdvancementEvent, WorthQueryAdvancementEventKind as Kind,
    WorthQueryAdvancementIdentity as Identity,
};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};
thread_local! {
    static MEMBER: RefCell<Option<Identity>> = const { RefCell::new(None) };
}
pub(in crate::domain_computation::primary_graph) struct AppliedIdentity {
    previous: Option<Identity>,
    owner_thread: PhantomData<Rc<()>>,
}
pub(in crate::domain_computation::primary_graph) fn apply_identity(
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
pub(in crate::domain_computation::primary_graph) fn record_commit(
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
) {
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
    super::record_event(WorthQueryAdvancementEvent {
        kind: Kind::Commit,
        identity: identity.clone(),
    });
    // A fresh application's closed receipt certifies its landed product publication.
    super::record_event(WorthQueryAdvancementEvent {
        kind: Kind::Publication,
        identity,
    });
}
