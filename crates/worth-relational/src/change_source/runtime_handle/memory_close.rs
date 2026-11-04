//! Request-bounded business retirement; allocation destruction is separate.
use super::{RelationalRuntimeHandle, RuntimeOwnership};
use crate::runtime::{RelationalRuntime, RelationalRuntimeCloseDenial};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalMemoryCloseDenial {
    SharedRepresentation,
    SharedOwner,
    SourceAffinity,
    Native(RelationalRuntimeCloseDenial),
}

/// Entry refusal preserves the exact untouched handle and its metadata.
#[derive(Debug)]
pub struct RelationalMemoryCloseRefused {
    handle: RelationalRuntimeHandle,
    reason: RelationalMemoryCloseDenial,
}
impl RelationalMemoryCloseRefused {
    pub fn reason(&self) -> RelationalMemoryCloseDenial {
        self.reason
    }
    pub fn into_handle(self) -> RelationalRuntimeHandle {
        self.handle
    }
}

#[derive(Debug)]
pub enum RelationalMemoryCloseObservation {
    Advanced,
    Busy,
    Stopped(worth_execution::WorkCeilingDenial),
    Refused(RelationalRuntimeCloseDenial),
    Complete,
}

/// Owns the authentic runtime across every stop and refusal. Keep this phase
/// outside disposable callbacks. Generic final Drop is UNBUDGETED abandonment:
/// it invokes the existing owner-loss destructor and produces no close proof.
/// Only `advance` performs request-bounded, nonwaiting owner steps.
///
/// ```compile_fail
/// use worth_relational::facade::change_source::RelationalMemoryClose;
/// fn no_runtime_escape(close: RelationalMemoryClose) { close.runtime(); }
/// ```
#[must_use]
pub struct RelationalMemoryClose {
    runtime: RelationalRuntime,
    runtime_instance_id: u64,
    lock_observation: Arc<super::lock_observation::LockObservationSlot>,
    sealed: bool,
    complete: bool,
    cache_cursor: usize,
    cache_complete: bool,
    head_cursor: usize,
}

/// Nonserving authentic owner allocation after ALL business custody retired.
/// It has no runtime, recovery, mutation or serving-resume accessor. Its final
/// Drop/reclaim is explicitly UNBUDGETED allocation and user-payload destruction.
///
/// ```compile_fail
/// use worth_relational::facade::change_source::RetiredMemoryRelationalAllocation;
/// fn cannot_resume(value: RetiredMemoryRelationalAllocation) { value.resume(); }
/// ```
#[must_use]
pub struct RetiredMemoryRelationalAllocation {
    _runtime: RelationalRuntime,
    _lock_observation: Arc<super::lock_observation::LockObservationSlot>,
}
impl RetiredMemoryRelationalAllocation {
    pub fn reclaim(self) {
        drop(self);
    }
}

impl RelationalRuntimeHandle {
    pub fn into_memory_close(self) -> Result<RelationalMemoryClose, RelationalMemoryCloseRefused> {
        let reason = match &self.ownership {
            RuntimeOwnership::Shared(_) => Some(RelationalMemoryCloseDenial::SharedRepresentation),
            RuntimeOwnership::Immutable(runtime) if !runtime.is_owner() => Some(
                RelationalMemoryCloseDenial::Native(RelationalRuntimeCloseDenial::NotOwner),
            ),
            RuntimeOwnership::Immutable(runtime) if runtime.has_memory_close_source_affinity() => {
                Some(RelationalMemoryCloseDenial::SourceAffinity)
            }
            _ => None,
        };
        if let Some(reason) = reason {
            return Err(RelationalMemoryCloseRefused {
                handle: self,
                reason,
            });
        }
        let Self {
            ownership,
            runtime_instance_id,
            lock_observation,
        } = self;
        let RuntimeOwnership::Immutable(runtime) = ownership else {
            unreachable!(
                /* Proof: The Shared representation returned unchanged before destructuring this handle. */
                "memory close accepts only immutable ownership"
            )
        };
        match Arc::try_unwrap(runtime) {
            Ok(runtime) => Ok(RelationalMemoryClose {
                runtime,
                runtime_instance_id,
                lock_observation,
                sealed: false,
                complete: false,
                cache_cursor: 0,
                cache_complete: false,
                head_cursor: 0,
            }),
            Err(runtime) => Err(RelationalMemoryCloseRefused {
                handle: Self {
                    ownership: RuntimeOwnership::Immutable(runtime),
                    runtime_instance_id,
                    lock_observation,
                },
                reason: RelationalMemoryCloseDenial::SharedOwner,
            }),
        }
    }
}

impl RelationalMemoryClose {
    /// One original-request step. Never allocates, reserves, rehashes, shrinks,
    /// waits for a lock, or destroys the final Runtime/cached-state payload.
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> RelationalMemoryCloseObservation {
        use RelationalMemoryCloseObservation as O;
        if self.complete {
            return O::Complete;
        }
        if let Err(cause) = request.consult() {
            return O::Stopped(cause);
        }
        if !self.sealed {
            return match self.runtime.try_prepare_stopped_close() {
                Err(()) => O::Busy,
                Ok(Some(reason)) => O::Refused(reason),
                Ok(None) => {
                    self.sealed = true;
                    O::Advanced
                }
            };
        }
        if !self.cache_complete {
            return match self
                .runtime
                .visibility
                .cache
                .try_retire_memory_state(self.cache_cursor)
            {
                Err(()) => O::Busy,
                Ok(None) => {
                    self.cache_complete = true;
                    O::Advanced
                }
                Ok(Some(done)) => {
                    if done {
                        self.cache_cursor += 1;
                    }
                    O::Advanced
                }
            };
        }
        if self.runtime.history.has_detached_retention_obligations() {
            return O::Refused(RelationalRuntimeCloseDenial::ReadCustodyRetained);
        }
        match self
            .runtime
            .history
            .try_retire_stopped_head(self.head_cursor)
        {
            Err(()) => return O::Busy,
            Ok(None) => return O::Refused(RelationalRuntimeCloseDenial::ReadCustodyRetained),
            Ok(Some(false)) => {
                self.head_cursor += 1;
                return O::Advanced;
            }
            Ok(Some(true)) => {}
        }
        match self.runtime.complete_stopped_close_business() {
            Err(()) => O::Busy,
            Ok(Some(reason)) => O::Refused(reason),
            Ok(None) => {
                self.complete = true;
                O::Complete
            }
        }
    }

    /// Explicitly UNBUDGETED restoration before any irreversible cache step.
    /// Reallocates an Arc around the SAME runtime; preserves its instance and
    /// original lock-observation binding, not the old Arc allocation identity.
    pub fn into_refused_handle(self) -> Result<RelationalRuntimeHandle, Self> {
        if self.sealed {
            return Err(self);
        }
        Ok(RelationalRuntimeHandle {
            ownership: RuntimeOwnership::Immutable(Arc::new(self.runtime)),
            runtime_instance_id: self.runtime_instance_id,
            lock_observation: self.lock_observation,
        })
    }

    /// Continue ONLY an actual original-result task; no raw owner escapes.
    pub fn advance_commit_result_snapshot_cleanup(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        task: crate::branch::RelationalCommitResultSnapshotCleanup,
    ) -> crate::branch::RelationalCommitResultSnapshotCleanupProgress {
        task.advance(request, &self.runtime)
    }

    pub fn into_retired_memory(self) -> Result<RetiredMemoryRelationalAllocation, Self> {
        if !self.complete {
            return Err(self);
        }
        Ok(RetiredMemoryRelationalAllocation {
            _runtime: self.runtime,
            _lock_observation: self.lock_observation,
        })
    }
}

impl std::fmt::Debug for RelationalMemoryClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalMemoryClose")
            .field("runtime_instance_id", &self.runtime_instance_id)
            .field("sealed", &self.sealed)
            .field("complete", &self.complete)
            .field("cache_cursor", &self.cache_cursor)
            .field("head_cursor", &self.head_cursor)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for RetiredMemoryRelationalAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetiredMemoryRelationalAllocation")
            .finish_non_exhaustive()
    }
}
