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
