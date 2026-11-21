//! A quiet Home reopens its acknowledged heads after a quiescent kill or graceful close.
use super::*;
mod quiescent;

#[test]
fn quiescent_kill_reopens_the_acknowledged_native_and_world_heads() {
    quiescent::journey(quiescent::End::Killed);
}

#[test]
fn graceful_close_reopens_the_checkpointed_native_and_world_heads() {
    quiescent::journey(quiescent::End::Closed);
}
