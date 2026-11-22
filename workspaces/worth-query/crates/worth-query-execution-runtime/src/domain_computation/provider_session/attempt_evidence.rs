use std::sync::Arc;

use worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan;

use super::WorthQueryExecutionProviderSession;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExecutionResourceAttemptEvidence {
    identity: Arc<str>,
    admission_identity: Arc<str>,
    request_identity: Arc<str>,
    strategy: Arc<str>,
    envelope_identity: Arc<str>,
    support_snapshot_identity: Arc<str>,
    provider_session_identity: Arc<str>,
    provider_session_attempt_identity: Arc<str>,
}

impl WorthQueryExecutionResourceAttemptEvidence {
    pub(super) fn capture(
        plan: &WorthQueryAdmittedExecutionResourcePlan,
        session: &WorthQueryExecutionProviderSession,
    ) -> Self {
        let identity = Arc::<str>::from(session.identity());
        Self {
            identity,
            admission_identity: Arc::from(plan.identity()),
            request_identity: Arc::from(plan.request_identity()),
            strategy: Arc::from(plan.strategy().as_str()),
            envelope_identity: Arc::from(plan.envelope_identity()),
            support_snapshot_identity: Arc::from(plan.support_snapshot().identity()),
            provider_session_identity: Arc::from(session.identity()),
            provider_session_attempt_identity: Arc::from(session.attempt_identity()),
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn admission_identity(&self) -> &str {
        &self.admission_identity
    }

    pub fn request_identity(&self) -> &str {
        &self.request_identity
    }

    pub fn strategy(&self) -> &str {
        &self.strategy
    }

    pub fn envelope_identity(&self) -> &str {
        &self.envelope_identity
    }

    pub fn support_snapshot_identity(&self) -> &str {
        &self.support_snapshot_identity
    }

    pub fn provider_session_identity(&self) -> &str {
        &self.provider_session_identity
    }

    pub fn provider_session_attempt_identity(&self) -> &str {
        &self.provider_session_attempt_identity
    }
}
