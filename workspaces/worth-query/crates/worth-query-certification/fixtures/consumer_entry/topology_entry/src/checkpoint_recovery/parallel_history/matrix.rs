//! Required application waves, judged by declared input histories.
use super::{
    expected_history::{Partition, World},
    seeded_world::{self, FAMILIES, SEED},
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
