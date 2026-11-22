//! Exact transient registration associated with a captured primary read set.
use super::*;

#[derive(Clone)]
pub struct RegisteredPrimaryRead {
    identity: u64,
    facts: Arc<crate::domain_computation::primary_graph::ApplicationFactTable>,
}
impl RegisteredPrimaryRead {
    pub fn facts(&self) -> &Arc<crate::domain_computation::primary_graph::ApplicationFactTable> {
        &self.facts
    }
}
impl WorthQueryPrimaryGraphApplicationAttemptStore {
    pub fn registered_read(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> Option<RegisteredPrimaryRead> {
        let key = WorthQueryApplicationAttemptLookupKey::from_identity(session.affinity_identity());
        let WorthQueryApplicationAttemptEntry::Registered { identity, state } =
            self.attempts.get(&key)?
        else {
            return None;
        };
        let attempt = state.attempt();
        attempt
            .affinity()
            .admits_session(session)
            .then(|| RegisteredPrimaryRead {
                identity: *identity,
                facts: attempt.shared_facts(),
            })
    }
    pub fn retains_registered_read(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        captured: &RegisteredPrimaryRead,
    ) -> bool {
        self.registered_read(session).is_some_and(|current| {
            current.identity == captured.identity && Arc::ptr_eq(&current.facts, &captured.facts)
        })
    }
}
