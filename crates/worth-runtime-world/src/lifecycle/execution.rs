//! How a World's requests run. A host installs a request policy, and may
//! install its process authority beside it; an authority alone has no
//! request shape to lease. Typestate requires the policy; the builder refuses
//! a policy the authority can never lease.

use std::sync::Arc;

use worth_execution::{ExecutionAuthority, ExecutionPolicyDenial};
use worth_foundational::ExecutionRequestPolicy;

use crate::identity::RuntimeWorldIdentityExhaustion;

/// The installed form, held by the owner state.
#[derive(Debug)]
pub(crate) enum InstalledExecution {
    /// Requests run on the calling thread within the policy's memory limit.
    Serial(ExecutionRequestPolicy),
    /// Requests lease the policy's budget from the host's authority.
    Leased {
        authority: Arc<ExecutionAuthority>,
        policy: ExecutionRequestPolicy,
    },
}

/// How this World's requests run, borrowed from its owner.
#[derive(Clone, Copy, Debug)]
pub enum RuntimeWorldExecutionPlacement<'owner> {
    /// Requests run on the calling thread within the policy's memory limit.
    Serial(ExecutionRequestPolicy),
    /// Requests lease the policy's budget from the host's authority.
    Leased {
        authority: &'owner ExecutionAuthority,
        policy: ExecutionRequestPolicy,
    },
}

/// Why the builder produced no World.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeWorldBuildDenial {
    IdentityExhaustion(RuntimeWorldIdentityExhaustion),
    /// The policy asks for more workers than the authority admits.
    ExecutionPolicyWorkersExceedAuthority,
    /// The policy asks for more memory than the authority admits.
    ExecutionPolicyMemoryExceedsAuthority,
    /// The policy's equivalence contract is not registered with the
    /// authority.
    ExecutionPolicyContractUnavailable,
}

impl InstalledExecution {
    pub(crate) fn try_new(
        authority: Option<Arc<ExecutionAuthority>>,
        policy: ExecutionRequestPolicy,
    ) -> Result<Self, RuntimeWorldBuildDenial> {
        match authority {
            None => Ok(Self::Serial(policy)),
            Some(authority) => match authority.admits_policy(&policy) {
                Ok(()) => Ok(Self::Leased { authority, policy }),
                Err(denial) => Err(policy_denial(denial)),
            },
        }
    }

    pub(crate) fn request_policy(&self) -> ExecutionRequestPolicy {
        match self {
            Self::Serial(policy) | Self::Leased { policy, .. } => *policy,
        }
    }

    pub(crate) fn placement(&self) -> RuntimeWorldExecutionPlacement<'_> {
        match self {
            Self::Serial(policy) => RuntimeWorldExecutionPlacement::Serial(*policy),
            Self::Leased { authority, policy } => RuntimeWorldExecutionPlacement::Leased {
                authority,
                policy: *policy,
            },
        }
    }
}

fn policy_denial(denial: ExecutionPolicyDenial) -> RuntimeWorldBuildDenial {
    match denial {
        ExecutionPolicyDenial::WorkersExceedAuthority => {
            RuntimeWorldBuildDenial::ExecutionPolicyWorkersExceedAuthority
        }
        ExecutionPolicyDenial::MemoryExceedsAuthority => {
            RuntimeWorldBuildDenial::ExecutionPolicyMemoryExceedsAuthority
        }
        ExecutionPolicyDenial::ContractUnavailable => {
            RuntimeWorldBuildDenial::ExecutionPolicyContractUnavailable
        }
    }
}
