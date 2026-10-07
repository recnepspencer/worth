//! # worth-signal
//!
//! `worth-signal` is a deterministic incremental runtime for derived work.
//!
//! Your app owns the real state.
//! `worth-signal` owns dependency tracking, invalidation, recompute, rollback,
//! diagnostics, replay, and history.
//!
//! There are two normal entry paths:
//!
//! ```rust
//! use worth_signal::easy::*;
//! use worth_signal::facade::*;
//! ```
//!
//! Use `easy` for the shortest path.
//! Use `facade` when you want the broader runtime surface from the start.
//!
//! The important line is this:
//!
//! - not "reactive graph plus some debug helpers"
//! - not "incremental cache plus a separate audit layer"
//! - not "rerun less work and figure out the rest later"
//!
//! `worth-signal` keeps change propagation, transactions, diagnostics, and
//! history in one runtime.
//!
//! Most days, the center of gravity is:
//!
//! - [`facade::SignalGraph`]
//! - [`facade::SignalRuntime`]
//! - `runtime.advance_signal_branch(...)`
//! - `runtime.diagnostics()`
//! - `runtime.history()`
//!
//! ## Fast Mental Model
//!
//! - Build a dependency graph.
//! - Tell the runtime what changed.
//! - Read the derived node you care about.
//! - Ask diagnostics why something ran when it should not have.
//! - Use history when you need the trail, not just the latest answer.
//!
//! `worth-signal` is domain-free on purpose. The same runtime shape works for:
//!
//! - web backends and reactive views
//! - finance and risk pipelines
//! - ML feature and scoring flows
//! - document indexing or compiler-style partial recompute
//!
//! The flagship story looks like this:
//!
//! - a source file changes
//! - a transaction lands the update
//! - only the right downstream targets rerun
//! - diagnostics explain why the bundle moved
//! - replay keeps the trail
//!
//! ## Small Example
//!
//! ```no_run
//! use worth_signal::facade::*;
//! const PRICE: Aspect = Aspect::new(0);
//! const TOTAL: Aspect = Aspect::new(1);
//!
//! #[derive(Default)]
//! struct CheckoutState {
//!     price_version: u64,
//!     total_version: u64,
//! }
//!
//! let mut graph = SignalGraph::new();
//! let price = graph.node().build();
//! let total = graph.node().on_demand().build();
//!
//! graph.set_dependencies(total, [DependencyEdge::new(price, PRICE)])?;
//!
//! let mut runtime = SignalRuntime::build_for::<CheckoutState>(graph);
//!
//! let mut state = CheckoutState {
//!     price_version: 2,
//!     total_version: 5,
//! };
//!
//! let evaluate = |view: &mut EvaluationContext<'_, CheckoutState>| {
//!     let result = if view.node() == price {
//!         view.finish(NodeEvaluationResult::from_version(
//!             AspectVersion::from_updates([(PRICE, view.domain().price_version)]),
//!         ))
//!     } else {
//!         let _upstream = view.read_aspect_version(price, PRICE)?;
//!         view.finish(NodeEvaluationResult::from_version(
//!             AspectVersion::from_updates([(TOTAL, view.domain().total_version)]),
//!         ))
//!     };
//!     Ok::<_, SignalError>(result)
//! };
//!
//! let basis = runtime
//!     .observe_signal_branch_basis(runtime.current_branch())
//!     .expect("current branch should admit an owner basis");
//! let _next_basis = runtime.advance_signal_branch(&mut state, &basis, |tx| {
//!     tx.mark_changed(price, PRICE)?;
//!     tx.target(total).read(&evaluate)?;
//!     Ok(())
//! })
//! .expect("admitted branch advance should succeed")
//! .into_basis();
//!
//! let version = runtime.target(total).read(&state, &evaluate)?;
//! assert_eq!(version.get(TOTAL), 5);
//! # Ok::<(), SignalError>(())
//! ```
//!
//! ## Docs
//!
//! Start with:
//!
//! - `crates/worth-signal/docs/GETTING_STARTED.md`
//! - `crates/worth-signal/docs/API_OVERVIEW.md`
//! - `crates/worth-signal/docs/walkthroughs/compiler-targeted-rebuild.md`
//! - `crates/worth-signal/docs/guides/running-the-runtime.md`
//! - `crates/worth-signal/docs/guides/debugging-and-diagnostics.md`
//!
//! Examples live in:
//!
//! - `crates/worth-signal/examples/easy_task_board.rs`
//! - `crates/worth-signal/examples/compiler_targeted_rebuild.rs`
//! - `crates/worth-signal/examples/branch_bases.rs`

#![forbid(unsafe_code)]

mod branch;
mod clock;
mod data;
pub mod diagnostics;
pub mod easy;
mod logic;
pub(crate) mod observation;
#[cfg(not(test))]
mod presentation;
#[cfg(test)]
pub mod presentation;
pub(crate) mod runtime_policy;
pub mod schema;
mod state;

pub mod facade;

/// Compiles the branch-basis guide against the real public facade.
///
/// The module exists only while rustdoc collects doctests, so it never enters
/// the public API and `BRANCH_BASES.md` is never rendered as a rustdoc page.
/// It is here so the guide cannot drift away from the facade without breaking
/// `cargo test`.
#[cfg(doctest)]
pub mod branch_bases_guide;

#[cfg(test)]
pub(crate) mod tests;
