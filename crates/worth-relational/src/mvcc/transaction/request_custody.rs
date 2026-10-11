//! Compile-visible commit request custody.
//!
//! A commit seam cannot be called without the caller request.
//!
//! ```compile_fail,E0061
//! # use worth_execution::ExecutionRequest;
//! # use worth_relational::facade::{runtime::RelationalRuntime, mvcc::BranchBoundRelationalTransaction};
//! fn prepare(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, transaction: BranchBoundRelationalTransaction) {
//!     let _ = runtime.prepare_branch_transaction(transaction);
//! }
//! ```
//!
//! The passing twin adds only that borrowed argument.
//!
//! ```
//! # use worth_execution::ExecutionRequest;
//! # use worth_relational::facade::{runtime::RelationalRuntime, mvcc::BranchBoundRelationalTransaction};
//! fn prepare(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, transaction: BranchBoundRelationalTransaction) {
//!     let _ = runtime.prepare_branch_transaction(request, transaction);
//! }
//! ```
//!
//! A caller cannot keep the borrowed request with a prepared candidate after its host scope.
//!
//! ```compile_fail,E0521
//! # use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
//! # use worth_relational::facade::{mvcc::PreparedRelationalCommitCandidate, runtime::RelationalRuntimeApi, schema::RelationalSchemaRegistry};
//! # fn with_host_policy<R>(body: impl for<'scope> FnOnce(&'scope ExecutionRequest<'scope, 'scope>) -> R) -> R {
//! #     let backing = SerialRequest::from_memory(SerialMemoryBudget::new(64 * 1024 * 1024), CancellationSource::new().token(), None);
//! #     let request = ExecutionRequest::serial(&backing);
//! #     body(&request)
//! # }
//! # fn prepare(request: &ExecutionRequest<'_, '_>) -> PreparedRelationalCommitCandidate {
//! #     let runtime = RelationalRuntimeApi::builder().schema_registry(RelationalSchemaRegistry::new()).build();
//! #     let identity = runtime.main_branch_identity();
//! #     let (_, basis) = runtime.observe_branch(&identity).unwrap();
//! #     let transaction = runtime.begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()).unwrap();
//! #     runtime.prepare_branch_transaction(request, transaction).unwrap()
//! # }
//! struct PreparedWithRequest<'scope> {
//!     candidate: PreparedRelationalCommitCandidate,
//!     request: Option<&'scope ExecutionRequest<'scope, 'scope>>,
//! }
//! let mut retained: Option<PreparedWithRequest<'static>> = None;
//! with_host_policy(|request| {
//!     let candidate = prepare(request);
//!     retained = Some(PreparedWithRequest { candidate, request: Some(request) });
//! });
//! drop(retained);
//! ```
//!
//! The passing twin retains the owned candidate and no request borrow.
//!
//! ```
//! # use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
//! # use worth_relational::facade::{mvcc::PreparedRelationalCommitCandidate, runtime::RelationalRuntimeApi, schema::RelationalSchemaRegistry};
//! # fn with_host_policy<R>(body: impl for<'scope> FnOnce(&'scope ExecutionRequest<'scope, 'scope>) -> R) -> R {
//! #     let backing = SerialRequest::from_memory(SerialMemoryBudget::new(64 * 1024 * 1024), CancellationSource::new().token(), None);
//! #     let request = ExecutionRequest::serial(&backing);
//! #     body(&request)
//! # }
//! # fn prepare(request: &ExecutionRequest<'_, '_>) -> PreparedRelationalCommitCandidate {
//! #     let runtime = RelationalRuntimeApi::builder().schema_registry(RelationalSchemaRegistry::new()).build();
//! #     let identity = runtime.main_branch_identity();
//! #     let (_, basis) = runtime.observe_branch(&identity).unwrap();
//! #     let transaction = runtime.begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()).unwrap();
//! #     runtime.prepare_branch_transaction(request, transaction).unwrap()
//! # }
//! struct PreparedWithRequest<'scope> {
//!     candidate: PreparedRelationalCommitCandidate,
//!     request: Option<&'scope ExecutionRequest<'scope, 'scope>>,
//! }
//! let mut retained: Option<PreparedWithRequest<'static>> = None;
//! with_host_policy(|request| {
//!     let candidate = prepare(request);
//!     retained = Some(PreparedWithRequest { candidate, request: None });
//! });
//! drop(retained);
//! ```

//! An independent lease cannot override the caller's commit request.
//!
//! ```compile_fail,E0061
//! # use worth_execution::{ExecutionRequest, ExecutionResourceLease};
//! # use worth_relational::facade::{runtime::RelationalRuntime, mvcc::BranchBoundRelationalTransaction};
//! fn prepare(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, transaction: BranchBoundRelationalTransaction, unrelated: &ExecutionResourceLease<'_>) {
//!     let _ = runtime.prepare_branch_transaction(request, transaction, unrelated);
//! }
//! ```
//!
//! The passing twin lends only the caller request.
//!
//! ```
//! # use worth_execution::{ExecutionRequest, ExecutionResourceLease};
//! # use worth_relational::facade::{runtime::RelationalRuntime, mvcc::BranchBoundRelationalTransaction};
//! fn prepare(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, transaction: BranchBoundRelationalTransaction, unrelated: &ExecutionResourceLease<'_>) {
//!     let _ = runtime.prepare_branch_transaction(request, transaction);
//! }
//! ```
