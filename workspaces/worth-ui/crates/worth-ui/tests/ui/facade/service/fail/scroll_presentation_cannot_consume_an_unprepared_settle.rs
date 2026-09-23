//! Presentation consumes prepared results and nothing else. The prepared Scroll
//! settle transition is the evidence that preparation completed, so it is
//! runtime evidence rather than a product value: a caller able to mint one
//! could present a settle that no preparation ever produced, which is exactly
//! the ordering the composition law forbids.
//!
//! What this target proves is the boundary, and only the boundary: the prepared
//! result has no product-side name, so there is no product expression in which
//! presentation could receive one that preparation did not return. The ordering
//! on the runtime side of that boundary is not a compile-time question -- the
//! presentation entry takes the prepared value by type -- and is proved where it
//! can fail at runtime instead: a notch whose preparation cannot complete
//! publishes no settle, commits no successor and moves no offset.

fn main() {
    let _ = worth_ui::facade::service::UiPreparedScrollSettleTransition::prepare;
}
