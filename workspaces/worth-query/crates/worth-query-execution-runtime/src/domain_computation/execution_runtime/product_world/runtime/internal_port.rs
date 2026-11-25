//! Crate-boundary internal port for `WorthQueryProductRuntime` fields and core
//! runtime operations. Never exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::{
    InstalledCorrectionBinding, WorthQueryCompiledApplicationOperationContracts,
    WorthQueryInstalledAftermathContract,
};
use worth_relational::facade::branch::RelationalBranchLifecyclePort;
use worth_runtime_bridge::facade::{
    BridgeReadmittedAuthoredAssembly, RuntimeBridgeRelationalSource,
};
use worth_runtime_world::facade::{
    ObservedProductBranches, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, RecoveredRuntimeWorldRootAuthority,
    RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchPort, RuntimeWorldExecutionPlacement,
    RuntimeWorldInspectionPort, RuntimeWorldObservationPort, RuntimeWorldOwnerIdentity,
    RuntimeWorldPublicationPort, RuntimeWorldRecoveryPort, RuntimeWorldServiceDenial,
    WorldInterimCheckpointPort,
};

use super::super::activation::WorthQueryProductActivationRegistry;
use super::super::durable_admission::{
    DirectoryCorrectionShapeAdmission, WorthQueryDurableStateAdmissionDenial,
};
use super::super::owner_cleanup::WorthQueryProductBranchOwnerCleanupRegistry;
use super::super::publication_binding::WorthQueryProductPublicationBinding;
use super::super::read_consult::{NamedRead, NamedReadRegion};
use super::super::signal_lifecycle::SignalLifecycleServices;
use super::super::world_ports::WorthQueryProductWorldPorts;
use super::super::{
    WorthQueryProductRelationalInstallation, WorthQueryProductRuntimeInstallationDenial,
    WorthQueryProductWorldClock, WorthQueryProductWorldResources,
};
use super::{WorthQueryProductRootIdentity, WorthQueryProductRuntime};
use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
};
use crate::ports::HomeMaintenanceAfterAdvancement;

pub trait ProductRuntimeInternalPort: Sized {
    fn owner_ref(&self) -> &WorthQueryProductWorldPorts;
    fn source(&self) -> &RuntimeBridgeRelationalSource;
    fn source_slot_mut(&mut self) -> &mut Option<RuntimeBridgeRelationalSource>;
    fn home_maintenance(&self) -> Option<&dyn HomeMaintenanceAfterAdvancement>;
    fn home_maintenance_slot_ref(&self) -> &Option<Arc<dyn HomeMaintenanceAfterAdvancement>>;
    fn home_maintenance_slot_mut(
        &mut self,
    ) -> &mut Option<Arc<dyn HomeMaintenanceAfterAdvancement>>;
    fn activations_ref(&self) -> &Arc<WorthQueryProductActivationRegistry>;
    fn clock_ref(&self) -> &WorthQueryProductWorldClock;
    fn owner_cleanup_ref(&self) -> &WorthQueryProductBranchOwnerCleanupRegistry;
    fn default_branch(&self) -> &ProductBranchIdentity;
    fn default_occurrence(&self) -> ProductBranchIncarnation;
    fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity>;
    fn recovered_root_authority_ref(&self) -> Option<&Arc<RecoveredRuntimeWorldRootAuthority>>;

    #[allow(clippy::too_many_arguments)]
    fn from_parts(
        owner: WorthQueryProductWorldPorts,
        source: RuntimeBridgeRelationalSource,
        activations: WorthQueryProductActivationRegistry,
        clock: WorthQueryProductWorldClock,
        relational_lifecycle: RelationalBranchLifecyclePort,
        signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
        signal_lifecycle: SignalLifecycleServices,
        owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
        default_branch: ProductBranchIdentity,
        default_occurrence: ProductBranchIncarnation,
        recovered_root_authority: Option<Arc<RecoveredRuntimeWorldRootAuthority>>,
    ) -> Self;

    #[allow(clippy::too_many_arguments)]
    fn attach_home_world_ports(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &BridgeReadmittedAuthoredAssembly,
        resources: WorthQueryProductWorldResources,
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
        root: ProductBranchObservation,
        maintenance: Arc<dyn HomeMaintenanceAfterAdvancement>,
        branch_program_source_owner: Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        branches: &ObservedProductBranches,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial>;

    fn reserve_public_branch_ordinal(
        &self,
    ) -> Result<u64, RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>>;

    fn publication_binding(
        &self,
        observation: ProductBranchObservation,
    ) -> WorthQueryProductPublicationBinding;

    fn interim_checkpoint_port(&self) -> Option<WorldInterimCheckpointPort>;

    fn with_execution_placement<R>(
        &self,
        body: impl for<'owner> FnOnce(RuntimeWorldExecutionPlacement<'owner>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial>;

    fn is_directory_home(&self) -> bool;

    fn admit_owner_incarnation(
        &self,
        presented: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial>;

    fn enter_named_read(&self, name: NamedRead) -> NamedReadRegion;
    fn enter_authority_revalidation(&self) -> NamedReadRegion;

    fn admit_directory_original_shape(
        &self,
        contracts: &WorthQueryCompiledApplicationOperationContracts,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<DirectoryCorrectionShapeAdmission, WorthQueryDurableStateAdmissionDenial>;

    fn admit_directory_correction_shape(
        &self,
        aftermath: Option<&WorthQueryInstalledAftermathContract>,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial>;

    fn with_request_advancement_retaining<R>(
        &self,
        request: &WorthQueryRequestScope,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial>;

    fn with_funded_product_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial>;
}

impl ProductRuntimeInternalPort for WorthQueryProductRuntime {
    fn owner_ref(&self) -> &WorthQueryProductWorldPorts {
        &self.fields().owner
    }

    fn source(&self) -> &RuntimeBridgeRelationalSource {
        Self::source(self)
    }

    fn source_slot_mut(&mut self) -> &mut Option<RuntimeBridgeRelationalSource> {
        &mut self.source
    }

    fn home_maintenance(&self) -> Option<&dyn HomeMaintenanceAfterAdvancement> {
        Self::home_maintenance(self)
    }

    fn home_maintenance_slot_ref(&self) -> &Option<Arc<dyn HomeMaintenanceAfterAdvancement>> {
        &self.home_maintenance
    }

    fn home_maintenance_slot_mut(
        &mut self,
    ) -> &mut Option<Arc<dyn HomeMaintenanceAfterAdvancement>> {
        &mut self.home_maintenance
    }

    fn activations_ref(&self) -> &Arc<WorthQueryProductActivationRegistry> {
        &self.fields().activations
    }

    fn clock_ref(&self) -> &WorthQueryProductWorldClock {
        &self.fields().clock
    }

    fn owner_cleanup_ref(&self) -> &WorthQueryProductBranchOwnerCleanupRegistry {
        &self.owner_cleanup
    }

    fn default_branch(&self) -> &ProductBranchIdentity {
        Self::default_branch(self)
    }

    fn default_occurrence(&self) -> ProductBranchIncarnation {
        self.fields().default_occurrence
    }

    fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Self::root_identity(self)
    }

    fn recovered_root_authority_ref(&self) -> Option<&Arc<RecoveredRuntimeWorldRootAuthority>> {
        self.fields().recovered_root_authority.as_ref()
    }

    fn from_parts(
        owner: WorthQueryProductWorldPorts,
        source: RuntimeBridgeRelationalSource,
        activations: WorthQueryProductActivationRegistry,
        clock: WorthQueryProductWorldClock,
        relational_lifecycle: RelationalBranchLifecyclePort,
        signal_basis: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
        signal_lifecycle: SignalLifecycleServices,
        owner_cleanup: WorthQueryProductBranchOwnerCleanupRegistry,
        default_branch: ProductBranchIdentity,
        default_occurrence: ProductBranchIncarnation,
        recovered_root_authority: Option<Arc<RecoveredRuntimeWorldRootAuthority>>,
    ) -> Self {
        Self::from_parts(
            owner,
            source,
            activations,
            clock,
            relational_lifecycle,
            signal_basis,
            signal_lifecycle,
            owner_cleanup,
            default_branch,
            default_occurrence,
            recovered_root_authority,
        )
    }

    fn attach_home_world_ports(
        relational: WorthQueryProductRelationalInstallation,
        bridge: &BridgeReadmittedAuthoredAssembly,
        resources: WorthQueryProductWorldResources,
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        branch: RuntimeWorldBranchPort,
        observation: RuntimeWorldObservationPort,
        inspection: RuntimeWorldInspectionPort,
        recovery: RuntimeWorldRecoveryPort,
        root: ProductBranchObservation,
        maintenance: Arc<dyn HomeMaintenanceAfterAdvancement>,
        branch_program_source_owner: Arc<worth_proof::InstalledBranchProgramSourceOwner>,
        branches: &ObservedProductBranches,
    ) -> Result<Self, WorthQueryProductRuntimeInstallationDenial> {
        Self::attach_home_world_ports(
            relational,
            bridge,
            resources,
            publication,
            branch,
            observation,
            inspection,
            recovery,
            root,
            maintenance,
            branch_program_source_owner,
            branches,
        )
    }

    fn reserve_public_branch_ordinal(
        &self,
    ) -> Result<u64, RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>> {
        Self::reserve_public_branch_ordinal(self)
    }

    fn publication_binding(
        &self,
        observation: ProductBranchObservation,
    ) -> WorthQueryProductPublicationBinding {
        Self::publication_binding(self, observation)
    }

    fn interim_checkpoint_port(&self) -> Option<WorldInterimCheckpointPort> {
        Self::interim_checkpoint_port(self)
    }

    fn with_execution_placement<R>(
        &self,
        body: impl for<'owner> FnOnce(RuntimeWorldExecutionPlacement<'owner>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        Self::with_execution_placement(self, body)
    }

    fn is_directory_home(&self) -> bool {
        Self::is_directory_home(self)
    }

    fn admit_owner_incarnation(
        &self,
        presented: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial> {
        Self::admit_owner_incarnation(self, presented)
    }

    fn enter_named_read(&self, name: NamedRead) -> NamedReadRegion {
        Self::enter_named_read(self, name)
    }

    fn enter_authority_revalidation(&self) -> NamedReadRegion {
        Self::enter_authority_revalidation(self)
    }

    fn admit_directory_original_shape(
        &self,
        contracts: &WorthQueryCompiledApplicationOperationContracts,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<DirectoryCorrectionShapeAdmission, WorthQueryDurableStateAdmissionDenial> {
        Self::admit_directory_original_shape(self, contracts, correction)
    }

    fn admit_directory_correction_shape(
        &self,
        aftermath: Option<&WorthQueryInstalledAftermathContract>,
        correction: Option<&InstalledCorrectionBinding>,
    ) -> Result<(), WorthQueryDurableStateAdmissionDenial> {
        Self::admit_directory_correction_shape(self, aftermath, correction)
    }

    fn with_request_advancement_retaining<R>(
        &self,
        request: &WorthQueryRequestScope,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        Self::with_request_advancement_retaining(self, request, retain_returned_outcome, body)
    }

    fn with_funded_product_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        Self::with_funded_product_advancement(
            self,
            allocation_policy,
            cancellation,
            deadline,
            retain_returned_outcome,
            body,
        )
    }
}
