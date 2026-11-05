//! Consuming retirement of the original registered source and its local aliases.
use super::WorthQueryRelationalSourceOwner;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
};
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{change_source::*, runtime::RelationalRuntime};
use worth_runtime_bridge::facade::*;

pub(in crate::domain_computation) struct RetiringRelationalSource {
    runtime: Option<RelationalRuntimeHandle>,
    source: Option<RuntimeBridgeRelationalSource>,
    head_shares: Option<Arc<Mutex<Option<RelationalBridgeBranchHeadLease>>>>,
    head_slot: Option<Mutex<Option<RelationalBridgeBranchHeadLease>>>,
    head: Option<RelationalBridgeBranchHeadRetirement>,
    retired_head: Option<RetiredRelationalBridgeBranchHead>,
    subscription: Option<RelationalBridgeCanonicalSubscription>,
    retired_subscription: Option<RetiredRelationalBridgeCanonicalSubscription>,
    retired_runtime: Option<RetiredRelationalRuntimeHandleAlias>,
    retired_source: Option<RetiredRuntimeBridgeRelationalSource>,
    _allocation: SourceOwnerAllocation,
}

struct SourceOwnerAllocation {
    _runtime_instance_id: u64,
    _invalidation: Arc<crate::domain_computation::primary_graph::SourceInvalidationOwner>,
    _reads: Arc<std::sync::atomic::AtomicU64>,
    _observers: Arc<Mutex<Vec<RelationalRuntimeLockObservationForTest>>>,
}

impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        self.runtime.with_runtime(|owner| {
            self.runtime
                .validate_immutable_owner_alias(owner)
                .map_err(Denial::NativeAlias)?;
            self.source
                .preflight_alias_retirement(owner)
                .map_err(Denial::NativeAlias)
        })
    }

    pub(in crate::domain_computation) fn into_home_retirement(self) -> RetiringRelationalSource {
        let Self {
            runtime,
            runtime_instance_id,
            source,
            bridge_head,
            invalidation_owner,
            canonical_subscription,
            read_acquisitions,
            _bridge_lock_observations,
        } = self;
        RetiringRelationalSource {
            runtime: Some(runtime),
            source: Some(source),
            head_shares: Some(bridge_head),
            head_slot: None,
            head: None,
            retired_head: None,
            subscription: Some(canonical_subscription),
            retired_subscription: None,
            retired_runtime: None,
            retired_source: None,
            _allocation: SourceOwnerAllocation {
                _runtime_instance_id: runtime_instance_id,
                _invalidation: invalidation_owner,
                _reads: read_acquisitions,
                _observers: _bridge_lock_observations,
            },
        }
    }
}

impl RetiringRelationalSource {
    pub(in crate::domain_computation) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        Denial,
    > {
        use crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress as Step;
        if self.retired_source.is_some() {
            return Ok(Step::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        let source = self
            .source
            .as_ref()
            .expect("source remains until its final alias step");
        source
            .preflight_alias_retirement(owner)
            .map_err(Denial::NativeAlias)?;
        if let Some(shares) = self.head_shares.take() {
            // This is atomic local-share release, not a reference-count probe.
            // Only the last original share owns the head and subscription detach.
            self.head_slot = Arc::into_inner(shares);
            return Ok(Step::Advanced);
        }
        if let Some(slot) = self.head_slot.as_mut() {
            if let Some(head) = slot
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                self.head = Some(head.into_retirement());
                return Ok(Step::Advanced);
            }
            if let Some(head) = self.head.as_mut() {
                match head.advance(request, source) {
                    RelationalBridgeBasisRetirementProgress::Advanced => return Ok(Step::Advanced),
                    RelationalBridgeBasisRetirementProgress::Busy => {
                        return Err(Denial::Busy(Owner::RelationalSource))
                    }
                    RelationalBridgeBasisRetirementProgress::Stopped(cause) => {
                        return Err(Denial::Execution(cause))
                    }
                    RelationalBridgeBasisRetirementProgress::Refused(cause) => {
                        return Err(Denial::BridgeBasis(cause))
                    }
                    RelationalBridgeBasisRetirementProgress::Complete => {}
                }
                self.retired_head = Some(
                    self.head
                        .take()
                        .expect("same completed head")
                        .into_retired()
                        .unwrap_or_else(|_| unreachable!("owner reported Complete")),
                );
                return Ok(Step::Advanced);
            }
            if let Some(subscription) = self.subscription.take() {
                match subscription.try_retire_for_request(request) {
                    Ok(retired) => self.retired_subscription = Some(retired),
                    Err((subscription, cause)) => {
                        self.subscription = Some(subscription);
                        return Err(Denial::CanonicalSubscription(cause));
                    }
                }
                return Ok(Step::Advanced);
            }
        }
        if let Some(runtime) = self.runtime.take() {
            match runtime.try_retire_immutable_alias(request, owner) {
                Ok(retired) => self.retired_runtime = Some(retired),
                Err((runtime, cause)) => {
                    self.runtime = Some(runtime);
                    return Err(Denial::NativeAlias(cause));
                }
            }
            return Ok(Step::Advanced);
        }
        match self
            .source
            .take()
            .expect("same final source")
            .try_retire_alias_for_request(request, owner)
        {
            Ok(retired) => self.retired_source = Some(retired),
            Err((source, cause)) => {
                self.source = Some(source);
                return Err(Denial::NativeAlias(cause));
            }
        }
        Ok(Step::Complete)
    }
}
