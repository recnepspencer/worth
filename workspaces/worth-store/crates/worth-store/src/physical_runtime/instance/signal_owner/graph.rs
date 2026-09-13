use std::sync::Arc;

use worth_signal::facade::specialist::EvaluationOutput;
use worth_signal::facade::{
    AspectVersion, EvaluationContext, NodeEvaluationResult, NodeId, ResourceRequestHandle,
    RuntimeClockBasis, SignalError, SignalGraph, SignalRuntime,
};

use crate::physical_runtime::work::{
    AdmittedPhysicalWork, BlockedPhysicalWork, InstalledPhysicalSignalTopology,
    PendingPhysicalSignalTopology, PhysicalSignalAspectBindingDigest,
    PhysicalSignalAspectBindingSet, PhysicalSignalReadinessEvidence, PhysicalWorkPreEffectDenial,
    PhysicalWorkReadiness, ReadyPhysicalWork,
};

use super::PhysicalSignalConstructionFailure;

mod abandonment;
mod command;
mod completion;
mod delta;
mod locality;
mod observation;
mod request_cleanup;

pub(super) use abandonment::PhysicalSignalAbandonmentFailure;
use locality::PhysicalSignalLocalityIndex;
pub(in crate::physical_runtime::instance::signal_owner) use observation::PhysicalSignalGraphObservation;

struct PhysicalSignalContext {
    version: u64,
}

pub(super) struct PhysicalSignalGraph {
    bindings: Arc<PhysicalSignalAspectBindingSet>,
    runtime: SignalRuntime<(), (), (), PhysicalSignalContext, ()>,
    topology: InstalledPhysicalSignalTopology,
    context: PhysicalSignalContext,
    locality: PhysicalSignalLocalityIndex,
    healthy: bool,
}

impl PhysicalSignalGraph {
    pub(super) fn build(
        bindings: Arc<PhysicalSignalAspectBindingSet>,
    ) -> Result<Self, PhysicalSignalConstructionFailure> {
        let mut graph = SignalGraph::new();
        let pending = PendingPhysicalSignalTopology::build(&mut graph, &bindings)
            .map_err(|_| PhysicalSignalConstructionFailure::CapabilityDeclarationRejected)?;
        let mut runtime = SignalRuntime::build_for::<PhysicalSignalContext>(graph);
        let topology = pending
            .attach(&mut runtime)
            .map_err(|_| PhysicalSignalConstructionFailure::CapabilityDeclarationRejected)?;
        let locality = PhysicalSignalLocalityIndex::bounded(bindings.capacity().commands());
        let mut owner = Self {
            bindings,
            runtime,
            topology,
            context: PhysicalSignalContext { version: 0 },
            locality,
            healthy: true,
        };
        owner
            .evaluate_dirty()
            .map_err(|_| PhysicalSignalConstructionFailure::DependencyInitializationRejected)?;
        Ok(owner)
    }

    pub(super) fn clock_basis(&self) -> RuntimeClockBasis {
        self.runtime.clock_basis()
    }

    fn request(
        &mut self,
        route: PhysicalSignalAspectBindingDigest,
        admitted: AdmittedPhysicalWork,
    ) -> Result<PhysicalWorkReadiness, PhysicalWorkPreEffectDenial> {
        let capability = self
            .topology
            .capability(route, admitted.signal_family())
            .cloned()
            .ok_or(PhysicalWorkPreEffectDenial::CapabilityAbsent)?;
        if !self.locality.register(&admitted) {
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        }
        let identity = admitted.intent().identity();
        if !admitted.register_signal_locality(route) {
            self.release_identity(identity);
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        }
        self.settle_dependency(capability.node()).map_err(|_| {
            self.release_identity(identity);
            PhysicalWorkPreEffectDenial::SignalOwnerUnavailable
        })?;
        let report = self
            .runtime
            .admit_async_node_request(capability.request_intent_requiring_clean_dependencies())
            .map_err(|_| {
                self.release_identity(identity);
                PhysicalWorkPreEffectDenial::SignalOwnerUnavailable
            })?;
        let classification = report.classification();
        let Some(resource) = report.resource_admission() else {
            return Ok(PhysicalWorkReadiness::Blocked(BlockedPhysicalWork::new(
                admitted,
                classification.class(),
                classification.condition_block_class(),
            )));
        };
        let request = resource.admitted_request();
        if !self.locality.bind(identity, request.handle()) {
            self.cancel_unbound_request(identity, request.handle());
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        }
        Ok(PhysicalWorkReadiness::Ready(ReadyPhysicalWork::new(
            admitted,
            PhysicalSignalReadinessEvidence {
                signal_request: request.handle(),
                supersession: None,
                replaces: None,
                attempt: request.attempt(),
                capability_registry: capability.registry_digest().clone(),
                capability_bundle: capability.bundle_digest().clone(),
                payload_contract: capability.payload_contract_digest().clone(),
            },
        )))
    }

    fn revalidate_ready(
        &mut self,
        route: PhysicalSignalAspectBindingDigest,
        ready: ReadyPhysicalWork,
    ) -> Result<PhysicalWorkReadiness, PhysicalWorkPreEffectDenial> {
        if !self.locality.invalidated(ready.intent().identity()) {
            return Ok(PhysicalWorkReadiness::Ready(ready));
        }
        let (admitted, active) = ready.into_signal_parts();
        self.revalidate_signal(route, admitted, active)
    }

    fn revalidate_signal(
        &mut self,
        route: PhysicalSignalAspectBindingDigest,
        admitted: AdmittedPhysicalWork,
        active: ResourceRequestHandle,
    ) -> Result<PhysicalWorkReadiness, PhysicalWorkPreEffectDenial> {
        if !self.locality.register(&admitted) {
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        }
        let identity = admitted.intent().identity();
        if !admitted.register_signal_locality(route) {
            self.release_identity(identity);
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        }
        let Some(capability) = self
            .topology
            .capability(route, admitted.signal_family())
            .cloned()
        else {
            self.release_identity(identity);
            return Err(PhysicalWorkPreEffectDenial::CapabilityAbsent);
        };
        self.settle_dependency(capability.node()).map_err(|_| {
            self.release_identity(identity);
            PhysicalWorkPreEffectDenial::SignalOwnerUnavailable
        })?;
        let report = match self.runtime.revalidate_async_node(
            capability.revalidation_intent_requiring_clean_dependencies(active),
        ) {
            Ok(report) => report,
            Err(_) => {
                self.release_identity(identity);
                return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
            }
        };
        let classification = report.classification();
        let Some(resource) = report.resource_revalidation() else {
            return Ok(PhysicalWorkReadiness::Blocked(
                BlockedPhysicalWork::from_revalidation(
                    admitted,
                    classification.class(),
                    classification.condition_block_class(),
                    active,
                ),
            ));
        };
        let Some(revalidation) = resource.admitted_revalidation() else {
            return Ok(PhysicalWorkReadiness::Blocked(
                BlockedPhysicalWork::from_revalidation(
                    admitted,
                    classification.class(),
                    classification.condition_block_class(),
                    active,
                ),
            ));
        };
        let request = revalidation.admitted_request();
        let Some(supersession) = revalidation
            .supersession_record()
            .filter(|record| record.previous() == active && record.replacing() == request.handle())
        else {
            self.cancel_unbound_request(identity, request.handle());
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        };
        if !self.locality.bind(identity, request.handle()) {
            self.cancel_unbound_request(identity, request.handle());
            return Err(PhysicalWorkPreEffectDenial::SignalOwnerUnavailable);
        }
        Ok(PhysicalWorkReadiness::Ready(ReadyPhysicalWork::new(
            admitted,
            PhysicalSignalReadinessEvidence {
                signal_request: request.handle(),
                supersession: Some(supersession.clone()),
                replaces: Some(active),
                attempt: request.attempt(),
                capability_registry: capability.registry_digest().clone(),
                capability_bundle: capability.bundle_digest().clone(),
                payload_contract: capability.payload_contract_digest().clone(),
            },
        )))
    }

    fn evaluate_dirty(&mut self) -> Result<(), SignalError> {
        let topology = &self.topology;
        let bindings = &self.bindings;
        let version = self.context.version;
        self.runtime.evaluate_dirty(&self.context, &|view| {
            evaluate_physical_signal_node(topology, bindings, version, view)
        })?;
        Ok(())
    }

    fn settle_dependency(&mut self, node: NodeId) -> Result<(), SignalError> {
        let topology = &self.topology;
        let bindings = &self.bindings;
        let version = self.context.version;
        // A source update can dirty its consumer after the dirty-target snapshot.
        // Settle only the requested dependency chain before Signal checks readiness.
        self.runtime.read(node, &self.context, &|view| {
            evaluate_physical_signal_node(topology, bindings, version, view)
        })?;
        Ok(())
    }

    pub(super) fn release_identity(
        &mut self,
        identity: crate::physical_runtime::PhysicalWorkIdentity,
    ) {
        if let Some(signal) = self.locality.signal(identity) {
            let _ = self.runtime.cancel_resource_request(
                signal,
                worth_signal::facade::ResourceCancellationReason::RuntimePolicy,
            );
        }
        self.locality.release_identity(identity);
    }

    pub(super) fn release_signal(&mut self, signal: ResourceRequestHandle) {
        self.locality.release_signal(signal);
    }

    pub(super) fn release_envelope(
        &mut self,
        envelope: &worth_signal::facade::RawCompletionEnvelope,
    ) {
        self.locality.release_envelope(envelope);
    }
}

fn evaluate_physical_signal_node(
    topology: &InstalledPhysicalSignalTopology,
    bindings: &PhysicalSignalAspectBindingSet,
    version: u64,
    view: &mut EvaluationContext<'_, PhysicalSignalContext>,
) -> Result<EvaluationOutput, SignalError> {
    if let Some(binding) = bindings
        .bindings()
        .iter()
        .enumerate()
        .find_map(|(slot, binding)| {
            (topology.source_for_slot(slot) == Some(view.node())).then_some(binding)
        })
    {
        return Ok(view.finish(NodeEvaluationResult::from_version(
            AspectVersion::from_updates([(binding.signal_aspect(), version)]),
        )));
    }
    if let (Some(route), Some(family)) = (
        topology.route_for_node(view.node()),
        topology.family_for_node(view.node()),
    ) {
        if let Some(binding) = bindings
            .bindings()
            .iter()
            .find(|binding| binding.digest() == route && binding.serves_family(family))
        {
            let slot = binding.signal_aspect().index();
            let source = topology
                .source_for_slot(slot)
                .expect("installed binding and source slots remain aligned");
            if let Some(partition) = binding.partition() {
                let _ = view.read_partitioned_aspect_version(
                    source,
                    binding.signal_aspect(),
                    partition.clone(),
                )?;
            } else {
                let _ = view.read_aspect_version(source, binding.signal_aspect())?;
            }
        }
    }
    Ok(view.finish(NodeEvaluationResult::from_version(AspectVersion::zero())))
}
