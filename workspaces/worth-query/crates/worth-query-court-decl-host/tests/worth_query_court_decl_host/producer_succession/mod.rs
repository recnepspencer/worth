//! Generator succession through the public installed Query producer path.
#[macro_use]
mod fixture;
const PRIVATE_AGGREGATES: bool = false;
producer_succession_fixture!();
mod checkpoint;
mod checkpoint_value;
mod correction_courts;
mod courts;
mod history_courts;
mod independent_generator_courts;
mod locality;
mod refusal_courts;
mod write_fence_courts;

#[path = "route_degree/courts.rs"]
mod route_degree;
