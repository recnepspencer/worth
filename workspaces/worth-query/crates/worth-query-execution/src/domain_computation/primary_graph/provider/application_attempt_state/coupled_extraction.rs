//! One attempt-store transition transfers all genuinely prepared writers.
use super::commit_completion::WorthQueryApplicationAttemptCompletion;
use super::coupled_preparation::OwnedCoupledContribution;
use super::*;

impl WorthQueryPrimaryGraphApplicationAttemptStore {
    pub(super) fn take_commit_prepared(
        &mut self,
        session: WorthQueryProviderSessionView<'_>,
        attempts: Arc<Mutex<Self>>,
    ) -> Option<WorthQueryPreparedProviderApplicationAttempt> {
        let key = WorthQueryApplicationAttemptLookupKey::from_identity(session.affinity_identity());
        let members = self.attempt_state(session)?.coupled_members()?;
        // All checks precede every transfer. Existing descriptor storage carries
        // the extracted custody, so the transition itself cannot allocate.
        for (ordinal, member) in members.iter().enumerate() {
            let member_key = WorthQueryApplicationAttemptLookupKey::from_identity(
                member.terminal.affinity_identity(),
            );
            if member_key == key
                || members[..ordinal]
                    .iter()
                    .any(|prior| prior.terminal.same_session(&member.terminal))
            {
                return None;
            }
            let Some(WorthQueryApplicationAttemptEntry::Registered { state, .. }) =
                self.attempts.get(&member_key)
            else {
                return None;
            };
            if !state.is_coupled_ready()
                || !state.attempt().affinity().same_session(&member.terminal)
            {
                return None;
            }
        }
        let (identity, state) = self.take_registered(key);
        let (attempt, mut approved) = state
            .take_commit_parts()
            .expect("root readiness checked before transfer");
        for member in &mut approved.members {
            let member_key = WorthQueryApplicationAttemptLookupKey::from_identity(
                member.terminal.affinity_identity(),
            );
            let (identity, state) = self.take_registered(member_key);
            let (attempt, contribution) = state.take_coupled_parts();
            member.owned = Some(OwnedCoupledContribution {
                attempt,
                work: contribution.work,
                commit: None,
                _completion: WorthQueryApplicationAttemptCompletion::new(
                    Arc::clone(&attempts),
                    member_key,
                    identity,
                ),
            });
        }
        Some(WorthQueryPreparedProviderApplicationAttempt::new(
            attempt, approved, attempts, key, identity,
        ))
    }

    fn take_registered(
        &mut self,
        key: WorthQueryApplicationAttemptLookupKey,
    ) -> (u64, WorthQueryApplicationAttemptState) {
        let slot = self
            .attempts
            .get_mut(&key)
            .expect("prechecked registered attempt");
        let WorthQueryApplicationAttemptEntry::Registered { identity, .. } = slot else {
            unreachable!("prechecked registration")
        };
        let identity = *identity;
        let WorthQueryApplicationAttemptEntry::Registered { state, .. } = std::mem::replace(
            slot,
            WorthQueryApplicationAttemptEntry::Committing { identity },
        ) else {
            unreachable!()
        };
        (identity, state)
    }
}
