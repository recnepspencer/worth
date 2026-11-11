use worth_foundational::CanonicalDigestId;
use worth_store_io_scheduler::{
    foreground_reservation::ForegroundIoLaneKind, BackgroundIoPressureClass,
    LatencyEnvelopeAssessmentStatus,
};
use worth_store_physical_backend::{BackendTargetProfile, CapabilityEvidenceClass};

use crate::{
    IoPressureHarnessEvidenceDenial, PhysicalDriverKind, PhysicalScenarioFaultKind,
    PhysicalSimulationProfile, SimulationReplayBundle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhysicalFaultEvidenceClass {
    Simulated,
    InjectedProductionBoundary,
    BackendEmulated,
    ObservedHost,
    CertifiedBackend,
    ExternallyGuaranteed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IoPressureFaultKind {
    BackendLatencyInjection,
    QueueDepthSaturation,
    BandwidthThrottle,
    DelayedSync,
    PageCachePressure,
    BackgroundPacingLateYield,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IoPressureHarnessSecureIoPosture {
    ScopePreserving,
    UnsupportedTypedDenial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IoPressureEvidenceMaturity {
    SimulatedOnly,
    ProductionBoundaryInjected,
    BackendEmulated,
    HostObserved,
    BackendCertified,
    ExternalGuarantee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoPressureBackendSafetyQualificationDenial {
    SimulatedBackendSuccess,
    InjectedBoundaryOnly,
    BackendEmulatedOnly,
    ObservedHostOnly,
    BackendEvidenceClassTooWeak {
        required: CapabilityEvidenceClass,
        actual: CapabilityEvidenceClass,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoPressureHarnessScenario {
    backend_profile: BackendTargetProfile,
    backend_evidence_class: CapabilityEvidenceClass,
    foreground_lane: ForegroundIoLaneKind,
    background_pressure: BackgroundIoPressureClass,
    secure_io_posture: IoPressureHarnessSecureIoPosture,
    fault_kind: IoPressureFaultKind,
    fault_evidence_class: PhysicalFaultEvidenceClass,
    expected_status: LatencyEnvelopeAssessmentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IoPressureHarnessEvidence {
    scenario: IoPressureHarnessScenario,
    driver: PhysicalDriverKind,
    fault_phase: PhysicalScenarioFaultKind,
    maturity: IoPressureEvidenceMaturity,
    replay_profile: PhysicalSimulationProfile,
    replay_identity: CanonicalDigestId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoPressureOracleObservation {
    foreground_lane: ForegroundIoLaneKind,
    background_pressure: BackgroundIoPressureClass,
    secure_io_posture: IoPressureHarnessSecureIoPosture,
    fault_kind: IoPressureFaultKind,
    fault_evidence_class: PhysicalFaultEvidenceClass,
    envelope_status: LatencyEnvelopeAssessmentStatus,
    attribution_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealBackendSafetyQualification {
    backend_profile: BackendTargetProfile,
    evidence_class: PhysicalFaultEvidenceClass,
}

impl IoPressureHarnessScenario {
    pub const fn deterministic_read_under_repair_pressure() -> Self {
        Self {
            backend_profile: BackendTargetProfile::PosixFileFsyncDirSync,
            backend_evidence_class: CapabilityEvidenceClass::CertifiedBackendProfile,
            foreground_lane: ForegroundIoLaneKind::PointRead,
            background_pressure: BackgroundIoPressureClass::RepairScan,
            secure_io_posture: IoPressureHarnessSecureIoPosture::ScopePreserving,
            fault_kind: IoPressureFaultKind::BackendLatencyInjection,
            fault_evidence_class: PhysicalFaultEvidenceClass::InjectedProductionBoundary,
            expected_status: LatencyEnvelopeAssessmentStatus::Held,
        }
    }

    pub const fn with_fault_evidence_class(
        mut self,
        fault_evidence_class: PhysicalFaultEvidenceClass,
    ) -> Self {
        self.fault_evidence_class = fault_evidence_class;
        self
    }

    pub const fn with_foreground_lane(mut self, foreground_lane: ForegroundIoLaneKind) -> Self {
        self.foreground_lane = foreground_lane;
        self
    }

    pub const fn with_background_pressure(
        mut self,
        background_pressure: BackgroundIoPressureClass,
    ) -> Self {
        self.background_pressure = background_pressure;
        self
    }

    pub const fn with_backend_evidence_class(
        mut self,
        backend_evidence_class: CapabilityEvidenceClass,
    ) -> Self {
        self.backend_evidence_class = backend_evidence_class;
        self
    }

    pub const fn with_backend_profile(mut self, backend_profile: BackendTargetProfile) -> Self {
        self.backend_profile = backend_profile;
        self
    }

    pub const fn with_fault_kind(mut self, fault_kind: IoPressureFaultKind) -> Self {
        self.fault_kind = fault_kind;
        self
    }

    pub const fn with_expected_status(
        mut self,
        expected_status: LatencyEnvelopeAssessmentStatus,
    ) -> Self {
        self.expected_status = expected_status;
        self
    }

    pub const fn backend_profile(&self) -> BackendTargetProfile {
        self.backend_profile
    }

    pub const fn backend_evidence_class(&self) -> CapabilityEvidenceClass {
        self.backend_evidence_class
    }

    pub const fn foreground_lane(&self) -> ForegroundIoLaneKind {
        self.foreground_lane
    }

    pub const fn background_pressure(&self) -> BackgroundIoPressureClass {
        self.background_pressure
    }

    pub const fn secure_io_posture(&self) -> IoPressureHarnessSecureIoPosture {
        self.secure_io_posture
    }

    pub const fn fault_kind(&self) -> IoPressureFaultKind {
        self.fault_kind
    }

    pub const fn fault_evidence_class(&self) -> PhysicalFaultEvidenceClass {
        self.fault_evidence_class
    }

    pub const fn expected_status(&self) -> LatencyEnvelopeAssessmentStatus {
        self.expected_status
    }
}

impl IoPressureHarnessEvidence {
    pub fn from_replay_bundle(
        scenario: IoPressureHarnessScenario,
        replay: &SimulationReplayBundle,
    ) -> Result<Self, IoPressureHarnessEvidenceDenial> {
        super::replay::require_io_pressure_replay_bundle(&scenario, replay)?;
        Ok(Self::from_executed_replay(
            scenario,
            replay.plan().profile(),
            *replay.replay_basis_identity().digest_bytes(),
        ))
    }

    fn from_executed_replay(
        scenario: IoPressureHarnessScenario,
        replay_profile: PhysicalSimulationProfile,
        replay_identity: [u8; 32],
    ) -> Self {
        let fault_phase = super::vocabulary::fault_phase_for_pressure_fault(scenario.fault_kind);
        Self {
            maturity: super::vocabulary::maturity_for_fault_evidence_class(
                scenario.fault_evidence_class,
            ),
            scenario,
            driver: PhysicalDriverKind::IoPressureBoundary,
            fault_phase,
            replay_profile,
            replay_identity: CanonicalDigestId::new(replay_identity),
        }
    }

    pub const fn scenario(&self) -> &IoPressureHarnessScenario {
        &self.scenario
    }

    pub const fn driver(&self) -> PhysicalDriverKind {
        self.driver
    }

    pub const fn fault_phase(&self) -> PhysicalScenarioFaultKind {
        self.fault_phase
    }

    pub const fn maturity(&self) -> IoPressureEvidenceMaturity {
        self.maturity
    }

    pub const fn replay_identity(&self) -> &[u8; 32] {
        self.replay_identity.bytes()
    }

    pub const fn replay_profile(&self) -> PhysicalSimulationProfile {
        self.replay_profile
    }

    pub fn require_real_backend_safety(
        &self,
    ) -> Result<RealBackendSafetyQualification, IoPressureBackendSafetyQualificationDenial> {
        require_backend_evidence_class(self.scenario.backend_evidence_class)?;
        match self.scenario.fault_evidence_class {
            PhysicalFaultEvidenceClass::CertifiedBackend
            | PhysicalFaultEvidenceClass::ExternallyGuaranteed => {
                Ok(RealBackendSafetyQualification {
                    backend_profile: self.scenario.backend_profile,
                    evidence_class: self.scenario.fault_evidence_class,
                })
            }
            PhysicalFaultEvidenceClass::Simulated => {
                Err(IoPressureBackendSafetyQualificationDenial::SimulatedBackendSuccess)
            }
            PhysicalFaultEvidenceClass::InjectedProductionBoundary => {
                Err(IoPressureBackendSafetyQualificationDenial::InjectedBoundaryOnly)
            }
            PhysicalFaultEvidenceClass::BackendEmulated => {
                Err(IoPressureBackendSafetyQualificationDenial::BackendEmulatedOnly)
            }
            PhysicalFaultEvidenceClass::ObservedHost => {
                Err(IoPressureBackendSafetyQualificationDenial::ObservedHostOnly)
            }
        }
    }
}

impl RealBackendSafetyQualification {
    pub const fn backend_profile(&self) -> BackendTargetProfile {
        self.backend_profile
    }

    pub const fn evidence_class(&self) -> PhysicalFaultEvidenceClass {
        self.evidence_class
    }
}

impl IoPressureOracleObservation {
    pub(crate) fn from_executed_pressure(
        scenario: &IoPressureHarnessScenario,
        counters: crate::IoPressureExecutionCounters,
        envelope_status: LatencyEnvelopeAssessmentStatus,
    ) -> Self {
        Self {
            foreground_lane: scenario.foreground_lane,
            background_pressure: scenario.background_pressure,
            secure_io_posture: scenario.secure_io_posture,
            fault_kind: scenario.fault_kind,
            fault_evidence_class: scenario.fault_evidence_class,
            envelope_status,
            attribution_complete: counters.queue_depth() > 0
                && counters.interference_events() > 0
                && counters.allocation_bytes() > 0,
        }
    }

    pub const fn envelope_status(self) -> LatencyEnvelopeAssessmentStatus {
        self.envelope_status
    }

    pub const fn attribution_complete(self) -> bool {
        self.attribution_complete
    }

    pub fn matches_scenario(self, scenario: &IoPressureHarnessScenario) -> bool {
        self.foreground_lane == scenario.foreground_lane
            && self.background_pressure == scenario.background_pressure
            && self.secure_io_posture == scenario.secure_io_posture
            && self.fault_kind == scenario.fault_kind
            && self.fault_evidence_class == scenario.fault_evidence_class
            && self.envelope_status == scenario.expected_status
            && self.attribution_complete
    }
}

fn require_backend_evidence_class(
    actual: CapabilityEvidenceClass,
) -> Result<(), IoPressureBackendSafetyQualificationDenial> {
    if actual == CapabilityEvidenceClass::CertifiedBackendProfile {
        Ok(())
    } else {
        Err(
            IoPressureBackendSafetyQualificationDenial::BackendEvidenceClassTooWeak {
                required: CapabilityEvidenceClass::CertifiedBackendProfile,
                actual,
            },
        )
    }
}
