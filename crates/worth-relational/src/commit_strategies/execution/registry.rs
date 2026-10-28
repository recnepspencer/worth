use std::collections::BTreeMap;
use std::sync::Arc;

use crate::commit_strategies::data::{
    CommitStrategyDescriptorDigest, CommitStrategyExecutionRegistration, CommitStrategyExecutor,
    CommitStrategyId,
};

use super::super::FrozenCommitStrategyRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitStrategyExecutionRegistryError {
    DuplicateBinding {
        strategy_id: CommitStrategyId,
    },
    MissingDescriptorRegistration {
        strategy_id: CommitStrategyId,
        descriptor_digest: CommitStrategyDescriptorDigest,
    },
    DescriptorDigestMismatch {
        strategy_id: CommitStrategyId,
        descriptor_digest: CommitStrategyDescriptorDigest,
        registered_digest: CommitStrategyDescriptorDigest,
    },
}

#[derive(Clone)]
pub(super) struct RegisteredCommitStrategyExecutor {
    pub(super) descriptor_digest: CommitStrategyDescriptorDigest,
    pub(super) executor: Arc<dyn CommitStrategyExecutor>,
}

#[derive(Clone, Default)]
pub(crate) struct FrozenCommitStrategyExecutorRegistry {
    executors_by_id: Arc<BTreeMap<CommitStrategyId, RegisteredCommitStrategyExecutor>>,
}

impl std::fmt::Debug for FrozenCommitStrategyExecutorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrozenCommitStrategyExecutorRegistry")
            .field("bound_executor_count", &self.executors_by_id.len())
            .finish()
    }
}

impl FrozenCommitStrategyExecutorRegistry {
    pub(crate) fn from_registrations(
        bindings: Vec<CommitStrategyExecutionRegistration>,
        descriptor_registry: &FrozenCommitStrategyRegistry,
    ) -> Result<Self, CommitStrategyExecutionRegistryError> {
        let mut executors_by_id = BTreeMap::new();
        for binding in bindings {
            register_executor_binding(&mut executors_by_id, binding, descriptor_registry)?;
        }
        Ok(Self {
            executors_by_id: Arc::new(executors_by_id),
        })
    }

    pub(super) fn get(
        &self,
        strategy_id: CommitStrategyId,
    ) -> Option<&RegisteredCommitStrategyExecutor> {
        self.executors_by_id.get(&strategy_id)
    }
}

fn register_executor_binding(
    executors_by_id: &mut BTreeMap<CommitStrategyId, RegisteredCommitStrategyExecutor>,
    binding: CommitStrategyExecutionRegistration,
    descriptor_registry: &FrozenCommitStrategyRegistry,
) -> Result<(), CommitStrategyExecutionRegistryError> {
    reject_duplicate_executor_binding(executors_by_id, binding.strategy_id())?;
    let registered_digest = registered_descriptor_digest(&binding, descriptor_registry)?;
    reject_descriptor_digest_mismatch(&binding, registered_digest)?;
    executors_by_id.insert(
        binding.strategy_id(),
        RegisteredCommitStrategyExecutor {
            descriptor_digest: binding.descriptor_digest(),
            executor: binding.executor(),
        },
    );
    Ok(())
}

fn reject_duplicate_executor_binding(
    executors_by_id: &BTreeMap<CommitStrategyId, RegisteredCommitStrategyExecutor>,
    strategy_id: CommitStrategyId,
) -> Result<(), CommitStrategyExecutionRegistryError> {
    if executors_by_id.contains_key(&strategy_id) {
        return Err(CommitStrategyExecutionRegistryError::DuplicateBinding { strategy_id });
    }
    Ok(())
}

fn registered_descriptor_digest(
    binding: &CommitStrategyExecutionRegistration,
    descriptor_registry: &FrozenCommitStrategyRegistry,
) -> Result<CommitStrategyDescriptorDigest, CommitStrategyExecutionRegistryError> {
    descriptor_registry
        .iter()
        .find(|registration| registration.descriptor().id() == binding.strategy_id())
        .map(|registration| registration.descriptor().digest())
        .ok_or(
            CommitStrategyExecutionRegistryError::MissingDescriptorRegistration {
                strategy_id: binding.strategy_id(),
                descriptor_digest: binding.descriptor_digest(),
            },
        )
}

fn reject_descriptor_digest_mismatch(
    binding: &CommitStrategyExecutionRegistration,
    registered_digest: CommitStrategyDescriptorDigest,
) -> Result<(), CommitStrategyExecutionRegistryError> {
    if registered_digest != binding.descriptor_digest() {
        return Err(
            CommitStrategyExecutionRegistryError::DescriptorDigestMismatch {
                strategy_id: binding.strategy_id(),
                descriptor_digest: binding.descriptor_digest(),
                registered_digest,
            },
        );
    }
    Ok(())
}
