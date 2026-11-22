//! Compile-fail contract tests proving runtime internal ports, one-shot
//! publication custody extractors, execution-only operation-body handler
//! methods, and disclosure/source-expectation constructors stay inaccessible
//! through the public Host facade.
//!
//! Internal `product_world` modules and ports are not exported on Host `runtime`:
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::runtime::product_world;
//! ```
//!
//! Borrowed commit publications cannot drain one-shot custody observations:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryCommittedProductPublication;
//! fn drain_client(publication: &WorthQueryCommittedProductPublication) {
//!     let _ = publication.take_client_observation();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryCommittedProductPublication;
//! fn drain_successor(publication: &WorthQueryCommittedProductPublication) {
//!     let _ = publication.take_successor_observation();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryCommittedProductPublication;
//! fn drain_output_demand(publication: &WorthQueryCommittedProductPublication) {
//!     let _ = publication.take_output_demand_observation();
//! }
//! ```
//!
//! Mutation handlers cannot call execution-only `body_*` methods directly:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::declaration::application_schema::{
//!     ApplicationFieldBindingLocus, ApplicationMutationBinding,
//! };
//! use worth_query_host::facade::domain::ApplicationSchema;
//! use worth_query_host::facade::primary_graph::DecisionReader;
//! fn raw_body_read<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
//!     reader: &mut DecisionReader<'_, '_, '_, S, B>,
//!     locus: &ApplicationFieldBindingLocus,
//! ) {
//!     let _ = reader.body_scope_value(locus);
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::declaration::application_schema::{
//!     ApplicationFieldBindingLocus, ApplicationMutationBinding,
//! };
//! use worth_query_host::facade::domain::ApplicationSchema;
//! use worth_query_host::facade::primary_graph::{
//!     CandidateWriter, WorthQueryApplicationEffectEntity,
//! };
//! fn raw_body_write<S: ApplicationSchema, B: ApplicationMutationBinding<S>, E>(
//!     writer: &mut CandidateWriter<'_, S, B>,
//!     target: &WorthQueryApplicationEffectEntity<S, E>,
//!     locus: &ApplicationFieldBindingLocus,
//!     value: worth_query_host::facade::home_program_transition::AspectValue,
//! ) {
//!     let _ = writer.body_input_field(target, locus, value);
//! }
//! ```
//!
//! Disclosure receipt and source-expectation denial constructors remain
//! execution-internal:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationDisclosureReceipt;
//! fn forge_disclosure() {
//!     let _ = WorthQueryApplicationDisclosureReceipt::public();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::{
//!     WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind,
//! };
//! fn forge_source_denial() {
//!     let _ = WorthQuerySourceExpectationDenial::new(
//!         WorthQuerySourceExpectationDenialKind::MissingExpectation,
//!         "x",
//!     );
//! }
//! ```
//!
//! Product-world resources, direct runs, cleanup envelopes, and unpublished
//! product records do not expose internal custody decomposition or recovery
//! preparation hooks through Host:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::runtime::WorthQueryProductWorldResources;
//! fn decompose_resources(resources: WorthQueryProductWorldResources) {
//!     let _ = resources.into_parts();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::runtime::WorthQueryRunningDirectRun;
//! fn inspect_home_custody(run: &WorthQueryRunningDirectRun) {
//!     let _ = run.managed_home_custody();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCleanupRequired;
//! use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
//! fn forge_cleanup_envelope(policy: ExecutionAllocationPolicy<'_, '_>) {
//!     let _ = WorthQueryApplicationCleanupRequired::reserve(policy);
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::product::WorthQueryProductUnpublishedApplication;
//! fn raw_unpublished_adoption(
//!     unpublished: &WorthQueryProductUnpublishedApplication,
//!     cancellation: &worth_query_host::facade::runtime::RuntimeWorldCancellationToken,
//! ) {
//!     let _ = unpublished.prepare_settled_relational_adoption(cancellation, None);
//! }
//! ```
//!
//! Runtime wrappers do not expose transparent `Deref` to lower runtime crates,
//! and sibling-crate provider-session modules/ports are absent from Host:
//!
//! ```compile_fail,E0277
//! use std::ops::Deref;
//! use worth_query_host::facade::runtime::WorthQueryExecutionRuntime;
//! fn require_deref<T: Deref>(_: &T) {}
//! fn no_runtime_deref(runtime: &WorthQueryExecutionRuntime) {
//!     require_deref(runtime);
//! }
//! ```
//!
//! ```compile_fail,E0277
//! use std::ops::Deref;
//! use worth_query_host::facade::runtime::WorthQueryExecutionInstallationAuthority;
//! fn require_deref<T: Deref>(_: &T) {}
//! fn no_authority_deref(authority: &WorthQueryExecutionInstallationAuthority) {
//!     require_deref(authority);
//! }
//! ```
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::installed::provider_session::graph_obligation;
//! ```
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::installed::provider_session::protocol;
//! ```
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::installed::domain_computation::WorthQueryApplicationOperationBindingInput;
//! ```
