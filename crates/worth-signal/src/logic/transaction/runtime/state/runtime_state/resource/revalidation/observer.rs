use crate::data::resource::{
    LoweredResourceDescriptor, ObserverDemandResourceRevalidationProof, ResourceNodeId,
    ResourceRevalidationDenialClass, ResourceRevalidationReport,
};

use crate::logic::transaction::{CommittedObservationEventSummary, ObservationBoundaryOutcome};

use super::super::super::super::resource::ResourceRuntimeState;

use super::super::super::SignalRuntime;
mod policy_v1;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn latest_committed_observer_demand_event_for_node(
        &self,
        node: ResourceNodeId,
    ) -> Option<&CommittedObservationEventSummary> {
        self.observe()
            .latest_observation_summary()?
            .boundary_events
            .iter()
            .find(|event| {
                event.outcome == ObservationBoundaryOutcome::Delivered
                    && event.trigger_matched
                    && event
                        .matched_nodes
                        .iter()
                        .any(|matched| matched == node.node())
            })
    }

    fn observer_demand_observation_digest(event: &CommittedObservationEventSummary) -> String {
        let policy = policy_v1::encode(event.policy);
        let matched_nodes = event
            .matched_nodes
            .iter()
            .map(|node| format!("{}:{}", node.index(), node.generation()))
            .collect::<Vec<_>>();
        ResourceRuntimeState::observer_demand_observation_digest(
            event.observer_id.get(),
            event.handle_id.get(),
            policy,
            &matched_nodes,
            event.visited,
            event.recomputed,
            event.meaningful_change,
            event.trigger_matched,
            matches!(event.outcome, ObservationBoundaryOutcome::Delivered),
        )
    }

    pub fn prove_observer_demand_resource_revalidation(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<ObserverDemandResourceRevalidationProof, crate::data::error::SignalError> {
        if !self.graph.is_alive(node.node()) {
            self.with_resource_telemetry(|telemetry| {
                telemetry.resource_non_live_owner_denial_count += 1
            });
            return Err(crate::data::error::SignalError::invalid_input(format!(
                "cannot mint observer-demand revalidation proof for non-live resource node {}",
                node.node()
            )));
        }
        let capture_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        if capture_telemetry {
            self.telemetry
                .resource
                .resource_revalidation_observer_demand_proof_check_count += 1;
        }
        let descriptor = self.resource.descriptor_for_node(node).ok_or_else(|| {
            crate::data::error::SignalError::invalid_input(format!(
                "cannot mint observer-demand revalidation proof for undeclared resource node {}",
                node.node()
            ))
        })?;
        if !descriptor
            .revalidation_decision_plan()
            .permits_observer_demand_revalidation()
        {
            return Err(crate::data::error::SignalError::invalid_input(format!(
                "cannot mint observer-demand revalidation proof for policy-disabled resource node {}",
                node.node()
            )));
        }
        let event = self
            .latest_committed_observer_demand_event_for_node(node)
            .ok_or_else(|| {
                crate::data::error::SignalError::invalid_input(format!(
                    "cannot mint observer-demand revalidation proof without committed matching observation for resource node {}",
                    node.node()
                ))
            })?;
        Ok(ObserverDemandResourceRevalidationProof::new(
            node,
            event.observer_id.get(),
            event.handle_id.get(),
            Self::observer_demand_observation_digest(event),
            descriptor
                .revalidation_decision_plan()
                .decision_digest()
                .clone(),
        ))
    }

    pub fn revalidate_resource_node_for_observer_demand(
        &mut self,
        proof: ObserverDemandResourceRevalidationProof,
    ) -> Result<ResourceRevalidationReport, crate::data::error::SignalError> {
        let resource_node = proof.node();
        if !self.graph.is_alive(resource_node.node()) {
            self.with_resource_telemetry(|telemetry| {
                telemetry.resource_non_live_owner_denial_count += 1
            });
            return Err(crate::data::error::SignalError::invalid_input(format!(
                "cannot observer-demand revalidate non-live resource node {}",
                resource_node.node()
            )));
        }

        let current_tick = self.clock_basis().current_tick();
        let capture_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        if capture_telemetry {
            self.telemetry
                .resource
                .resource_revalidation_policy_decision_count += 1;
        }
        let event = self.latest_committed_observer_demand_event_for_node(resource_node);
        if let Some(class) = self
            .resource
            .validate_observer_demand_resource_revalidation_proof(&proof)
            .or_else(|| match event {
                Some(event)
                    if proof.observer_id() == event.observer_id.get()
                        && proof.handle_id() == event.handle_id.get()
                        && proof.observation_digest()
                            == Self::observer_demand_observation_digest(event) =>
                {
                    None
                }
                _ => Some(ResourceRevalidationDenialClass::ObserverDemandProofMismatch),
            })
        {
            return Ok(self.resource.deny_resource_revalidation_for_report(
                proof.node(),
                class,
                capture_telemetry.then_some(&mut self.telemetry.resource),
            ));
        }

        if capture_telemetry {
            self.telemetry
                .resource
                .resource_observer_demand_revalidation_count += 1;
        }
        let prepared_revalidation =
            match self.resource.prepare_observer_demand_resource_revalidation(
                proof,
                capture_telemetry.then_some(&mut self.telemetry.resource),
            ) {
                Ok(prepared) => prepared,
                Err(report) => return Ok(report),
            };
        let timeout_plan = self
            .resource
            .descriptor_for_node(resource_node)
            .map(|descriptor| descriptor.timeout_decision_plan().clone())
            .unwrap_or_else(LoweredResourceDescriptor::default_timeout_decision_plan);
        let resolved_timeout =
            self.resolve_timeout_admission(resource_node, &timeout_plan, current_tick, None)?;
        let prior_timeout_wake = self.resource.active_timeout_wake_for_node(resource_node);
        let prior_stale_after_wake = self
            .resource
            .active_stale_after_wake_for_node(resource_node);
        let scheduled_timeout_wake = resolved_timeout
            .as_ref()
            .map(|resolved| self.schedule_resource_timeout_wake(resource_node, resolved))
            .transpose()?;
        let scheduled_timeout_admission = resolved_timeout.map(|resolved| {
            resolved.bind_scheduled_wake(
                scheduled_timeout_wake
                    .as_ref()
                    .expect("resolved timeout must schedule one temporal wake")
                    .id(),
            )
        });
        let report = self.resource.admit_prepared_resource_revalidation(
            prepared_revalidation,
            self.graph.current_branch().id,
            current_tick,
            scheduled_timeout_admission,
            capture_telemetry.then_some(&mut self.telemetry.resource),
        );
        self.reconcile_resource_revalidation_wakes(
            &report,
            resource_node,
            prior_timeout_wake,
            prior_stale_after_wake,
            scheduled_timeout_wake,
        )?;
        Ok(report)
    }
}
