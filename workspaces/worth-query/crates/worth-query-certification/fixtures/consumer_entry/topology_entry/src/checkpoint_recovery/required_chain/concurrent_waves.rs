//! Application-level wave evidence: real dependencies, owner events, unequal compute.
use super::*;
mod application;
mod computation;
mod history;

mod queue_failures;

mod ordinary_read_application;
mod ordinary_reads;

mod three_members;

mod abandonment;
mod deferral_freeze;

mod frozen_truth;

mod abandonment_thaw;

mod two_owners;

mod checkpoint_restore;

mod charge_prefix;
