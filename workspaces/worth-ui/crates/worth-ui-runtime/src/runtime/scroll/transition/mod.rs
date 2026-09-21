//! Scroll transition succession: the semantic target a Scroll owner is moving
//! toward between accepted samples, its settle horizon, and the Motion request
//! that interpolates it.
//!
//! Targets are derived authority, not displayed truth. The accepted Motion
//! sample alone governs displayed geometry, hit testing and thumb position;
//! a target only says where that sample is heading and until when.

mod denial;
mod motion_request;
mod settle_horizon;
mod succession;
mod target;
mod wheel_accumulation;

pub(crate) use denial::UiScrollTransitionDenial;
pub(crate) use motion_request::{
    scroll_settle_motion_request, UiScrollMotionBinding, UiScrollMotionRequestDenial,
};
pub(crate) use settle_horizon::UiScrollSettleHorizon;
pub(crate) use succession::{
    UiScrollTransitionBasis, UiScrollTransitionReclampOutcome, UiScrollTransitionSuccession,
};
pub(crate) use target::UiScrollTransitionTarget;
pub(crate) use wheel_accumulation::{
    line_travel, UiScrollWheelInput, UiScrollWheelLineDelta, UiScrollWheelWindow,
    UI_SCROLL_WHEEL_LINE_MILLI_PER_LINE,
};

#[cfg(test)]
mod motion_request_tests;
#[cfg(test)]
mod succession_tests;
#[cfg(test)]
mod wheel_accumulation_tests;
