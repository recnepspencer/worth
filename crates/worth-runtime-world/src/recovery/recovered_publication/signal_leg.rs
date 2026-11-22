//! A mixed original recovered with only Native's effect durable keeps its
//! planned Signal leg until World performs that leg once.
use crate::durability::AdmittedWorldPlannedSignalLeg;
use crate::publication::durable_signal::WorldDurableSignalAdvance;
use std::sync::{Arc, Mutex};
use worth_signal::facade::durability::SignalReadmittedSourcePoint;

#[derive(Debug)]
pub(crate) struct RecoveredSignalLeg {
    pub(crate) planned: AdmittedWorldPlannedSignalLeg,
    pub(crate) phase: Mutex<RecoveredSignalLegPhase>,
}

#[derive(Debug)]
pub(crate) enum RecoveredSignalLegPhase {
    /// Not begun in this process; release remains available.
    Unbegun,
    /// The one Signal leg task, driven again until Signal reports its fate.
    Driving(Arc<WorldDurableSignalAdvance>),
    /// Signal could not report the leg's fate here. It is never begun again
    /// in this process; the next reopen reads Signal's own inventory.
    ReopenRequired,
    /// Signal's effect is durable and readmitted from its own inventory.
    Durable(Arc<SignalReadmittedSourcePoint>),
}

impl RecoveredSignalLeg {
    pub(crate) fn new(planned: AdmittedWorldPlannedSignalLeg) -> Self {
        Self {
            planned,
            phase: Mutex::new(RecoveredSignalLegPhase::Unbegun),
        }
    }
    pub(crate) fn durable_effect(&self) -> Option<Arc<SignalReadmittedSourcePoint>> {
        match &*self.phase.lock().unwrap_or_else(|e| e.into_inner()) {
            RecoveredSignalLegPhase::Durable(point) => Some(point.clone()),
            RecoveredSignalLegPhase::Unbegun
            | RecoveredSignalLegPhase::Driving(_)
            | RecoveredSignalLegPhase::ReopenRequired => None,
        }
    }
    /// Release stays available only while no Signal leg has begun here.
    pub(crate) fn release_refusal(&self) -> Option<crate::recovery::RuntimeWorldRecoveryDenial> {
        use crate::recovery::RuntimeWorldRecoveryDenial as Denial;
        match &*self.phase.lock().unwrap_or_else(|e| e.into_inner()) {
            RecoveredSignalLegPhase::Unbegun => None,
            RecoveredSignalLegPhase::Driving(_) => Some(Denial::Busy),
            RecoveredSignalLegPhase::ReopenRequired => Some(Denial::ReopenRequired),
            RecoveredSignalLegPhase::Durable(_) => Some(Denial::ContinuationAlreadyStarted),
        }
    }
}
