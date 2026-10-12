//! Required application waves, judged by declared input histories.
use super::{
    expected_history::{Partition, World},
    seeded_world::{self, Family, SEED},
};
mod application;
mod cases;
mod computation;
mod control;
mod edit;
mod facts;
mod installation;
mod model;
mod operation;
mod producer;
mod readiness;
mod runtime;
mod source;

mod table;

mod inventory;

mod model_self_check;

// Part one's fixture keeps its names. Here the original four serial frames
// are Zipped; the eighth family holds one real four-partition parallel map.
const FAMILIES: [Family; 8] = [
    Family::Empty,
    Family::Singleton,
    Family::Chain,
    Family::Diamond,
    Family::Independent,
    Family::Unequal,
    Family::Nested,
    Family::Nested,
];
const FAMILY_NAMES: [&str; 8] = [
    "Empty",
    "Singleton",
    "Chain",
    "Diamond",
    "Independent",
    "Unequal",
    "Zipped",
    "NestedParallel",
];
mod nested_cost;
