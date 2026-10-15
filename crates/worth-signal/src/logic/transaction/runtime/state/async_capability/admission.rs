use crate::data::aspect::{AspectMask, AspectVersion};
use crate::data::async_node::{
    AsyncNodeAdmissionClass, AsyncNodeAdmissionClassification, AsyncNodeConditionBlockClass,
    AsyncNodeRequestAdmissionReport, AsyncNodeRequestIntent, AsyncNodeRevalidationIntent,
    AsyncNodeRevalidationReport,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::EvaluationCondition;
use crate::data::proof::invalidation::revalidation::NodeInvalidationInput;
use crate::data::resource::{
    ResourceBoundaryPerformanceEnvelope, ResourceLifecycleClass, ResourceNodeId,
};
use crate::data::temporal::{TemporalCondition, TemporalPreviousValueReference, TemporalWakeOwner};

use super::{AsyncAdmissionMode, SignalRuntime};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn admit_async_node_request(
        &mut self,
        intent: AsyncNodeRequestIntent,
    ) -> Result<AsyncNodeRequestAdmissionReport, SignalError> {
        let classification = self.classify_async_node_admission(
            intent.node(),
            intent.previous_value_reference(),
            intent.requires_clean_dependencies(),
            AsyncAdmissionMode::NewLineage,
        )?;
        if classification.class() == AsyncNodeAdmissionClass::BlockedByCondition {
            return Ok(AsyncNodeRequestAdmissionReport::blocked(classification));
        }
        let resource_admission = self.admit_resource_request(intent.into_resource_intent())?;
        let classification = self.with_async_classification_performance(
            classification,
            resource_admission.performance(),
        );
        Ok(AsyncNodeRequestAdmissionReport::admitted(
            classification,
            resource_admission,
        ))
    }

    pub fn revalidate_async_node(
        &mut self,
        intent: AsyncNodeRevalidationIntent,
    ) -> Result<AsyncNodeRevalidationReport, SignalError> {
        let classification = self.classify_async_node_admission(
            intent.node(),
            intent.previous_value_reference(),
            intent.requires_clean_dependencies(),
            AsyncAdmissionMode::Refresh,
        )?;
        if classification.class() == AsyncNodeAdmissionClass::BlockedByCondition {
            return Ok(AsyncNodeRevalidationReport::blocked(classification));
        }
        let resource_revalidation = self.revalidate_resource_node(intent.into_resource_intent())?;
        let classification = self.with_async_classification_performance(
            classification,
            resource_revalidation.performance(),
        );
        Ok(AsyncNodeRevalidationReport::revalidated(
            classification,
            resource_revalidation,
        ))
    }

    fn classify_async_node_admission(
        &mut self,
        node: NodeId,
        previous_value_reference: Option<&TemporalPreviousValueReference>,
        requires_clean_dependencies: bool,
        mode: AsyncAdmissionMode,
    ) -> Result<AsyncNodeAdmissionClassification, SignalError> {
        self.ensure_live_async_node_owner(node, "admit async node request")?;
        let decision_digest = self
            .resource
            .descriptor_for_node(ResourceNodeId::from_node(node))
            .map(|descriptor| descriptor.lowered_policy_bundle().bundle_digest().clone())
            .ok_or_else(|| {
                self.with_resource_telemetry(|telemetry| {
                    telemetry.resource_undeclared_owner_denial_count += 1;
                });
                SignalError::invalid_input(format!(
                    "cannot use async capability APIs for undeclared node {}",
                    node
                ))
            })?;
        let node_state = self.graph.get_state(node)?;
        let invalidation = self.graph.node_invalidation_input(node)?;
        let (dirty_aspects, dirty_scoped_aspects, pending_node) = match invalidation {
            NodeInvalidationInput::Pending(_) => (AspectMask::EMPTY, Vec::new(), true),
            NodeInvalidationInput::Resolved(causes) => (
                causes.dirty_aspects(),
                causes.dirty_scoped_aspects().to_vec(),
                false,
            ),
            NodeInvalidationInput::ResolvedNoChange(_) => (AspectMask::EMPTY, Vec::new(), false),
        };
        let pending_dependency = pending_node || self.graph.has_current_unsettled_upstream(node)?;
        let contract = self.graph.get_contract(node)?.clone();
        let lifecycle_class = self
            .resource
            .lifecycle_summary_for_node(ResourceNodeId::from_node(node))
            .map(|summary| summary.lifecycle())
            .unwrap_or(ResourceLifecycleClass::Unrequested);
        let condition = self.graph.node_condition(node)?;
        let max_dependency_delta = max_dependency_delta(&self.graph, node)?;

        let mut block_class = pending_dependency
            .then_some(AsyncNodeConditionBlockClass::DependencyNotReady)
            .or_else(|| self.locality_block_class(&contract, dirty_aspects, &dirty_scoped_aspects));
        if block_class.is_none() {
            block_class = match previous_value_reference {
                Some(reference) => self.previous_value_reference_block_class(node, reference)?,
                None => None,
            };
        }
        if block_class.is_none() {
            block_class =
                self.condition_block_class(node, &condition, dirty_aspects, max_dependency_delta)?;
        }
        if block_class.is_none()
            && requires_clean_dependencies
            && node_state != crate::data::node::NodeState::Clean
        {
            block_class = Some(AsyncNodeConditionBlockClass::DependencyNotReady);
        }

        let class = match (mode, block_class, lifecycle_class) {
            (
                AsyncAdmissionMode::Refresh,
                Some(AsyncNodeConditionBlockClass::AspectFilterMismatch),
                lifecycle,
            )
            | (
                AsyncAdmissionMode::Refresh,
                Some(AsyncNodeConditionBlockClass::DeltaThresholdNotCrossed),
                lifecycle,
            )
            | (
                AsyncAdmissionMode::Refresh,
                Some(AsyncNodeConditionBlockClass::TemporalConditionNotReady),
                lifecycle,
            ) if lifecycle != ResourceLifecycleClass::Unrequested => {
                self.with_resource_telemetry(|telemetry| {
                    telemetry.async_node_revalidation_eligibility_count += 1;
                });
                AsyncNodeAdmissionClass::RefreshEligibleNoNewLineage
            }
            (_, Some(_), _) => {
                self.with_resource_telemetry(|telemetry| {
                    telemetry.async_node_condition_blocked_admission_count += 1;
                });
                AsyncNodeAdmissionClass::BlockedByCondition
            }
            _ => AsyncNodeAdmissionClass::AdmittedNewLineage,
        };

        if class != AsyncNodeAdmissionClass::BlockedByCondition
            && self.is_interior_async_gate(node)?
        {
            self.with_resource_telemetry(|telemetry| {
                telemetry.async_node_interior_gate_admission_count += 1;
            });
        }

        self.record_locality_counters(mode, class, &contract, dirty_aspects, &dirty_scoped_aspects);
        let performance = self.record_async_admission_envelope(mode, class);
        Ok(AsyncNodeAdmissionClassification::new(
            node,
            node_state,
            lifecycle_class,
            condition,
            class,
            block_class,
            dirty_aspects,
            dirty_scoped_aspects.len() as u32,
            contract
                .projection
                .consumes_partitions
                .as_ref()
                .map_or(0, |scopes| scopes.len() as u32),
            max_dependency_delta,
            requires_clean_dependencies,
            previous_value_reference.cloned(),
            decision_digest,
            performance,
        ))
    }

    fn locality_block_class(
        &self,
        contract: &crate::data::node::NodeContract,
        dirty_aspects: AspectMask,
        dirty_scoped_aspects: &[(
            crate::data::aspect::Aspect,
            crate::data::output::PartitionSubscription,
        )],
    ) -> Option<AsyncNodeConditionBlockClass> {
        if dirty_aspects.is_empty()
            || contract.cares_about_correlated_change(dirty_aspects, dirty_scoped_aspects)
        {
            return None;
        }
        if !dirty_scoped_aspects.is_empty() && contract.projection.consumes_partitions.is_some() {
            Some(AsyncNodeConditionBlockClass::PartitionScopeMismatch)
        } else {
            Some(AsyncNodeConditionBlockClass::ContractAspectMismatch)
        }
    }

    fn record_locality_counters(
        &mut self,
        mode: AsyncAdmissionMode,
        class: AsyncNodeAdmissionClass,
        contract: &crate::data::node::NodeContract,
        dirty_aspects: AspectMask,
        dirty_scoped_aspects: &[(
            crate::data::aspect::Aspect,
            crate::data::output::PartitionSubscription,
        )],
    ) {
        if mode != AsyncAdmissionMode::Refresh
            || class == AsyncNodeAdmissionClass::BlockedByCondition
        {
            return;
        }
        if !dirty_aspects.is_empty() && contract.projection.consumes != AspectMask::ALL {
            self.with_resource_telemetry(|telemetry| {
                telemetry.async_node_aspect_local_refresh_count += 1;
            });
        }
        if !dirty_scoped_aspects.is_empty() && contract.projection.consumes_partitions.is_some() {
            self.with_resource_telemetry(|telemetry| {
                telemetry.async_node_partition_local_refresh_count += 1;
            });
        }
    }

    fn condition_block_class(
        &mut self,
        node: NodeId,
        condition: &EvaluationCondition,
        dirty_aspects: AspectMask,
        max_dependency_delta: u64,
    ) -> Result<Option<AsyncNodeConditionBlockClass>, SignalError> {
        match condition {
            EvaluationCondition::Always | EvaluationCondition::OnDemand => Ok(None),
            EvaluationCondition::AspectFilter(mask) => Ok((!dirty_aspects.is_empty()
                && !dirty_aspects.intersects(*mask))
            .then_some(AsyncNodeConditionBlockClass::AspectFilterMismatch)),
            EvaluationCondition::DeltaThreshold(threshold) => {
                let has_dependency_snapshot =
                    !self.graph.get_dep_snapshot(node)?.entries().is_empty();
                Ok((has_dependency_snapshot
                    && !dirty_aspects.is_empty()
                    && (max_dependency_delta as f64) <= *threshold)
                    .then_some(AsyncNodeConditionBlockClass::DeltaThresholdNotCrossed))
            }
            EvaluationCondition::Temporal(condition) => {
                self.with_resource_telemetry(|telemetry| {
                    telemetry.async_node_condition_governed_admission_count += 1;
                });
                Ok((!self.temporal_condition_ready(node, condition)?)
                    .then_some(AsyncNodeConditionBlockClass::TemporalConditionNotReady))
            }
            EvaluationCondition::Custom(_) => Ok(Some(
                AsyncNodeConditionBlockClass::CustomConditionResolverRequired,
            )),
            EvaluationCondition::Installed(_) => Ok(Some(
                AsyncNodeConditionBlockClass::CustomConditionResolverRequired,
            )),
        }
    }

    fn previous_value_reference_block_class(
        &mut self,
        node: NodeId,
        reference: &TemporalPreviousValueReference,
    ) -> Result<Option<AsyncNodeConditionBlockClass>, SignalError> {
        self.with_resource_telemetry(|telemetry| {
            telemetry.async_node_previous_value_governed_admission_count += 1;
        });
        let current_output_identity = self
            .graph
            .observe()
            .runtime_artifact_warm(node)?
            .and_then(|warm| warm.output_identity.clone());
        let current_version: AspectVersion = self.graph.node_aspect_version(node)?;
        let drifted = reference.branch_id() != self.graph.current_branch().id
            || reference.node() != node
            || reference.aspect_version() != current_version
            || reference.output_identity().cloned() != current_output_identity;
        Ok(drifted.then_some(AsyncNodeConditionBlockClass::PreviousValueReferenceDrifted))
    }

    fn temporal_condition_ready(
        &mut self,
        node: NodeId,
        condition: &TemporalCondition,
    ) -> Result<bool, SignalError> {
        if matches!(condition, TemporalCondition::AtOrAfter(at) if self.clock_basis().current_tick() >= at.tick())
        {
            return Ok(true);
        }
        let owner = TemporalWakeOwner::Node(node);
        if let Some(ready) = self.temporal.ready_wake_for_owner(owner) {
            return Ok(ready.condition() == condition);
        }
        if let Some(wake_id) = self.temporal.active_wake_for_owner(owner) {
            if let Some(scheduled) = self.temporal.scheduled_wake(wake_id)
                && scheduled.condition() == condition
                && scheduled.due_tick() <= self.clock_basis().current_tick()
            {
                let ready = self.promote_temporal_wake_ready(wake_id)?;
                return Ok(ready.condition() == condition);
            }
        } else {
            self.admit_node_temporal_wake_with_summary(node)?;
        }
        Ok(self
            .temporal
            .ready_wake_for_owner(owner)
            .is_some_and(|ready| ready.condition() == condition))
    }

    fn record_async_admission_envelope(
        &mut self,
        mode: AsyncAdmissionMode,
        class: AsyncNodeAdmissionClass,
    ) -> ResourceBoundaryPerformanceEnvelope {
        let envelope = match mode {
            AsyncAdmissionMode::NewLineage => {
                ResourceBoundaryPerformanceEnvelope::request_admission(
                    0,
                    u32::from(class == AsyncNodeAdmissionClass::BlockedByCondition),
                    0,
                )
            }
            AsyncAdmissionMode::Refresh => {
                ResourceBoundaryPerformanceEnvelope::revalidation_admission(
                    0,
                    u32::from(class == AsyncNodeAdmissionClass::BlockedByCondition),
                    0,
                    0,
                )
            }
        };
        self.with_resource_telemetry(|telemetry| {
            telemetry.record_boundary_performance_envelope(envelope);
        });
        envelope
    }
}

fn max_dependency_delta(graph: &SignalGraph, node: NodeId) -> Result<u64, SignalError> {
    let mut max_delta = 0;
    for snapshot_entry in graph.get_dep_snapshot(node)?.entries() {
        if !graph.is_alive(snapshot_entry.source) {
            continue;
        }
        let current_version = graph.node_version_for_scope(
            snapshot_entry.source,
            snapshot_entry.aspect,
            snapshot_entry.scope.as_ref(),
        )?;
        max_delta = max_delta.max(current_version.abs_diff(snapshot_entry.cached_version));
    }
    Ok(max_delta)
}
