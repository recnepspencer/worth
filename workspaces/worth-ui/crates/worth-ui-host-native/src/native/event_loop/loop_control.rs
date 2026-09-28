use std::cell::Cell;

use winit::event_loop::{ActiveEventLoop, ControlFlow};

/// The control the host's callbacks exercise over the loop that dispatches
/// them: whether it keeps running and how it waits for the next turn.
///
/// The platform event loop is one implementation. The offscreen pump, which
/// dispatches the same callbacks turn by turn without a window, is the other.
pub(crate) trait UiNativeLoopControl {
    fn exit(&self);
    fn exiting(&self) -> bool;
    fn control_flow(&self) -> ControlFlow;
    fn set_control_flow(&self, control_flow: ControlFlow);
}

impl UiNativeLoopControl for ActiveEventLoop {
    fn exit(&self) {
        ActiveEventLoop::exit(self);
    }

    fn exiting(&self) -> bool {
        ActiveEventLoop::exiting(self)
    }

    fn control_flow(&self) -> ControlFlow {
        ActiveEventLoop::control_flow(self)
    }

    fn set_control_flow(&self, control_flow: ControlFlow) {
        ActiveEventLoop::set_control_flow(self, control_flow);
    }
}

/// The offscreen pump's loop control: the same exit and wait decisions,
/// recorded for the pump to act on between turns.
pub(crate) struct UiNativeOffscreenLoopControl {
    exiting: Cell<bool>,
    control_flow: Cell<ControlFlow>,
}

impl UiNativeOffscreenLoopControl {
    pub(crate) fn new() -> Self {
        Self {
            exiting: Cell::new(false),
            control_flow: Cell::new(ControlFlow::Wait),
        }
    }
}

impl UiNativeLoopControl for UiNativeOffscreenLoopControl {
    fn exit(&self) {
        self.exiting.set(true);
    }

    fn exiting(&self) -> bool {
        self.exiting.get()
    }

    fn control_flow(&self) -> ControlFlow {
        self.control_flow.get()
    }

    fn set_control_flow(&self, control_flow: ControlFlow) {
        self.control_flow.set(control_flow);
    }
}
