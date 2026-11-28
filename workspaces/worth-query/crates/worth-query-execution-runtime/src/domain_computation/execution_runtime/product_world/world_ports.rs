//! Operational World access never carries Home lifecycle ownership.
use std::sync::Arc;
use worth_runtime_world::facade::*;

#[derive(Clone)]
pub struct WorthQueryProductWorldPorts {
    identity: RuntimeWorldOwnerIdentity,
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    branch: RuntimeWorldBranchPort,
    observation: RuntimeWorldObservationPort,
    inspection: RuntimeWorldInspectionPort,
    history_custody: RuntimeWorldHistoryCustodyPort,
    recovery: RuntimeWorldRecoveryPort,
    execution_authority: Option<Arc<worth_execution::ExecutionAuthority>>,
    // Passive memory custody is cloned with every product runtime. Directory
    // attachment supplies None; no operation dispatches through this keepalive.
    _memory_world_keepalive: Option<Arc<RuntimeWorldOwner<(), (), (), (), ()>>>,
}

impl WorthQueryProductWorldPorts {
    pub(super) fn memory(owner: RuntimeWorldOwner<(), (), (), (), ()>) -> Self {
        let execution_authority = owner
            .publication_port()
            .execution_authority_handle()
            .expect("the newly installed memory World is available");
        Self {
            identity: owner.owner_identity(),
            publication: owner.publication_port(),
            branch: owner.branch_port(),
            observation: owner.observation_port(),
            inspection: owner.inspection_port(),
            history_custody: owner.history_custody_port(),
            recovery: owner.recovery_port(),
            execution_authority,
            _memory_world_keepalive: Some(Arc::new(owner)),
        }
    }

    pub(super) fn attached(
        identity: RuntimeWorldOwnerIdentity,
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
    ) -> Result<Self, RuntimeWorldOwnerUnavailable> {
        let history_custody = publication.history_custody_port()?;
        let execution_authority = publication.execution_authority_handle()?;
        Ok(Self {
            identity,
            publication,
            branch,
            observation,
            inspection,
            history_custody,
            recovery,
            execution_authority,
            _memory_world_keepalive: None,
        })
    }

    pub fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.identity
    }
    pub fn publication_port(&self) -> RuntimeWorldPublicationPort<(), (), (), (), ()> {
        self.publication.clone()
    }
    pub fn branch_port(&self) -> RuntimeWorldBranchPort {
        self.branch.clone()
    }
    pub fn observation_port(&self) -> RuntimeWorldObservationPort {
        self.observation.clone()
    }
    pub fn inspection_port(&self) -> RuntimeWorldInspectionPort {
        self.inspection.clone()
    }
    pub fn history_custody_port(&self) -> RuntimeWorldHistoryCustodyPort {
        self.history_custody.clone()
    }
    pub fn recovery_port(&self) -> RuntimeWorldRecoveryPort {
        self.recovery.clone()
    }
    pub fn execution_authority(&self) -> Option<&worth_execution::ExecutionAuthority> {
        self.execution_authority.as_deref()
    }
}

impl super::WorthQueryProductRuntime {
    pub(crate) fn with_execution_placement<R>(
        &self,
        body: impl for<'owner> FnOnce(RuntimeWorldExecutionPlacement<'owner>) -> R,
    ) -> Result<R, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
        self.fields().owner.publication_port().with_execution_placement(body).map_err(|_| {
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::NoActiveExecutionScope,
            )
        })
    }
}
