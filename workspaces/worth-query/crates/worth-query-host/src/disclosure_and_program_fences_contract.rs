//! Host-facade compile-fail fences verifying that sealed effect programs,
//! builders, entity targets, read sets, admitted query plans, and commit
//! receipts keep internal fields and retained disclosure payloads private.
//!
//! Direct access to retained pre-image fields on a commit receipt is refused:
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn leak_preimage_field(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = &receipt.retained_preimage;
//! }
//! ```
//!
//! Direct access to authority-binding fields on a commit receipt is refused:
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn leak_binding_field(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = &receipt.authority_binding;
//! }
//! ```
//!
//! Direct method access to retained pre-image payload is refused through Host:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn leak_preimage_method(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.retained_preimage();
//! }
//! ```
//!
//! Direct method access to retained governed input is refused through Host:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn leak_governed_input<T: 'static>(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.authority_binding().retained_governed_input::<T>();
//! }
//! ```
//!
//! Direct access to the retained governed-input carrier is refused through Host:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn leak_governed_input_carrier(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.authority_binding().retained_governed_input_carrier();
//! }
//! ```
//!
//! Sealed effect programs do not expose writable effects, read sets, or authored stages:
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgram;
//! fn mutate_effects<S, O, I, P>(program: &mut WorthQueryApplicationEffectProgram<S, O, I, P>) {
//!     program.effects.clear();
//! }
//! ```
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgram;
//! fn read_program_read_set<S, O, I, P>(program: &WorthQueryApplicationEffectProgram<S, O, I, P>) {
//!     let _ = &program.read_set;
//! }
//! ```
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgram;
//! fn mutate_authored_stages<S, O, I, P>(
//!     program: &mut WorthQueryApplicationEffectProgram<S, O, I, P>,
//! ) {
//!     program.authored_stages.clear();
//! }
//! ```
//!
//! Effect program builders and entity targets keep backing fields private:
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder;
//! fn mutate_builder<S, O, I, P>(
//!     builder: &mut WorthQueryApplicationEffectProgramBuilder<S, O, I, P>,
//! ) {
//!     builder.effects.clear();
//! }
//! ```
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectEntity;
//! fn leak_effect_entity<S, E>(entity: &WorthQueryApplicationEffectEntity<S, E>) {
//!     let _ = &entity.reference;
//! }
//! ```
//!
//! Sealed read sets and admitted query plans keep internal authority fields private:
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryCompleteApplicationReadSet;
//! fn leak_read_set<S, O, I, P>(read_set: &WorthQueryCompleteApplicationReadSet<S, O, I, P>) {
//!     let _ = &read_set.admission;
//! }
//! ```
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::primary_graph::WorthQueryAdmittedApplicationQueryPlan;
//! fn leak_query_plan<S, Q, P, R, Pr, Pi, Sc>(
//!     plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, Pr, Pi, Sc>,
//! ) {
//!     let _ = &plan.authorization;
//! }
//! ```
//!
//! Host callers cannot take governance off an admitted query plan:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryAdmittedApplicationQueryPlan;
//! fn strip_governance<S, Q, P, R, Pr, Pi, Sc>(
//!     plan: &mut WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, Pr, Pi, Sc>,
//! ) {
//!     let _ = plan.take_governance();
//! }
//! ```
//!
//! Host callers cannot extract capability disclosure values directly:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::declaration::application_capability::ApplicationCapabilityRequest;
//! use worth_query_host::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
//! fn read_disclosure<S, C, O, I: ApplicationCapabilityRequest<S, C>>(
//!     access: &WorthQueryAdmittedApplicationCapabilityAccess<S, C, O, I>,
//! ) {
//!     let _ = access.disclosure_value();
//! }
//! ```
//!
//! Host callers cannot invoke internal authoring or mutation methods on
//! effect program builders, finished programs, read sets, or specialized wrappers:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder;
//! fn touch_native_effects<S, O, I, P>(
//!     builder: &mut WorthQueryApplicationEffectProgramBuilder<S, O, I, P>,
//! ) {
//!     let _ = builder.native_effects();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgram;
//! fn mutate_program_posture<S, O, I, P>(
//!     program: WorthQueryApplicationEffectProgram<S, O, I, P>,
//! ) {
//!     let _ = program.with_client_observation();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryCompleteApplicationReadSet;
//! fn append_read_set_facts<S, O, I, P>(
//!     read_set: &mut WorthQueryCompleteApplicationReadSet<S, O, I, P>,
//! ) {
//!     let _ = read_set.append_completed_facts(
//!         std::iter::empty(),
//!         worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
//!     );
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryCapabilityRevocationProgram;
//! fn unwrap_revocation<S, O, I, P>(
//!     program: WorthQueryCapabilityRevocationProgram<S, O, I, P>,
//! ) {
//!     let _ = program.into_inner();
//! }
//! ```
//!
//! Host callers cannot mint principals, inbound receipts, or extract raw
//! execution/settled-producer internals through re-exported types:
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryAuthenticatedPrincipal;
//! fn mint_principal<S, P, Id>(
//!     external: worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S, P, Id>,
//! ) {
//!     let _ = WorthQueryAuthenticatedPrincipal::<S, P, Id>::mint;
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryInboundReceipt;
//! fn construct_inbound_receipt() {
//!     let _ = WorthQueryInboundReceipt::from_accepted;
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
//! fn read_performed_inputs(receipt: &WorthQueryApplicationCommitReceipt) {
//!     let _ = receipt.performed_input_facts();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationReadObservation;
//! fn read_settled_producer(observation: &WorthQueryApplicationReadObservation) {
//!     let _ = observation.original_settled_producer();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::runtime::WorthQueryExecutionRuntime;
//! fn extract_generation_counter(runtime: &WorthQueryExecutionRuntime) {
//!     let _ = runtime.retain_current_generation();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::runtime::WorthQueryExecutionRuntime;
//! fn extract_primary_graph(runtime: &mut WorthQueryExecutionRuntime) {
//!     let _ = runtime.primary_graph_mut();
//! }
//! ```
//!
//! ```compile_fail,E0624
//! use worth_query_host::facade::primary_graph::WorthQueryAdvancementPhase;
//! fn extract_execution_request(phase: &WorthQueryAdvancementPhase<'_>) {
//!     let _ = phase.execution_request_for;
//! }
//! ```
