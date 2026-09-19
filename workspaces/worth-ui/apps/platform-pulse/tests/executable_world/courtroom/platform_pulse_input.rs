use std::time::Instant;

use crate::product_process::{
    InitialBlue, NativeInputCausalStep, NativeInputReached, Published, PulseExecutableWorld,
};

pub(super) fn reach_native_input_observed(
    initial: PulseExecutableWorld<Published<InitialBlue>>,
    deadline: Instant,
    observe_step: impl FnMut(NativeInputCausalStep),
) -> PulseExecutableWorld<Published<NativeInputReached<InitialBlue>>> {
    let reached = initial
        .reach_native_input_observed(deadline, observe_step)
        .unwrap_or_else(|failure| {
            panic!("real native input reaches the production adapter: {failure}")
        });
    let evidence = reached.evidence();
    assert_eq!(evidence.sequences(), (5, 6, 7));
    assert_eq!(evidence.delivered_event_count(), 4);
    assert!(evidence.pointer_button_events() > 0);
    assert!(evidence.keyboard_events() > 0);
    assert!(evidence.matching_blue_samples() * 4 >= evidence.sampled_pixels() * 3);
    reached
}
