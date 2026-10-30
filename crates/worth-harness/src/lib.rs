#![cfg_attr(test, allow(clippy::disallowed_methods))]
#![forbid(unsafe_code)]
//! WORTH Harness is first-class execution infrastructure for WORTH runtimes.
//!
//! It provides the stable substrate for scenario fixtures, mutation batches,
//! execution profiles, record capture, replay, event streams, comparison, and
//! harness-native tooling such as benches, run matrices, and parity suites.
//!
//! The crate is intentionally runtime-neutral. `worth-signal` is the first
//! adapter, but the contract is shaped to support relational, bridge, kernel,
//! fintech, and game-loop style runtimes without leaking runtime internals into
//! the generic harness surface.

pub mod artifact;
pub mod capture;
pub mod comparison;
pub mod compatibility;
pub mod export;
pub mod extension;
pub mod identity;
pub mod replay;
pub mod runtime;
pub mod scenario;
pub mod timeline;
pub mod tooling;
pub mod workflow;
pub mod workload;

pub mod facade;
