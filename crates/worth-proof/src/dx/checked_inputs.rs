use core::convert::Infallible;

use crate::transition::{PreConstructionGate, TransitionReadiness};

pub fn gate_ready<C>(context: C) -> PreConstructionGate<C, Infallible, Infallible> {
    PreConstructionGate::ready(context)
}

pub fn ready_now<C>(
    context: C,
) -> TransitionReadiness<C, Infallible, Infallible, Infallible, Infallible, Infallible> {
    TransitionReadiness::ready(context)
}
