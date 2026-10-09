use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::{
    WorthQueryProviderExecutionPlanContract, WorthQueryProviderSessionAffinityIdentity,
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
};

static NEXT_PROTOCOL_SESSION: AtomicU64 = AtomicU64::new(1);

pub struct WorthQueryProviderSessionTokenAdmission {
    plan_identity: Arc<str>,
    provider_identity: Arc<str>,
    provider_generation: u64,
}

impl WorthQueryProviderSessionTokenAdmission {
    pub(super) fn new(plan: &WorthQueryProviderExecutionPlanContract) -> Self {
        Self {
            plan_identity: plan.identity().into(),
            provider_identity: plan.provider_identity().into(),
            provider_generation: plan.provider_generation(),
        }
    }

    pub fn admit(
        self,
        physical_session_identity: impl Into<String>,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        let physical_session_identity = physical_session_identity.into();
        if physical_session_identity.trim().is_empty()
            || physical_session_identity.trim() != physical_session_identity
        {
            return Err(WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::EmptyPhysicalSessionIdentity,
                WorthQueryProviderSessionProtocolStage::PlanReadmission,
                "provider physical session identity must be non-empty and canonical",
                WorthQueryProviderSessionProtocolCounters::default(),
            ));
        }
        let generation =
            next_protocol_session_generation(&NEXT_PROTOCOL_SESSION).ok_or_else(|| {
                WorthQueryProviderSessionFailure::new(
                    WorthQueryProviderSessionDenialKind::SessionIdentityExhausted,
                    WorthQueryProviderSessionProtocolStage::PlanReadmission,
                    "provider session identity space is exhausted",
                    WorthQueryProviderSessionProtocolCounters::default(),
                )
            })?;
        Ok(WorthQueryProviderSessionToken {
            identity: Arc::from(format!("provider-session:{generation}")),
            plan_identity: self.plan_identity,
            provider_identity: self.provider_identity,
            provider_generation: self.provider_generation,
            generation,
            physical_session_identity: physical_session_identity.into(),
        })
    }
}

fn next_protocol_session_generation(counter: &AtomicU64) -> Option<u64> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_session_generation_exhaustion_cannot_wrap() {
        let counter = AtomicU64::new(u64::MAX - 1);

        assert_eq!(
            next_protocol_session_generation(&counter),
            Some(u64::MAX - 1)
        );
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
        assert_eq!(next_protocol_session_generation(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), u64::MAX);
    }
}

pub struct WorthQueryProviderSessionToken {
    identity: Arc<str>,
    plan_identity: Arc<str>,
    provider_identity: Arc<str>,
    provider_generation: u64,
    generation: u64,
    physical_session_identity: Arc<str>,
}

impl std::fmt::Debug for WorthQueryProviderSessionToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProviderSessionToken")
            .field("identity", &self.identity)
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

impl WorthQueryProviderSessionToken {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn belongs_to(&self, plan: &WorthQueryProviderExecutionPlanContract) -> bool {
        self.plan_identity.as_ref() == plan.identity()
            && self.provider_identity.as_ref() == plan.provider_identity()
            && self.provider_generation == plan.provider_generation()
    }

    pub(super) fn view<'scope>(
        &'scope self,
        request: worth_execution::ExecutionRequest<'scope, 'scope>,
    ) -> WorthQueryProviderSessionView<'scope> {
        WorthQueryProviderSessionView {
            token: self,
            request,
        }
    }
}

#[derive(Clone, Copy)]
pub struct WorthQueryProviderSessionView<'session> {
    token: &'session WorthQueryProviderSessionToken,
    request: worth_execution::ExecutionRequest<'session, 'session>,
}

impl<'session> WorthQueryProviderSessionView<'session> {
    /// The admitted request that owns this provider call.
    pub fn execution_request(self) -> worth_execution::ExecutionRequest<'session, 'session> {
        self.request
    }

    #[allow(dead_code)]
    pub(in crate::domain_computation) fn affinity_identity(
        self,
    ) -> WorthQueryProviderSessionAffinityIdentity {
        WorthQueryProviderSessionAffinityIdentity::from_token(self.token)
    }

    pub fn identity(&self) -> &str {
        self.token.identity()
    }

    pub fn generation(&self) -> u64 {
        self.token.generation()
    }

    pub fn provider_identity(&self) -> &str {
        &self.token.provider_identity
    }

    pub(in crate::domain_computation) fn plan_identity(&self) -> &str {
        &self.token.plan_identity
    }

    pub fn provider_generation(&self) -> u64 {
        self.token.provider_generation
    }

    pub fn physical_session_identity(&self) -> &str {
        &self.token.physical_session_identity
    }
}

#[derive(Clone, Copy)]
pub struct WorthQueryProviderExecutionPlanView<'plan> {
    request: worth_execution::ExecutionRequest<'plan, 'plan>,
    contract: &'plan WorthQueryProviderExecutionPlanContract,
}

impl<'plan> WorthQueryProviderExecutionPlanView<'plan> {
    pub(super) fn new(
        request: worth_execution::ExecutionRequest<'plan, 'plan>,
        contract: &'plan WorthQueryProviderExecutionPlanContract,
    ) -> Self {
        Self { request, contract }
    }

    pub fn execution_request(self) -> worth_execution::ExecutionRequest<'plan, 'plan> {
        self.request
    }

    pub fn contract(self) -> &'plan WorthQueryProviderExecutionPlanContract {
        self.contract
    }
}
