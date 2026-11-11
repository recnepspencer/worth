use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQuerySharedLiveProjectionLease;

/// Caller-declared delivery policy. This describes what one consumer requests;
/// it is not authority until the current Query runtime admits it for a live
/// shared lease.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySharedConsumerDeliveryPolicy {
    purpose: String,
    disclosure: String,
    continuation: String,
    backpressure: crate::subscription::DeliveryBackpressurePolicy,
}

impl WorthQuerySharedConsumerDeliveryPolicy {
    pub fn new(
        purpose: impl Into<String>,
        disclosure: impl Into<String>,
        continuation: impl Into<String>,
        backpressure: crate::subscription::DeliveryBackpressurePolicy,
    ) -> Option<Self> {
        let purpose = purpose.into();
        let disclosure = disclosure.into();
        let continuation = continuation.into();
        if purpose.trim().is_empty()
            || disclosure.trim().is_empty()
            || continuation.trim().is_empty()
        {
            return None;
        }
        Some(Self {
            purpose,
            disclosure,
            continuation,
            backpressure,
        })
    }

    pub fn purpose(&self) -> &str {
        &self.purpose
    }

    pub fn disclosure(&self) -> &str {
        &self.disclosure
    }

    pub fn continuation(&self) -> &str {
        &self.continuation
    }

    pub const fn backpressure(&self) -> crate::subscription::DeliveryBackpressurePolicy {
        self.backpressure
    }
}

/// Query-owned evidence that a delivery policy was admitted for one current
/// shared consumer lease.
pub struct WorthQuerySharedConsumerDeliveryPolicyAdmission {
    lease: crate::runtime::WorthQuerySharedProjectionLeaseIdentity,
    policy_generation: u64,
    identity: String,
}

impl WorthQuerySharedConsumerDeliveryPolicyAdmission {
    pub const fn lease_identity(&self) -> crate::runtime::WorthQuerySharedProjectionLeaseIdentity {
        self.lease
    }

    pub const fn policy_generation(&self) -> u64 {
        self.policy_generation
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQuerySharedLiveProjectionLease<D, O, F, L>
{
    pub fn admit_consumer_delivery_policy(
        &self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
        policy: WorthQuerySharedConsumerDeliveryPolicy,
    ) -> Result<
        WorthQuerySharedConsumerDeliveryPolicyAdmission,
        crate::runtime::WorthQueryRuntimeError,
    > {
        let policy_generation = workspace.admit_shared_consumer_delivery_policy(
            self.workspace_capability(),
            self.readmission(),
            policy,
        )?;
        Ok(WorthQuerySharedConsumerDeliveryPolicyAdmission {
            lease: self.lease_identity(),
            policy_generation,
            identity: crate::identity::hash_parts(&[
                "worth_query_shared_consumer_delivery_policy_admission_v1".into(),
                format!("owner:{}", self.owner_identity().slot()),
                format!("lease:{}", self.lease_identity().slot()),
                format!("policy-generation:{policy_generation}"),
            ]),
        })
    }
}
#[cfg(test)]
#[path = "consumer_policy_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
