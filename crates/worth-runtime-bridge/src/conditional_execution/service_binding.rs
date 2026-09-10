use std::sync::{Arc, Mutex, PoisonError};

use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};
use worth_signal::facade::branch::{
    SignalConditionalDefinitionPublicationPort, SignalConditionalExecutionPort,
    SignalOwnerServicePorts,
};
use worth_signal::facade::SignalRuntime;

use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeInstalledConditionalLowering,
    BridgeOwnedSignalRuntime,
};

pub(super) struct BridgeSignalServiceBinding {
    _services: SignalOwnerServicePorts<(), (), (), (), ()>,
    definition_publication: Option<SignalConditionalDefinitionPublicationPort<(), (), (), (), ()>>,
    source_owner: ConditionalSourceObservationOwner,
    conditional_port: Mutex<super::signal_port::BridgeConditionalSignalPort>,
}

impl BridgeOwnedSignalRuntime {
    /// Seals the Signal owner partition and publishes the one claimant-bound
    /// conditional operation port used by every installed lowering.
    pub(super) fn seal_conditional_operations(&mut self) -> Result<(), BridgeConditionalDenial> {
        if self.signal_services.is_some() {
            return Ok(());
        }
        let signal_runtime = self
            .signal_runtime
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let binding = {
            let mut lowerings = self
                .conditional_lowerings
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            BridgeSignalServiceBinding::seal(
                signal_runtime,
                &mut lowerings,
                &self.bridge.signal_aspect_lowering_owner,
            )?
        };
        self.signal_services = Some(binding);
        Ok(())
    }

    pub(super) fn signal_services(
        &self,
    ) -> Result<&BridgeSignalServiceBinding, BridgeConditionalDenial> {
        self.signal_services.as_ref().ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                "Signal owner services were not sealed before operation dispatch",
            )
        })
    }

    pub(super) fn signal_services_mut(
        &mut self,
    ) -> Result<&mut BridgeSignalServiceBinding, BridgeConditionalDenial> {
        self.signal_services.as_mut().ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                "Signal owner services were not sealed before operation dispatch",
            )
        })
    }
}

impl BridgeSignalServiceBinding {
    pub(super) fn seal(
        runtime: &mut SignalRuntime<(), (), (), (), ()>,
        lowerings: &mut super::lowering_registry::BridgeConditionalLoweringRegistry,
        claimant: &worth_signal::facade::SignalAspectLoweringOwner,
    ) -> Result<Self, BridgeConditionalDenial> {
        let selected = runtime.current_branch();
        let basis = runtime
            .observe_signal_branch_basis(selected)
            .map_err(|error| service_denial("selected basis admission", error))?;
        let source_owner = ConditionalSourceObservationOwner::fresh();
        let source_authority = source_owner.authority();
        let services = runtime
            .owner_component_services()
            .map_err(|error| service_denial("owner-service sealing", error))?;
        let definition_publication = runtime
            .runtime_world_definition_publication_port()
            .map_err(|error| service_denial("definition-publication sealing", error))?;
        let conditional_port = runtime
            .issue_conditional_execution_service(&basis, claimant, &source_authority)
            .map_err(|error| service_denial("conditional service issuance", error))?;
        let conditional_port = Arc::new(conditional_port);
        let installed = lowerings.values().cloned().collect::<Vec<_>>();
        for lowering in installed {
            lowering.bind_signal_port(Arc::clone(&conditional_port));
            lowerings.index_exact_basis(&lowering);
        }
        let conditional_port =
            super::signal_port::BridgeConditionalSignalPort::shared(conditional_port);
        Ok(Self {
            _services: services,
            definition_publication: Some(definition_publication),
            source_owner,
            conditional_port: Mutex::new(conditional_port),
        })
    }

    pub(super) fn conditional_port(
        &self,
        lowering: &BridgeInstalledConditionalLowering,
    ) -> Result<super::signal_port::BridgeConditionalSignalPort, BridgeConditionalDenial> {
        lowering.signal_port().ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                "installed lowering has no exact Signal service binding",
            )
        })
    }

    pub(super) fn owner_services(&self) -> SignalOwnerServicePorts<(), (), (), (), ()> {
        self._services.clone()
    }

    pub(super) fn take_definition_publication(
        &mut self,
    ) -> Result<
        SignalConditionalDefinitionPublicationPort<(), (), (), (), ()>,
        BridgeConditionalDenial,
    > {
        self.definition_publication.take().ok_or_else(|| {
            BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::SignalExecution,
                "Runtime World already owns the Signal definition-publication capability",
            )
        })
    }

    pub(super) fn conditional_extension_port(
        &self,
    ) -> super::signal_port::BridgeConditionalSignalPort {
        self.conditional_port
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(super) fn publish_conditional_port_from_lowering(
        &self,
        lowering: &BridgeInstalledConditionalLowering,
    ) {
        if let Some(successor) = lowering.signal_port() {
            *self
                .conditional_port
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = successor;
        }
    }

    pub(super) fn publish_reconstituted_service(
        &self,
        successor: Arc<SignalConditionalExecutionPort<(), (), ()>>,
    ) {
        *self
            .conditional_port
            .lock()
            .unwrap_or_else(PoisonError::into_inner) =
            super::signal_port::BridgeConditionalSignalPort::shared(successor);
    }

    pub(super) fn issuance_basis(&self) -> worth_signal::facade::branch::AdmittedSignalBranchBasis {
        self.conditional_port
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .issuance_basis()
            .clone()
    }

    pub(super) fn evaluation_source(
        &self,
        snapshot: Option<
            &crate::snapshot::AdmittedSnapshotContext<
                Box<dyn crate::snapshot::TruthSnapshotReader>,
            >,
        >,
    ) -> ConditionalEvaluationSource {
        match snapshot {
            Some(snapshot) => self
                .source_owner
                .admit(super::source_projection::conditional_source_projection(
                    snapshot.snapshot_identity(),
                ))
                .into(),
            None => ConditionalEvaluationSource::NoRelationalSource,
        }
    }
}

fn service_denial(context: &str, error: impl std::fmt::Debug) -> BridgeConditionalDenial {
    BridgeConditionalDenial::new(
        BridgeConditionalDenialKind::SignalExecution,
        format!("Signal {context} failed: {error:?}"),
    )
}
