//! Framework loans cannot become owned prepared data.
//!
//! The decision reader and execution are lent during preparation. The attempt
//! and writer remain on the owner; the same lifetime fence applies to them.
//! An author can carry owned data, including a `Send` value that is not `Sync`.
//!
//! The decision reader loan cannot escape:
//!
//! ```compile_fail
//! use worth_query_execution::facade::primary_graph::DecisionReader;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! use worth_query_installation::facade::ApplicationSchema;
//! use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call, S: ApplicationSchema + 'static, B: ApplicationMutationBinding<S> + 'static>(
//!     loan: &'call mut DecisionReader<'call, 'call, 'call, S, B>,
//! ) -> impl WorthQueryManagedComputationPrepared {
//!     Held(loan)
//! }
//! ```
//!
//! ```
//! use worth_query_execution::facade::primary_graph::DecisionReader;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! use worth_query_installation::facade::ApplicationSchema;
//! use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call, S: ApplicationSchema + 'static, B: ApplicationMutationBinding<S> + 'static>(
//!     loan: &'call mut DecisionReader<'call, 'call, 'call, S, B>,
//! ) -> impl WorthQueryManagedComputationPrepared {
//!     Held(std::cell::Cell::new(1_u64))
//! }
//! ```
//!
//! The candidate writer loan cannot escape:
//!
//! ```compile_fail
//! use worth_query_execution::facade::primary_graph::CandidateWriter;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! use worth_query_installation::facade::ApplicationSchema;
//! use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call, S: ApplicationSchema + 'static, B: ApplicationMutationBinding<S> + 'static>(
//!     loan: &'call mut CandidateWriter<'call, S, B>,
//! ) -> impl WorthQueryManagedComputationPrepared {
//!     Held(loan)
//! }
//! ```
//!
//! ```
//! use worth_query_execution::facade::primary_graph::CandidateWriter;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! use worth_query_installation::facade::ApplicationSchema;
//! use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call, S: ApplicationSchema + 'static, B: ApplicationMutationBinding<S> + 'static>(
//!     loan: &'call mut CandidateWriter<'call, S, B>,
//! ) -> impl WorthQueryManagedComputationPrepared {
//!     Held(std::cell::Cell::new(1_u64))
//! }
//! ```
//!
//! The attempt context loan cannot escape:
//!
//! ```compile_fail
//! use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantContext;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! use worth_query_installation::facade::ApplicationSchema;
//! use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call, S: ApplicationSchema + 'static>(
//!     loan: &'call WorthQueryApplicationInvariantContext<'call, 'call, S>,
//! ) -> impl WorthQueryManagedComputationPrepared {
//!     Held(loan)
//! }
//! ```
//!
//! ```
//! use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantContext;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! use worth_query_installation::facade::ApplicationSchema;
//! use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call, S: ApplicationSchema + 'static>(
//!     loan: &'call WorthQueryApplicationInvariantContext<'call, 'call, S>,
//! ) -> impl WorthQueryManagedComputationPrepared {
//!     Held(std::cell::Cell::new(1_u64))
//! }
//! ```
//!
//! The advancement phase loan cannot escape:
//!
//! ```compile_fail
//! use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call>(loan: &'call WorthQueryAdvancementPhase<'call>) -> impl WorthQueryManagedComputationPrepared {
//!     Held(loan)
//! }
//! ```
//!
//! ```
//! use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call>(loan: &'call WorthQueryAdvancementPhase<'call>) -> impl WorthQueryManagedComputationPrepared {
//!     Held(std::cell::Cell::new(1_u64))
//! }
//! ```
//!
//! The execution request loan cannot escape:
//!
//! ```compile_fail
//! use worth_query_execution::facade::primary_graph::ExecutionRequest;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call>(loan: ExecutionRequest<'call, 'call>) -> impl WorthQueryManagedComputationPrepared {
//!     Held(loan)
//! }
//! ```
//!
//! ```
//! use worth_query_execution::facade::primary_graph::ExecutionRequest;
//! use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationPrepared;
//! struct Held<T>(T);
//! impl<T: Send + 'static> WorthQueryManagedComputationPrepared for Held<T> {
//!     fn retained_bytes(&self) -> usize { std::mem::size_of::<T>() }
//! }
//! fn prepare<'call>(loan: ExecutionRequest<'call, 'call>) -> impl WorthQueryManagedComputationPrepared {
//!     Held(std::cell::Cell::new(1_u64))
//! }
//! ```
//!
