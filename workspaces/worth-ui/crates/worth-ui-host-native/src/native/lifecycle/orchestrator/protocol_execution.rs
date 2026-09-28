use crate::native::presentation::port::orchestrator::UiNativePresentationStageFailure;
use crate::native::presentation::{UiNativePresentationFault, UiNativePresentationRecoveryClass};
use crate::native::{UiNativePresentationEffectPhase, UiNativeRecoveryCause};

use super::{UiNativeLifecycleDirective, UiNativeSurfaceBasisTransition};
use crate::native::lifecycle::protocol_world::presentation::{
    UiProtocolCloseControl, UiProtocolPresentationPort,
};
use crate::native::lifecycle::protocol_world::resources::UiProtocolResources;
use crate::native::lifecycle::protocol_world::schema::{
    UiNativeLifecycleProtocolReport, UiNativeLifecycleProtocolSchedule,
    UiNativeProtocolCloseDisposition, UiNativeProtocolClosePoint, UiNativeProtocolNextAction,
    UiNativeProtocolPredecessor, UiNativeProtocolReadback, UiNativeProtocolResourceCensus,
    UiNativeProtocolSurfaceTransition,
};

pub(super) struct UiProtocolExecution {
    schedule: UiNativeLifecycleProtocolSchedule,
    completed_stages: Vec<UiNativePresentationEffectPhase>,
    resources: UiProtocolResources,
    lifecycle: super::UiNativeLifecycleOrchestrator,
    peak: UiNativeProtocolResourceCensus,
    device_generation: u64,
    surface_generation: u64,
    reconstructed_bindings: usize,
    pending_readback_observed: bool,
}

impl UiProtocolExecution {
    pub(super) fn new(schedule: UiNativeLifecycleProtocolSchedule) -> Self {
        let resources = UiProtocolResources::new(schedule);
        let mut peak = UiNativeProtocolResourceCensus::from_registry(resources.peak(), 0);
        peak.queued_readiness = resources.queued_readiness_count();
        Self {
            schedule,
            completed_stages: Vec::new(),
            resources,
            lifecycle: super::UiNativeLifecycleOrchestrator::new(),
            peak,
            device_generation: 1,
            surface_generation: 1,
            reconstructed_bindings: 0,
            pending_readback_observed: false,
        }
    }

    pub(super) fn run(mut self) -> UiNativeLifecycleProtocolReport {
        if let Some(transition) = self.schedule.scheduled_surface_transition() {
            return self.handle_surface_transition(transition);
        }
        if self.schedule.close_point() == Some(UiNativeProtocolClosePoint::PreparedUpload) {
            return self.closed_report(UiNativeProtocolClosePoint::PreparedUpload);
        }
        match self.run_attempt(
            self.schedule.acquisition_fault(),
            self.schedule.close_point(),
            self.schedule.readback_posture(),
        ) {
            Ok(readback) => self.complete_or_close(readback),
            Err(UiNativePresentationStageFailure::Control(point)) => self.closed_report(point),
            Err(UiNativePresentationStageFailure::Port(fault)) => self.handle_fault(fault),
        }
    }

    fn handle_surface_transition(
        mut self,
        transition: UiNativeProtocolSurfaceTransition,
    ) -> UiNativeLifecycleProtocolReport {
        let production_transition = match transition {
            UiNativeProtocolSurfaceTransition::ZeroSized => {
                UiNativeSurfaceBasisTransition::ZeroSized
            }
            UiNativeProtocolSurfaceTransition::Minimized => {
                UiNativeSurfaceBasisTransition::Minimized
            }
            UiNativeProtocolSurfaceTransition::Resize => UiNativeSurfaceBasisTransition::Resize,
            UiNativeProtocolSurfaceTransition::Dpi => UiNativeSurfaceBasisTransition::Dpi,
        };
        let directive = self
            .lifecycle
            .observe_surface_transition(production_transition, recovery_bindings(self.schedule));
        self.record_peak();
        if directive == UiNativeLifecycleDirective::WaitForVisibility {
            self.resources.abandon_presentation();
            return self.open_report(
                UiNativeProtocolNextAction::WaitForVisibility,
                UiNativeProtocolPredecessor::Retained,
                Some(1),
            );
        }
        if directive == UiNativeLifecycleDirective::Repaint {
            return match self.run_attempt(None, None, UiNativeProtocolReadback::Complete) {
                Ok(_) => self.completed_report(None),
                Err(_) => unreachable!("a surface-basis repaint has no injected port denial"),
            };
        }
        let UiNativeLifecycleDirective::Reconstruct(recovery) = directive else {
            unreachable!("surface transitions produce visibility, repaint or reconstruction")
        };
        self.recover(recovery);
        match self.run_attempt(None, None, UiNativeProtocolReadback::Complete) {
            Ok(_) => self.completed_report(Some(1)),
            Err(_) => unreachable!("surface-basis reconstruction has no injected port denial"),
        }
    }

    fn run_attempt(
        &mut self,
        fault: Option<UiNativePresentationFault>,
        close_at: Option<UiNativeProtocolClosePoint>,
        readback: UiNativeProtocolReadback,
    ) -> Result<
        UiNativeProtocolReadback,
        UiNativePresentationStageFailure<UiNativePresentationFault, UiNativeProtocolClosePoint>,
    > {
        let mut port = UiProtocolPresentationPort::new(fault, readback);
        let mut control = UiProtocolCloseControl::new(close_at, &mut self.completed_stages);
        self.lifecycle
            .run_controlled_presentation(&mut port, &mut control)
    }

    fn complete_or_close(
        mut self,
        readback: UiNativeProtocolReadback,
    ) -> UiNativeLifecycleProtocolReport {
        self.resources.finish_queued_work();
        match readback {
            UiNativeProtocolReadback::Complete => self.completed_report(None),
            UiNativeProtocolReadback::PendingThenComplete => {
                self.observe_pending_readback();
                if self.schedule.close_point() == Some(UiNativeProtocolClosePoint::Readback) {
                    self.closed_report(UiNativeProtocolClosePoint::Readback)
                } else {
                    self.resources.settle_readback();
                    self.completed_report(None)
                }
            }
            UiNativeProtocolReadback::Indeterminate => {
                self.observe_pending_readback();
                self.lifecycle.require_recovery_for(
                    recovery_bindings(self.schedule),
                    UiNativeRecoveryCause::PresentationIndeterminate,
                );
                self.record_peak();
                self.open_report(
                    UiNativeProtocolNextAction::Reconstruct(
                        UiNativePresentationRecoveryClass::PresentationIndeterminate,
                    ),
                    UiNativeProtocolPredecessor::Retained,
                    Some(1),
                )
            }
        }
    }

    fn handle_fault(mut self, fault: UiNativePresentationFault) -> UiNativeLifecycleProtocolReport {
        self.resources.abandon_presentation();
        let directive = self
            .lifecycle
            .observe_protocol_fault(fault, recovery_bindings(self.schedule));
        self.record_peak();
        match directive {
            UiNativeLifecycleDirective::RetryAfterTimeout => self.open_report(
                UiNativeProtocolNextAction::RetryAfterTimeout,
                UiNativeProtocolPredecessor::Retained,
                None,
            ),
            UiNativeLifecycleDirective::WaitForVisibility => self.open_report(
                UiNativeProtocolNextAction::WaitForVisibility,
                UiNativeProtocolPredecessor::Retained,
                None,
            ),
            UiNativeLifecycleDirective::RejectValidation => self.open_report(
                UiNativeProtocolNextAction::RejectValidation,
                UiNativeProtocolPredecessor::Retained,
                None,
            ),
            UiNativeLifecycleDirective::Repaint => {
                unreachable!("presentation faults never carry the retained target onward")
            }
            UiNativeLifecycleDirective::Reconstruct(recovery) => {
                if !self.schedule.recovers() {
                    return self.open_report(
                        UiNativeProtocolNextAction::Reconstruct(recovery),
                        UiNativeProtocolPredecessor::Retained,
                        Some(1),
                    );
                }
                self.recover(recovery);
                if !self.schedule.resumes_after_recovery() {
                    return self.open_report(
                        UiNativeProtocolNextAction::Complete,
                        UiNativeProtocolPredecessor::Retained,
                        Some(1),
                    );
                }
                match self.run_attempt(None, None, UiNativeProtocolReadback::Complete) {
                    Ok(_) => self.completed_report(Some(1)),
                    Err(_) => unreachable!("recovery resume has no external denial or close"),
                }
            }
        }
    }

    fn recover(&mut self, recovery: UiNativePresentationRecoveryClass) {
        if let Some(preparation) = self.lifecycle.physical_recovery_preparation(1) {
            match recovery {
                UiNativePresentationRecoveryClass::Resize
                | UiNativePresentationRecoveryClass::Dpi => {}
                UiNativePresentationRecoveryClass::SurfaceOutdated => {}
                UiNativePresentationRecoveryClass::SurfaceLost => {
                    self.resources.replace_surface();
                    self.surface_generation += 1;
                }
                UiNativePresentationRecoveryClass::DeviceLost => {
                    self.resources.replace_device_and_queue();
                    self.device_generation += 1;
                }
                UiNativePresentationRecoveryClass::PresentationIndeterminate => {}
            }
            self.record_peak();
            assert!(self.lifecycle.commit_physical_recovery(
                preparation,
                self.device_generation,
                self.surface_generation,
            ));
        }
        for binding in 1..=self.schedule.recovery_bindings().max(1) as u64 {
            let requirement = self
                .lifecycle
                .take_recovery(binding)
                .expect("one physical fact admits every affected semantic binding");
            assert!(self.lifecycle.settle_recovery(requirement));
            self.reconstructed_bindings += 1;
        }
        self.lifecycle.reset_presentation_effects();
    }

    fn observe_pending_readback(&mut self) {
        self.pending_readback_observed = true;
        self.resources.begin_readback();
        self.record_peak();
    }

    fn completed_report(
        mut self,
        recovery_binding: Option<u64>,
    ) -> UiNativeLifecycleProtocolReport {
        self.lifecycle.record_presented();
        self.resources.finish_presentation();
        self.open_report(
            UiNativeProtocolNextAction::Complete,
            UiNativeProtocolPredecessor::Replaced,
            recovery_binding,
        )
    }

    fn closed_report(
        mut self,
        point: UiNativeProtocolClosePoint,
    ) -> UiNativeLifecycleProtocolReport {
        self.record_peak();
        self.lifecycle.close_protocol_resources(&mut self.resources);
        self.report(
            UiNativeProtocolNextAction::Closed,
            UiNativeProtocolPredecessor::Released,
            UiNativeProtocolCloseDisposition::ClosedAt(point),
            None,
        )
    }

    fn open_report(
        self,
        next_action: UiNativeProtocolNextAction,
        predecessor: UiNativeProtocolPredecessor,
        recovery_binding: Option<u64>,
    ) -> UiNativeLifecycleProtocolReport {
        self.report(
            next_action,
            predecessor,
            UiNativeProtocolCloseDisposition::Open,
            recovery_binding,
        )
    }

    fn record_peak(&mut self) {
        let mut registry_peak = UiNativeProtocolResourceCensus::from_registry(
            self.resources.peak(),
            self.lifecycle.recovery_count(),
        );
        registry_peak.queued_readiness = self.resources.queued_readiness_count();
        self.peak = self.peak.max(registry_peak).max(self.current_census());
    }

    fn current_census(&self) -> UiNativeProtocolResourceCensus {
        let mut census = UiNativeProtocolResourceCensus::from_registry(
            self.resources.current(),
            self.lifecycle.recovery_count(),
        );
        census.queued_readiness = self.resources.queued_readiness_count();
        census
    }

    fn report(
        mut self,
        next_action: UiNativeProtocolNextAction,
        predecessor: UiNativeProtocolPredecessor,
        close: UiNativeProtocolCloseDisposition,
        recovery_binding: Option<u64>,
    ) -> UiNativeLifecycleProtocolReport {
        self.record_peak();
        let terminal = self.current_census();
        UiNativeLifecycleProtocolReport {
            effect_posture: self.lifecycle.effect_posture(),
            completed_stages: self.completed_stages.into_boxed_slice(),
            next_action,
            predecessor,
            close,
            recovery_binding,
            reconstructed_bindings: self.reconstructed_bindings,
            device_generation: self.device_generation,
            surface_generation: self.surface_generation,
            pending_readback_observed: self.pending_readback_observed,
            peak: self.peak,
            terminal,
        }
    }
}

fn recovery_bindings(schedule: UiNativeLifecycleProtocolSchedule) -> std::ops::RangeInclusive<u64> {
    1..=schedule.recovery_bindings().max(1) as u64
}
