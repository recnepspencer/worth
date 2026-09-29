use winit::event_loop::ControlFlow;

use super::loop_control::UiNativeLoopControl;

use super::UiNativeEventLoopDirective;

pub(super) fn apply(
    event_loop: &dyn UiNativeLoopControl,
    directive: UiNativeEventLoopDirective,
) -> bool {
    match directive {
        UiNativeEventLoopDirective::Continue => set_wait(event_loop),
        UiNativeEventLoopDirective::WaitUntil(deadline) => {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            false
        }
        UiNativeEventLoopDirective::Close => true,
    }
}

fn set_wait(event_loop: &dyn UiNativeLoopControl) -> bool {
    event_loop.set_control_flow(ControlFlow::Wait);
    false
}
