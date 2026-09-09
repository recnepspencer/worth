#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedFrameExecutionPosture {
    ActiveFrame { frame_epoch: u64 },
    ReplacementCandidate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedFrameReuseComparator {
    ExactOrderedDependencies,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedFrameReuseMintingStage {
    FrameworkTurnExactBasis,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedFrameReuseDependency {
    ApplicationGeneration,
    HostSession,
    ExecutionPosture,
    GraphWorld,
    ActivePlan,
    AllocationTruth,
    MountedSemanticState,
    SurfaceBindingState,
    FrameRequest,
    LaneParticipation,
    HostProtocol,
    MountedFrameSchema,
    MountedStaticPaintSchema,
    MountedPresentationSchema,
    CapabilityGeneration,
    CapabilityProfile,
    PointerAffordance,
    VisualOverlay,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedFrameReuseContract {
    generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    host_session: u64,
    execution: UiMountedFrameExecutionPosture,
    graph_world: u64,
    plan_digest: u64,
    allocation_truth_revision: u64,
    mounted_semantic_revision: u64,
    surface_binding_revision: u64,
    request: super::assembly::UiMountedFrameRequestIdentity,
    lanes: super::UiMountedLaneAssembly,
    protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    visual_overlay_revision: u64,
    pointer_affordance: crate::runtime::pointer_affordance::UiPointerAffordanceReuseBasis,
}

pub(crate) struct UiMountedFrameReuseExternalBasis {
    pub generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub host_session: u64,
    pub execution: UiMountedFrameExecutionPosture,
    pub plan_digest: u64,
    pub allocation_truth_revision: u64,
    pub request: super::assembly::UiMountedFrameRequestIdentity,
    pub lanes: super::UiMountedLaneAssembly,
    pub protocol: worth_ui_host_contract::UiHostProtocolAgreement,
    pub capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub capability_profile_digest: u64,
    pub visual_overlay_revision: u64,
    pub pointer_affordance: crate::runtime::pointer_affordance::UiPointerAffordanceReuseBasis,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedFrameReuseWitness {
    contract: UiMountedFrameReuseContract,
    publication: super::UiMountedFramePublicationReceipt,
    stage: UiMountedFrameReuseMintingStage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiMountedFrameReuse {
    Exact(UiMountedFrameReuseWitness),
    ComparisonRequired(UiMountedFrameReuseContract),
}

impl UiMountedFrameReuseContract {
    pub(in crate::mounting) fn seal(
        basis: UiMountedFrameReuseExternalBasis,
        graph_world: u64,
        mounted_semantic_revision: u64,
        surface_binding_revision: u64,
    ) -> Self {
        let UiMountedFrameReuseExternalBasis {
            generation,
            host_session,
            execution,
            plan_digest,
            allocation_truth_revision,
            request,
            lanes,
            protocol,
            capability_generation,
            capability_profile_digest,
            visual_overlay_revision,
            pointer_affordance,
        } = basis;
        Self {
            generation,
            host_session,
            execution,
            graph_world,
            plan_digest,
            allocation_truth_revision,
            mounted_semantic_revision,
            surface_binding_revision,
            request,
            lanes,
            protocol,
            capability_generation,
            capability_profile_digest,
            visual_overlay_revision,
            pointer_affordance,
        }
    }

    pub const fn comparator(&self) -> UiMountedFrameReuseComparator {
        UiMountedFrameReuseComparator::ExactOrderedDependencies
    }

    pub const fn canonical_dependency_order() -> &'static [UiMountedFrameReuseDependency] {
        &[
            UiMountedFrameReuseDependency::ApplicationGeneration,
            UiMountedFrameReuseDependency::HostSession,
            UiMountedFrameReuseDependency::ExecutionPosture,
            UiMountedFrameReuseDependency::GraphWorld,
            UiMountedFrameReuseDependency::ActivePlan,
            UiMountedFrameReuseDependency::AllocationTruth,
            UiMountedFrameReuseDependency::MountedSemanticState,
            UiMountedFrameReuseDependency::SurfaceBindingState,
            UiMountedFrameReuseDependency::FrameRequest,
            UiMountedFrameReuseDependency::LaneParticipation,
            UiMountedFrameReuseDependency::HostProtocol,
            UiMountedFrameReuseDependency::MountedFrameSchema,
            UiMountedFrameReuseDependency::MountedStaticPaintSchema,
            UiMountedFrameReuseDependency::MountedPresentationSchema,
            UiMountedFrameReuseDependency::CapabilityGeneration,
            UiMountedFrameReuseDependency::CapabilityProfile,
            UiMountedFrameReuseDependency::PointerAffordance,
            UiMountedFrameReuseDependency::VisualOverlay,
        ]
    }

    pub const fn execution_posture(&self) -> UiMountedFrameExecutionPosture {
        self.execution
    }

    pub const fn graph_world(&self) -> u64 {
        self.graph_world
    }

    pub(in crate::mounting) const fn host_session(&self) -> u64 {
        self.host_session
    }

    pub(in crate::mounting) const fn capability_generation(
        &self,
    ) -> worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration {
        self.capability_generation
    }

    pub(in crate::mounting) const fn capability_profile_digest(&self) -> u64 {
        self.capability_profile_digest
    }

    pub const fn plan_digest(&self) -> u64 {
        self.plan_digest
    }

    pub const fn allocation_truth_revision(&self) -> u64 {
        self.allocation_truth_revision
    }

    pub const fn mounted_semantic_revision(&self) -> u64 {
        self.mounted_semantic_revision
    }

    pub const fn surface_binding_revision(&self) -> u64 {
        self.surface_binding_revision
    }

    pub const fn visual_overlay_revision(&self) -> u64 {
        self.visual_overlay_revision
    }

    pub(in crate::mounting) fn reconciled(
        &self,
        surface_binding_revision: u64,
        protocol: worth_ui_host_contract::UiHostProtocolAgreement,
        capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
        capability_profile_digest: u64,
    ) -> Self {
        let mut reconciled = self.clone();
        reconciled.surface_binding_revision = surface_binding_revision;
        reconciled.protocol = protocol;
        reconciled.capability_generation = capability_generation;
        reconciled.capability_profile_digest = capability_profile_digest;
        reconciled
    }
}

impl UiMountedFrameReuseWitness {
    pub(crate) fn mint(
        contract: UiMountedFrameReuseContract,
        publication: super::UiMountedFramePublicationReceipt,
    ) -> Self {
        Self {
            contract,
            publication,
            stage: UiMountedFrameReuseMintingStage::FrameworkTurnExactBasis,
        }
    }

    pub fn contract(&self) -> &UiMountedFrameReuseContract {
        &self.contract
    }

    pub fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.publication.frame()
    }

    pub fn minting_stage(&self) -> UiMountedFrameReuseMintingStage {
        self.stage
    }

    pub(crate) fn publication(&self) -> &super::UiMountedFramePublicationReceipt {
        &self.publication
    }
}
