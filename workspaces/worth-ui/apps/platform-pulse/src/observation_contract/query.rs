use serde::{Deserialize, Serialize};
use worth_ui::facade::query_binding::{
    UiProjectionObservation, UiQueryObservationReportingProjection,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseQueryProjectionEvidence {
    projection_identity: String,
    owner_order: u64,
    posture: PlatformPulseQueryProjectionPosture,
    native_value: Option<String>,
    query_world: String,
    binding: String,
    source_generation: String,
    result_generation: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseQueryProjectionPosture {
    Pending,
    Current,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseQueryWatcherShutdownEvidence {
    worker_joined: bool,
    pending_observation_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformPulseQueryShutdownEvidence {
    watcher: PlatformPulseQueryWatcherShutdownEvidence,
    owner_terminal: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseQueryProjectionPublished {
    projection: PlatformPulseQueryProjectionEvidence,
    frame: super::lifecycle::PlatformPulseMountedFrameObservation,
}

impl PlatformPulseQueryWatcherShutdownEvidence {
    pub fn new(worker_joined: bool, pending_observation_count: u64) -> Self {
        Self {
            worker_joined,
            pending_observation_count,
        }
    }

    pub fn worker_joined(self) -> bool {
        self.worker_joined
    }

    pub fn pending_observation_count(self) -> u64 {
        self.pending_observation_count
    }
}

impl PlatformPulseQueryShutdownEvidence {
    pub fn new(watcher: PlatformPulseQueryWatcherShutdownEvidence, owner_terminal: bool) -> Self {
        Self {
            watcher,
            owner_terminal,
        }
    }

    pub fn watcher(self) -> PlatformPulseQueryWatcherShutdownEvidence {
        self.watcher
    }

    pub fn owner_terminal(self) -> bool {
        self.owner_terminal
    }
}

impl PlatformPulseQueryProjectionEvidence {
    pub fn from_observation(
        observation: &UiProjectionObservation,
    ) -> Result<Self, super::projection::PlatformPulseLifecycleObservationProjectionDenial> {
        let UiProjectionObservation::ApplicationScalar(scalar) = observation else {
            return Err(
                super::projection::PlatformPulseLifecycleObservationProjectionDenial::
                    QueryProjectionUnsupported,
            );
        };
        let fact = scalar.fact();
        let (posture, native_value) = if fact.value().revision == 0 {
            (PlatformPulseQueryProjectionPosture::Pending, None)
        } else {
            (
                PlatformPulseQueryProjectionPosture::Current,
                Some(fact.value().status.clone()),
            )
        };
        let reporting = UiQueryObservationReportingProjection::from_observation(observation);
        Ok(Self {
            projection_identity: observation.projection_identity().as_str().to_owned(),
            owner_order: observation.owner_order(),
            posture,
            native_value,
            query_world: reporting.query_world().to_owned(),
            binding: reporting.binding().to_owned(),
            source_generation: reporting.source_generation().to_owned(),
            result_generation: reporting.result_generation().to_owned(),
        })
    }

    pub fn projection_identity(&self) -> &str {
        &self.projection_identity
    }

    pub fn owner_order(&self) -> u64 {
        self.owner_order
    }

    pub fn posture(&self) -> PlatformPulseQueryProjectionPosture {
        self.posture
    }

    pub fn native_value(&self) -> Option<&str> {
        self.native_value.as_deref()
    }

    pub fn query_world(&self) -> &str {
        &self.query_world
    }

    pub fn binding(&self) -> &str {
        &self.binding
    }

    pub fn source_generation(&self) -> &str {
        &self.source_generation
    }

    pub fn result_generation(&self) -> &str {
        &self.result_generation
    }
}

impl PlatformPulseQueryProjectionPublished {
    pub(super) fn new(
        projection: PlatformPulseQueryProjectionEvidence,
        frame: super::lifecycle::PlatformPulseMountedFrameObservation,
    ) -> Self {
        Self { projection, frame }
    }

    pub fn projection(&self) -> &PlatformPulseQueryProjectionEvidence {
        &self.projection
    }

    pub fn frame(&self) -> super::lifecycle::PlatformPulseMountedFrameObservation {
        self.frame
    }
}
