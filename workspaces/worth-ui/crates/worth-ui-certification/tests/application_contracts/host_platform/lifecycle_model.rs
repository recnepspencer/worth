use worth_ui_host_native::{
    UiNativeLifecycleProtocolSchedule, UiNativeLifecycleProtocolWorld, UiNativePresentationFault,
    UiNativeProtocolClosePoint, UiNativeProtocolReadback, UiNativeProtocolSurfaceTransition,
};

#[path = "lifecycle_model/production_projection.rs"]
mod production_projection;

use production_projection::production_report;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Prepared,
    Acquired,
    Encoded,
    Submitted,
    Handoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Posture {
    BeforeEffects,
    Stage(Stage),
    Presented,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Recovery {
    Resize,
    Dpi,
    SurfaceOutdated,
    SurfaceLost,
    DeviceLost,
    PresentationIndeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Action {
    Complete,
    RetryAfterTimeout,
    WaitForVisibility,
    RejectValidation,
    Reconstruct(Recovery),
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Predecessor {
    Retained,
    Replaced,
    Released,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Census {
    queued: usize,
    held_attempts: usize,
    prepared_uploads: usize,
    surfaces: usize,
    devices: usize,
    queues: usize,
    presentations: usize,
    readbacks: usize,
    reconstructions: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct ModelReport {
    posture: Posture,
    stages: Vec<Stage>,
    action: Action,
    predecessor: Predecessor,
    close: Option<UiNativeProtocolClosePoint>,
    recovery_binding: Option<u64>,
    reconstructed_bindings: usize,
    device_generation: u64,
    surface_generation: u64,
    pending_readback_observed: bool,
    peak: Census,
    terminal: Census,
}

pub(super) fn assert_world(schedule: UiNativeLifecycleProtocolSchedule) {
    let production = production_report(UiNativeLifecycleProtocolWorld::run(schedule));
    assert_eq!(production, model_report(schedule));
}

fn model_report(schedule: UiNativeLifecycleProtocolSchedule) -> ModelReport {
    let mut model = ModelExecution::new(schedule);
    if let Some(transition) = schedule.scheduled_surface_transition() {
        return model.surface_transition(transition, schedule);
    }
    if schedule.close_point() == Some(UiNativeProtocolClosePoint::PreparedUpload) {
        return model.close(UiNativeProtocolClosePoint::PreparedUpload);
    }
    if let Some(fault) = schedule.acquisition_fault() {
        model.stages.push(Stage::Prepared);
        model.posture = Posture::Stage(Stage::Prepared);
        model.terminal.presentations = 0;
        return model.fault(fault, schedule);
    }
    if let Some(close_at) = schedule
        .close_point()
        .filter(|point| *point != UiNativeProtocolClosePoint::Readback)
    {
        model.complete_through(close_at);
        return model.close(close_at);
    }
    model.complete_presentation_stages();
    model.terminal.queued = 0;
    model.terminal.held_attempts = 0;
    match schedule.readback_posture() {
        UiNativeProtocolReadback::Complete => model.complete(None),
        UiNativeProtocolReadback::PendingThenComplete => {
            model.observe_pending_readback();
            if schedule.close_point() == Some(UiNativeProtocolClosePoint::Readback) {
                model.close(UiNativeProtocolClosePoint::Readback)
            } else {
                model.terminal.readbacks = 0;
                model.complete(None)
            }
        }
        UiNativeProtocolReadback::Indeterminate => {
            model.observe_pending_readback();
            model.terminal.reconstructions = schedule.recovery_bindings().max(1);
            model.record_peak();
            model.report(
                Action::Reconstruct(Recovery::PresentationIndeterminate),
                Predecessor::Retained,
                None,
                Some(1),
            )
        }
    }
}

struct ModelExecution {
    posture: Posture,
    stages: Vec<Stage>,
    peak: Census,
    terminal: Census,
    device_generation: u64,
    surface_generation: u64,
    pending_readback_observed: bool,
    reconstructed_bindings: usize,
}

impl ModelExecution {
    fn new(schedule: UiNativeLifecycleProtocolSchedule) -> Self {
        let terminal = Census {
            queued: usize::from(schedule.queued_readiness()),
            held_attempts: 0,
            prepared_uploads: usize::from(
                schedule.close_point() == Some(UiNativeProtocolClosePoint::PreparedUpload),
            ),
            surfaces: 1,
            devices: 1,
            queues: 1,
            presentations: 1,
            ..Census::default()
        };
        Self {
            posture: Posture::BeforeEffects,
            stages: Vec::new(),
            peak: terminal,
            terminal,
            device_generation: 1,
            surface_generation: 1,
            pending_readback_observed: false,
            reconstructed_bindings: 0,
        }
    }

    fn fault(
        mut self,
        fault: UiNativePresentationFault,
        schedule: UiNativeLifecycleProtocolSchedule,
    ) -> ModelReport {
        let action = match fault {
            UiNativePresentationFault::Timeout => Action::RetryAfterTimeout,
            UiNativePresentationFault::Occluded => Action::WaitForVisibility,
            UiNativePresentationFault::Validation => Action::RejectValidation,
            UiNativePresentationFault::Outdated => Action::Reconstruct(Recovery::SurfaceOutdated),
            UiNativePresentationFault::SurfaceLost => Action::Reconstruct(Recovery::SurfaceLost),
            UiNativePresentationFault::DeviceLost => Action::Reconstruct(Recovery::DeviceLost),
        };
        let Action::Reconstruct(recovery) = action else {
            return self.report(action, Predecessor::Retained, None, None);
        };
        let binding_count = schedule.recovery_bindings().max(1);
        self.terminal.reconstructions = binding_count;
        self.record_peak();
        if !schedule.recovers() {
            return self.report(action, Predecessor::Retained, None, Some(1));
        }
        match recovery {
            Recovery::Resize | Recovery::Dpi => {
                unreachable!("surface fault rows cannot synthesize basis transitions")
            }
            Recovery::SurfaceOutdated => {}
            Recovery::SurfaceLost => {
                self.terminal.surfaces = 2;
                self.record_peak();
                self.terminal.surfaces = 1;
                self.surface_generation += 1;
            }
            Recovery::DeviceLost => {
                self.terminal.devices = 2;
                self.terminal.queues = 2;
                self.record_peak();
                self.terminal.devices = 1;
                self.terminal.queues = 1;
                self.device_generation += 1;
            }
            Recovery::PresentationIndeterminate => unreachable!(),
        }
        self.terminal.reconstructions = 0;
        self.reconstructed_bindings = binding_count;
        self.posture = Posture::BeforeEffects;
        self.complete_presentation_stages();
        self.complete(Some(1))
    }

    fn surface_transition(
        mut self,
        transition: UiNativeProtocolSurfaceTransition,
        schedule: UiNativeLifecycleProtocolSchedule,
    ) -> ModelReport {
        match transition {
            UiNativeProtocolSurfaceTransition::ZeroSized
            | UiNativeProtocolSurfaceTransition::Minimized => {
                self.terminal.presentations = 0;
                self.terminal.reconstructions = schedule.recovery_bindings().max(1);
                self.record_peak();
                self.report(
                    Action::WaitForVisibility,
                    Predecessor::Retained,
                    None,
                    Some(1),
                )
            }
            // Same-scale resize keeps every retained resource and repaints.
            UiNativeProtocolSurfaceTransition::Resize => {
                self.complete_presentation_stages();
                self.complete(None)
            }
            UiNativeProtocolSurfaceTransition::Dpi => {
                self.terminal.reconstructions = schedule.recovery_bindings().max(1);
                self.record_peak();
                self.reconstructed_bindings = self.terminal.reconstructions;
                self.terminal.reconstructions = 0;
                self.complete_presentation_stages();
                self.complete(Some(1))
            }
        }
    }

    fn complete_through(&mut self, close_at: UiNativeProtocolClosePoint) {
        let count = match close_at {
            UiNativeProtocolClosePoint::Prepared => 1,
            UiNativeProtocolClosePoint::SurfaceAcquired => 2,
            UiNativeProtocolClosePoint::Encoded => 3,
            UiNativeProtocolClosePoint::Submitted => 4,
            UiNativeProtocolClosePoint::PresentHandoff => 5,
            UiNativeProtocolClosePoint::PreparedUpload | UiNativeProtocolClosePoint::Readback => 0,
        };
        for stage in all_stages().into_iter().take(count) {
            self.stages.push(stage);
            self.posture = Posture::Stage(stage);
        }
    }

    fn complete_presentation_stages(&mut self) {
        for stage in all_stages() {
            self.stages.push(stage);
            self.posture = Posture::Stage(stage);
        }
    }

    fn observe_pending_readback(&mut self) {
        self.pending_readback_observed = true;
        self.terminal.readbacks = 1;
        self.record_peak();
    }

    fn complete(mut self, authority: Option<u64>) -> ModelReport {
        self.posture = Posture::Presented;
        self.terminal.presentations = 0;
        self.terminal.readbacks = 0;
        self.report(Action::Complete, Predecessor::Replaced, None, authority)
    }

    fn close(mut self, point: UiNativeProtocolClosePoint) -> ModelReport {
        self.record_peak();
        self.terminal = Census::default();
        self.report(Action::Closed, Predecessor::Released, Some(point), None)
    }

    fn record_peak(&mut self) {
        self.peak = max_census(self.peak, self.terminal);
    }

    fn report(
        self,
        action: Action,
        predecessor: Predecessor,
        close: Option<UiNativeProtocolClosePoint>,
        recovery_binding: Option<u64>,
    ) -> ModelReport {
        ModelReport {
            posture: self.posture,
            stages: self.stages,
            action,
            predecessor,
            close,
            recovery_binding,
            reconstructed_bindings: self.reconstructed_bindings,
            device_generation: self.device_generation,
            surface_generation: self.surface_generation,
            pending_readback_observed: self.pending_readback_observed,
            peak: self.peak,
            terminal: self.terminal,
        }
    }
}

fn all_stages() -> [Stage; 5] {
    [
        Stage::Prepared,
        Stage::Acquired,
        Stage::Encoded,
        Stage::Submitted,
        Stage::Handoff,
    ]
}

fn max_census(left: Census, right: Census) -> Census {
    Census {
        queued: left.queued.max(right.queued),
        held_attempts: left.held_attempts.max(right.held_attempts),
        prepared_uploads: left.prepared_uploads.max(right.prepared_uploads),
        surfaces: left.surfaces.max(right.surfaces),
        devices: left.devices.max(right.devices),
        queues: left.queues.max(right.queues),
        presentations: left.presentations.max(right.presentations),
        readbacks: left.readbacks.max(right.readbacks),
        reconstructions: left.reconstructions.max(right.reconstructions),
    }
}
