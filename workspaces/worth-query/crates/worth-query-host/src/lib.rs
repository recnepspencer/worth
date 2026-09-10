//! Query host audience facade.
//!
//! Admission, lowering, execution, and publication consumers depend on this
//! crate instead of importing Query's internal authority packages directly.
//! The read-only [`facade::domain`] surface exposes an installed schema's
//! `native_contracts()`, an installed operation's typed `graph_reads()` and
//! `touches()`, and complete aftermath inspection through `authority()`,
//! `recovery()`, `reconciliation()`, and
//! `external_effect().correlation_family()`. These accessors inspect retained
//! meaning; they do not grant installation, execution, correction, recovery,
//! or external-effect authority.
//!
//! Product-aware hosts enter through
//! `WorthQueryPrimaryGraphApplicationRuntime::current_world`, `branches`, and
//! `on_branch`. Selection returns an exact World-owned composite occurrence;
//! reads, admitted changes, publication, delivery, and inspection carry that
//! occurrence without reconstructing authority from component identifiers.
//! Runnable ordinary and advanced journeys live in the
//! `worth-query-certification` package examples.
//!
//! ```
//! use worth_query_host::facade::domain::{
//!     WorthQueryInstalledApplicationOperation, WorthQueryOperationGraphReadScope,
//! };
//!
//! fn inspect<Schema, Operation, Input>(
//!     operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
//! ) {
//!     for role in operation.contracts().graph_reads().roles() {
//!         for scope in role.read_scopes() {
//!             if let WorthQueryOperationGraphReadScope::NativeProjection(scope) = scope {
//!                 let _ = (
//!                     scope.entity().semantic_key(),
//!                     scope.aspect().as_str(),
//!                     scope.projection().mask(),
//!                 );
//!             }
//!         }
//!     }
//!
//!     if let Some(aftermath) = operation.contracts().aftermath() {
//!         let _ = (
//!             aftermath.authority(),
//!             aftermath.recovery(),
//!             aftermath.reconciliation(),
//!             aftermath.external_effect().correlation_family(),
//!         );
//!     }
//! }
//! ```
//!
//! ```
//! use worth_query_host::facade::{admission, domain, primary_graph, runtime};
//! # fn _host_surface(
//! #     installer: runtime::WorthQueryExecutionRuntimeInstaller,
//! #     package: domain::WorthQueryPortableDomainPackage,
//! # ) {
//! #     let _ = (
//! #         installer,
//! #         package,
//! #         std::any::TypeId::of::<primary_graph::WorthQueryPrincipalResolutionMode>(),
//! #         std::any::TypeId::of::<admission::resource_admission::WorthQueryExecutionResourceAdmissionDenial>(),
//! #     );
//! # }
//! ```
//!
//! Raw primary-graph integration is not an audience capability:
//!
//! ```compile_fail
//! use worth_query_host::facade::runtime::WorthQueryExecutionRuntime;
//!
//! fn cannot_extract_relational_graph(runtime: &WorthQueryExecutionRuntime) {
//!     let _ = runtime.retain_primary_graph_integration_handle();
//! }
//! ```
//!
//! ```compile_fail
//! use worth_query_host::facade::domain::ApplicationSchema;
//! use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphBootstrap;
//! use worth_query_host::facade::runtime::{
//!     WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
//! };
//!
//! fn cannot_publish_broad_runtime<Schema: ApplicationSchema>(
//!     graph: WorthQueryPrimaryGraphBootstrap<Schema>,
//!     runtime: &mut WorthQueryExecutionRuntime,
//!     authority: &WorthQueryExecutionInstallationAuthority,
//! ) {
//!     graph.publish(runtime, authority).unwrap();
//! }
//! ```
//!
//! The public compiler contract is exercised as one family. Product tokens
//! cannot be minted, recovery custody is linear, and ordinary reads cannot
//! skip into effect authoring:
//!
//! ```compile_fail
//! use worth_query_host::facade::product::WorthQueryProductBranch;
//!
//! fn cannot_mint_product_branch() -> WorthQueryProductBranch {
//!     WorthQueryProductBranch { occurrence: todo!() }
//! }
//! ```
//!
//! ```compile_fail
//! use worth_query_host::facade::primary_graph::WorthQueryProductUnpublishedRecovery;
//!
//! fn cannot_duplicate_recovery(recovery: WorthQueryProductUnpublishedRecovery) {
//!     let first = recovery;
//!     let second = recovery;
//!     drop((first, second));
//! }
//! ```
//!
//! ```compile_fail
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryCompleteApplicationReadSet, WorthQueryOrdinaryApplicationRead,
//! };
//!
//! fn cannot_skip_read_completion<Schema, Operation, Input, Scope>(
//!     reads: WorthQueryCompleteApplicationReadSet<
//!         Schema, Operation, Input, Scope, WorthQueryOrdinaryApplicationRead,
//!     >,
//! ) {
//!     let _ = reads.begin_effect_program();
//! }
//! ```
//!
//! Their positive twins invoke the owning selection, recovery, and phase
//! boundaries with authority issued by the preceding public step:
//!
//! ```
//! use worth_query_host::facade::{domain::ApplicationSchema, primary_graph, product};
//!
//! fn select_issued<Schema: ApplicationSchema>(
//!     runtime: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
//!     branch: product::WorthQueryProductBranch,
//! ) {
//!     let _ = runtime.on_branch(branch).select();
//! }
//!
//! fn release_linear_recovery<Schema: ApplicationSchema>(
//!     runtime: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
//!     recovery: primary_graph::WorthQueryProductUnpublishedRecovery,
//! ) {
//!     let _ = recovery.record_handle();
//!     let _ = runtime.release_product_publication_recovery(recovery, 0);
//! }
//!
//! fn author_after_projected_completion<Schema, Operation, Input, Scope>(
//!     reads: primary_graph::WorthQueryCompleteApplicationReadSet<
//!         Schema,
//!         Operation,
//!         Input,
//!         Scope,
//!         primary_graph::WorthQueryProjectedApplicationMutation,
//!     >,
//! ) {
//!     let _ = reads.begin_effect_program();
//! }
//! ```

pub mod facade;
