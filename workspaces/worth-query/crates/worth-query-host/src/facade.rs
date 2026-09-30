//! Host-audience view over the production Query authority graph.
//!
//! Program-output publication custody is deliberately absent from this facade;
//! the workspace boundary checker reserves its issuer to the publication owner.

//! Inbound completion has separate declaration, installed source, decoded
//! occurrence, and accepted receipt roles. Only the runtime can issue an
//! installed route or accepted receipt:
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::primary_graph::WorthQueryInboundVerifierHandle;
//! fn forge_source() -> WorthQueryInboundVerifierHandle {
//!     WorthQueryInboundVerifierHandle {
//!         runtime: panic!(), operation: String::new(), installed: panic!(),
//!     }
//! }
//! ```
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryInboundReceipt, WorthQueryInboundReceiptPosture,
//! };
//! fn forge_completion() -> WorthQueryInboundReceipt {
//!     WorthQueryInboundReceipt {
//!         message_identity: [0; 32], envelope_digest: [0; 32],
//!         posture: WorthQueryInboundReceiptPosture::Performed,
//!         pending_reason: None, requires_maintenance_cue: false,
//!     }
//! }
//! ```
//!
//! A product verifier may decode claims, but claims cannot stand in for an
//! owner-accepted completion:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryInboundOccurrenceClaims, WorthQueryInboundReceipt,
//! };
//! fn consume_completion(_: WorthQueryInboundReceipt) {}
//! fn cannot_complete_from_claims(claims: WorthQueryInboundOccurrenceClaims) {
//!     consume_completion(claims);
//! }
//! ```
//!
//! A receipt also cannot author a workflow wait. The wait requires the
//! declared effect binding selected by the workflow definition:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{
//!     declaration::{
//!         application_program::{
//!             ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowInboundWait,
//!             ApplicationWorkflowNodeRef, ApplicationWorkflowOperationNode,
//!             ApplicationWorkflowSpec,
//!         },
//!         application_schema::ApplicationEffectMarkerIdentity,
//!     },
//!     primary_graph::WorthQueryInboundReceipt,
//! };
//! fn cannot_use_completion_as_wait_permit<Spec, Effect>(
//!     builder: &mut ApplicationWorkflowDefinitionBuilder<Spec>,
//!     origin: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
//!     receipt: WorthQueryInboundReceipt,
//! ) where Spec: ApplicationWorkflowSpec,
//!         Effect: ApplicationEffectMarkerIdentity<Spec::Schema> + 'static {
//!     let _ = builder.await_inbound::<Effect>(
//!         "await", origin, receipt, ApplicationWorkflowInboundWait::UntilInstanceDeadline,
//!     );
//! }
//! ```
//!
//! The corresponding public shapes remain usable without internal crates:
//!
//! ```
//! use worth_query_host::facade::{
//!     application_entry::WorthQueryApplicationInboundOccurrencesExt,
//!     declaration::{
//!         application_program::{
//!             ApplicationWorkflowAuthoringDenial, ApplicationWorkflowAwaitInboundNode,
//!             ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowInboundWait,
//!             ApplicationWorkflowNodeRef, ApplicationWorkflowOperationNode,
//!             ApplicationWorkflowSpec,
//!         },
//!         application_schema::{
//!             ApplicationEffectMarkerIdentity, ApplicationInboundOccurrenceBinding,
//!         },
//!     },
//!     domain::ApplicationSchema,
//!     primary_graph::{
//!         WorthQueryInboundOccurrenceClaims, WorthQueryInboundReceipt,
//!         WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime,
//!     },
//! };
//! fn receive_from_installed_source<Schema: ApplicationSchema>(
//!     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
//!     source: &WorthQueryInboundVerifierHandle,
//!     bytes: &[u8],
//!     scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
//! ) -> Result<WorthQueryInboundReceipt, worth_query_host::facade::primary_graph::WorthQueryInboundAdmissionDenial> {
//!     runtime.inbound_occurrences().receive(source, bytes, scope).execute()
//! }
//! fn declared_wait<Spec, Effect>(
//!     builder: &mut ApplicationWorkflowDefinitionBuilder<Spec>,
//!     origin: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
//!     binding: ApplicationInboundOccurrenceBinding<Effect>,
//! ) -> Result<ApplicationWorkflowNodeRef<ApplicationWorkflowAwaitInboundNode>, ApplicationWorkflowAuthoringDenial>
//! where Spec: ApplicationWorkflowSpec,
//!       Effect: ApplicationEffectMarkerIdentity<Spec::Schema> + 'static {
//!     builder.await_inbound("await", origin, binding, ApplicationWorkflowInboundWait::UntilInstanceDeadline)
//! }
//! fn decoded_claims_are_data(_: WorthQueryInboundOccurrenceClaims) {}
//! ```
//!
//! The advanced path progresses through sealed phases. Neither authenticated
//! bytes nor owner correlation can execute a completion before finite custody:
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::{
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryAuthenticatedInboundOccurrence,
//! };
//! fn forge_authenticated<'a, Schema: ApplicationSchema>() -> WorthQueryAuthenticatedInboundOccurrence<'a, Schema> {
//!     WorthQueryAuthenticatedInboundOccurrence {
//!         runtime: panic!(), operation: String::new(), installed: panic!(),
//!         envelope: &[], claims: panic!(),
//!     }
//! }
//! ```
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::{
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryAdmittedInboundOccurrence,
//! };
//! fn forge_admitted<'a, Schema: ApplicationSchema>() -> WorthQueryAdmittedInboundOccurrence<'a, Schema> {
//!     WorthQueryAdmittedInboundOccurrence {
//!         runtime: panic!(), envelope: &[], admission: panic!(),
//!     }
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     admission::authenticated_principal::WorthQueryRequestScope,
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryAuthenticatedInboundOccurrence,
//! };
//! fn cannot_execute_authenticated<Schema: ApplicationSchema>(
//!     phase: WorthQueryAuthenticatedInboundOccurrence<'_, Schema>,
//!     scope: &WorthQueryRequestScope,
//! ) {
//!     let _ = phase.execute(scope);
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     admission::authenticated_principal::WorthQueryRequestScope,
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryCorrelatedInboundOccurrence,
//! };
//! fn cannot_execute_correlated<Schema: ApplicationSchema>(
//!     phase: WorthQueryCorrelatedInboundOccurrence<'_, Schema>,
//!     scope: &WorthQueryRequestScope,
//! ) {
//!     let _ = phase.execute(scope);
//! }
//! ```
//!
//! The valid counterpart starts at the installed runtime and carries each
//! phase's result into the next operation:
//!
//! ```
//! use worth_query_host::facade::{
//!     admission::authenticated_principal::WorthQueryRequestScope,
//!     domain::ApplicationSchema,
//!     primary_graph::{
//!         WorthQueryInboundAdmissionDenial, WorthQueryInboundReceipt,
//!         WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime,
//!     },
//! };
//! fn receive_in_phases<'a, Schema: ApplicationSchema>(
//!     runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
//!     source: &WorthQueryInboundVerifierHandle,
//!     bytes: &'a [u8],
//!     scope: &WorthQueryRequestScope,
//! ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
//!     runtime.authenticate_inbound_occurrence(source, bytes)?
//!         .correlate()?.accept()?.execute(scope)
//! }
//! ```

pub use worth_query_admission::facade as admission;
pub use worth_query_declaration::facade as declaration;
pub use worth_query_declaration::{
    worth_query_application, worth_query_application_contribution, worth_query_application_query,
    worth_query_application_schema, worth_query_aspect, worth_query_effect, worth_query_entity,
    worth_query_field, worth_query_operation, worth_query_operation_emits,
    worth_query_operation_reads, worth_query_operation_writes, worth_query_portable_type,
    worth_query_principal_binding, worth_query_relation, worth_query_structured_value_binding,
    worth_query_workflow,
};
pub use worth_query_execution::facade::application_contribution;
pub use worth_query_execution::facade::application_discovery;
pub use worth_query_execution::facade::application_installation;
pub use worth_query_execution::facade::application_invariants;
pub use worth_query_execution::facade::convergence_epoch;
pub use worth_query_execution::facade::installed;
pub use worth_query_execution::facade::primary_graph;
pub use worth_query_execution::facade::product;
pub use worth_query_execution::facade::provisional_aftermath;
pub use worth_query_execution::facade::runtime;
pub use worth_query_installation::facade as domain;
pub use worth_query_installation::facade::{
    inspect_installed_graph_obligations, WorthQueryGraphObligationAdoptionDenial,
    WorthQueryGraphObligationAdoptionDenialKind, WorthQueryGraphObligationAdoptionProof,
    WorthQueryGraphObligationAdoptionRow,
};
pub use worth_query_installation::worth_query_conditional_node;
pub use worth_query_publication::facade as publication;
pub use worth_query_publication::facade::application_entry;
