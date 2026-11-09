//! Fresh Bridge installation against each actual recovered Signal branch.
use super::{
    definition::BridgeReadmittedDefinitionSource, layout::Layout,
    BridgeAuthoredDependencyDescription, BridgeAuthoredSection, BridgeAuthoredSectionDenial,
};
use crate::conditional_execution::{
    BridgeConditionalContract, BridgeConditionalDenial, BridgeConditionalLocation,
    BridgeConditionalProviderSet,
};
use crate::facade::RuntimeBridge;
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BridgeSemanticClaimReference, DurableSignalBranchIdentity};
use worth_signal::facade::durability::{
    SignalAuthoredBranchInstallation, SignalAuthoredInstallationRefused,
    SignalAuthoredOwnerInstallation,
};
pub(in crate::conditional_execution) mod backing;
mod claims;
mod definitions;

/// Fresh program bindings supply live authority; durable bytes supply meaning.
pub trait BridgeAuthoredInstallationBindings {
    fn dependency(
        &mut self,
        branch: DurableSignalBranchIdentity,
        reference: BridgeSemanticClaimReference,
        description: &BridgeAuthoredDependencyDescription,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::correspondence::BridgeSemanticDependencyCandidate,
        BridgeAuthoredReadmissionDenial,
    >;
    fn providers(
        &mut self,
        branch: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial>;
}

#[derive(Debug)]
pub enum BridgeAuthoredReadmissionDenial {
    Section(Box<BridgeAuthoredSectionDenial>),
    Signal(Box<worth_signal::facade::durability::SignalOwnerReadmissionDenial>),
    Source(Box<worth_signal::facade::durability::SignalDurableSourceDenial>),
    Conditional(Box<BridgeConditionalDenial>),
    Correspondence(crate::correspondence::BridgeCorrespondenceDenialKind),
    SemanticBindingMismatch(BridgeSemanticClaimReference),
    AliasedBindings {
        first: BridgeSemanticClaimReference,
        second: BridgeSemanticClaimReference,
    },
    SourceProfileMismatch,
    MissingBridgeSection,
    ConditionalDefinitionMismatch,
    UnsupportedProgramFamily,
    FreshBindingUnavailable,
    Memory(worth_execution::MemoryLimitDenial),
    SignalNode(Box<worth_signal::facade::SignalError>),
}
impl BridgeAuthoredReadmissionDenial {
    pub(in crate::conditional_execution) fn section(cause: BridgeAuthoredSectionDenial) -> Self {
        Self::Section(Box::new(cause))
    }
    pub(in crate::conditional_execution) fn signal(
        cause: worth_signal::facade::durability::SignalOwnerReadmissionDenial,
    ) -> Self {
        Self::Signal(Box::new(cause))
    }
    pub(in crate::conditional_execution) fn source(
        cause: worth_signal::facade::durability::SignalDurableSourceDenial,
    ) -> Self {
        Self::Source(Box::new(cause))
    }
    pub(in crate::conditional_execution) fn conditional(cause: BridgeConditionalDenial) -> Self {
        Self::Conditional(Box::new(cause))
    }
}
impl From<BridgeAuthoredSectionDenial> for BridgeAuthoredReadmissionDenial {
    fn from(value: BridgeAuthoredSectionDenial) -> Self {
        Self::section(value)
    }
}
pub(in crate::conditional_execution) struct PreparedDefinition {
    pub(in crate::conditional_execution) source:Arc<BridgeReadmittedDefinitionSource>,
    pub(in crate::conditional_execution) signal:worth_signal::facade::InstalledSignalConditionalContract,
    pub(in crate::conditional_execution) correspondences:Arc<[crate::correspondence::BridgeInstalledSemanticCorrespondence]>,
    pub(in crate::conditional_execution) plan:Option<crate::conditional_execution::semantic_observation_plan::BridgeConditionalSemanticObservationPlan>,
    pub(in crate::conditional_execution) providers:BridgeConditionalProviderSet,
    pub(in crate::conditional_execution) provider_admission:crate::conditional_execution::provider_admission::BridgeConditionalProviderAdmission,
}
pub(in crate::conditional_execution) struct Program {
    pub(in crate::conditional_execution) bridge: RuntimeBridge,
    pub(in crate::conditional_execution) installation:
        worth_proof::BridgeProgramInstallationBinding,
    pub(in crate::conditional_execution) _memory:
        Option<worth_execution::ExecutionMemoryReservation>,
}
impl Program {
    pub(in crate::conditional_execution) fn new(
        bridge: RuntimeBridge,
        installation: worth_proof::BridgeProgramInstallationBinding,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, BridgeAuthoredReadmissionDenial> {
        let bytes =
            crate::conditional_execution::retention::arc_charge::<Self>().map_err(|_| {
                BridgeAuthoredReadmissionDenial::section(BridgeAuthoredSectionDenial::SizeLimit)
            })?;
        let memory = execution.reserve_retained_memory(bytes).map_err(|cause| {
            BridgeAuthoredReadmissionDenial::section(BridgeAuthoredSectionDenial::Request(cause))
        })?;
        Ok(Arc::new(Self {
            bridge,
            installation,
            _memory: memory,
        }))
    }
}
pub(in crate::conditional_execution) struct PreparedBranch {
    pub(in crate::conditional_execution) graph_lifecycle:
        worth_signal::facade::SignalGraphLifecycleProbe,
    pub(in crate::conditional_execution) graph_instance_id: u64,
    pub(in crate::conditional_execution) program: Arc<Program>,
    pub(in crate::conditional_execution) dependencies:
        crate::correspondence::AdmittedSemanticDependencyRegistry,
    pub(in crate::conditional_execution) allocations:
        crate::correspondence::SharedCorrespondenceAllocationRegistry,
    pub(in crate::conditional_execution) section: BridgeAuthoredSection,
    pub(in crate::conditional_execution) definitions: Vec<PreparedDefinition>,
    pub(in crate::conditional_execution) backing: Arc<backing::Backing>,
}
pub(in crate::conditional_execution) struct Installation<'a> {
    pub(in crate::conditional_execution) program: Arc<Program>,
    pub(in crate::conditional_execution) bindings: &'a mut dyn BridgeAuthoredInstallationBindings,
    pub(in crate::conditional_execution) branches:
        BTreeMap<DurableSignalBranchIdentity, Box<PreparedBranch>>,
    pub(in crate::conditional_execution) refusal: Option<BridgeAuthoredReadmissionDenial>,
}
impl Installation<'_> {
    pub(in crate::conditional_execution) fn prepare(
        &mut self,
        branch: SignalAuthoredBranchInstallation<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<PreparedBranch>, BridgeAuthoredReadmissionDenial> {
        use BridgeAuthoredReadmissionDenial as D;
        if branch.bridge_section().is_empty() {
            return Err(D::MissingBridgeSection);
        }
        let identity = branch.branch();
        let backing = backing::Backing::new(execution)?;
        backing.grow(execution, std::mem::size_of::<PreparedBranch>() as u64)?;
        let section =
            BridgeAuthoredSection::readmit_bytes(execution, branch.bridge_section(), identity)?;
        let layout = Layout::read(section.as_signal_section().as_bytes())?;
        let actual = self
            .program
            .bridge
            .authoritative_source_profile
            .as_ref()
            .map(|profile| profile.adapter_semantic_identity());
        if actual != layout.adapter {
            return Err(D::SourceProfileMismatch);
        }
        let registrations =
            claims::registrations(&layout, &branch, self.bindings, &backing, execution)?;
        let mut allocations = crate::correspondence::CorrespondenceAllocationRegistry::default();
        claims::allocations(
            &mut allocations,
            &layout,
            branch.graph(),
            &backing,
            execution,
        )?;
        let allocation_bytes = crate::conditional_execution::retention::arc_charge::<
            std::sync::RwLock<crate::correspondence::CorrespondenceAllocationRegistry>,
        >()
        .map_err(|_| D::section(BridgeAuthoredSectionDenial::SizeLimit))?;
        backing.grow(execution, allocation_bytes)?;
        let allocations = Arc::new(std::sync::RwLock::new(allocations));
        let dependencies = crate::correspondence::AdmittedSemanticDependencyRegistry::freeze(
            registrations.values().cloned().collect(),
        )
        .map_err(|_| D::ConditionalDefinitionMismatch)?;
        let definitions = definitions::prepare(
            &self.program.bridge,
            &allocations,
            &layout,
            &branch,
            &registrations,
            self.bindings,
            &backing,
            execution,
        )?;
        backing.grow(
            execution,
            crate::conditional_execution::retention::btree_charge::<
                DurableSignalBranchIdentity,
                Box<PreparedBranch>,
            >(1)
            .map_err(|_| D::section(BridgeAuthoredSectionDenial::SizeLimit))?,
        )?;
        Ok(Box::new(PreparedBranch {
            graph_lifecycle: branch.graph().lifecycle_probe(),
            graph_instance_id: branch
                .graph()
                .installed_graph_capability()
                .graph_instance_id(),
            program: Arc::clone(&self.program),
            dependencies,
            allocations,
            section,
            definitions,
            backing,
        }))
    }
}
impl SignalAuthoredOwnerInstallation for Installation<'_> {
    fn readmit_branch(
        &mut self,
        branch: SignalAuthoredBranchInstallation<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalAuthoredInstallationRefused> {
        let identity = branch.branch();
        match self.prepare(branch, execution) {
            Ok(prepared) => {
                self.branches.insert(identity, prepared);
                Ok(())
            }
            Err(reason) => {
                self.refusal = Some(reason);
                Err(SignalAuthoredInstallationRefused)
            }
        }
    }
}
