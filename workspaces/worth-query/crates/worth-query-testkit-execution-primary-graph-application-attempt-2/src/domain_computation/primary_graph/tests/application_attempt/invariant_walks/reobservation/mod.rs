//! Exact proposal views freshly observe reached survivors and real CRUD absences.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::Probe;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::*;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[path = "program/mod.rs"]
pub mod program;
