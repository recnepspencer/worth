//! Host-audience view over the production Query authority graph.
//!
//! Start with the unified application facade:
//! - Open or reopen a persistent Home in one call via [`WorthQueryApplicationHome::open_directory`]
//!   with [`WorthQueryHomeConfig::desktop_default`], check [`WorthQueryApplicationHome::open_disposition`],
//!   and retire cleanly with [`WorthQueryApplicationHome::close_clean`].
//! - Create a reusable session via [`WorthQueryApplicationHome::session`] (`home.session(me)`)
//!   with bounded defaults (`DEFAULT_HOST_EXECUTION_POLICY`) and run actions (`session.run(..)`),
//!   typed queries (`session.query(..)`), live subscriptions ([`WorthQueryTypedLiveSubscription`]),
//!   managed-index watches and readiness (`session.watch(..)`, `session.ensure_index_ready(..)`).
//! - Fork and adopt "what-if" branches ([`WorthQueryApplicationHome::fork_preview`],
//!   [`WorthQueryApplicationHome::adopt_preview`]). Audit reads are in [`audit`].
//! - Show any refusal to an end user with [`WorthQueryUserFacingRefusal`]
//!   (`denial.user_message()`), keeping `{denial:?}` for developer logs.
//!
//! Ordinary action owners retain the authentic installed runtime and complete installed action
//! beyond construction. Applications keep their own runtime handle for fresh requests; operation
//! keepalive grants no admission or cleanup. Bind initial workflow vocabulary before retaining
//! handles, and preserve runtime/spec/signing custody when configuration refuses.
//!
//! Raw-part `from_untrusted*` archive decoders are sealed off this audience: each demands an
//! internal witness that neither this facade nor its declaration re-export names.
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::declaration::archive_decode_authority::ARCHIVE_DECODE;
//! ```
//!
//! ```compile_fail,E0277
//! use worth_query_host::facade::application_installation::WorthQueryApplicationCheckpoint;
//! fn forge(bytes: Vec<u8>) -> WorthQueryApplicationCheckpoint {
//!     WorthQueryApplicationCheckpoint::from_untrusted_bytes(Default::default(), bytes)
//! }
//! ```
//!
//! Transported checkpoints readmit through `WorthQueryApplicationCheckpoint::readmit_transported`
//! under their capture origin; transports describe correction sources with typed coordinates:
//!
//! ```
//! use worth_query_host::facade::primary_graph::{
//!     BranchId, CommitId, RelationalCommitReceipt, VersionId,
//!     WorthQueryCorrectionSourceCoordinates, WorthQueryCorrectionSourceMember,
//! };
//! let coordinates = WorthQueryCorrectionSourceCoordinates::new(
//!     worth_foundational::WorldOwnerCoordinate::MemoryOnly { observation: 7 },
//!     19,
//!     RelationalCommitReceipt {
//!         commit_id: CommitId(23),
//!         version_id: VersionId(29),
//!         branch_id: BranchId("product:fork".into()),
//!         parents: vec![CommitId(17)],
//!     },
//!     WorthQueryCorrectionSourceMember::Primary,
//! );
//! assert_eq!(coordinates.relational().commit_id, CommitId(23));
//! ```
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
    worth_query_access_fragment, worth_query_application_query,
    worth_query_authentication_audience, worth_query_authentication_method, worth_query_capability,
    worth_query_capability_context, worth_query_capability_provenance,
    worth_query_disclosure_classification, worth_query_effect, worth_query_identity_namespace,
    worth_query_operation, worth_query_operation_emits, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_portable_type, worth_query_principal_binding,
    worth_query_principal_issuer, worth_query_structured_value_binding, worth_query_workflow,
};
/// Explain saved query results and authored source operations: start with
/// [`application_entry::WorthQueryApplicationRequest::inspect`] or `inspect_source`. Each
/// execution checks current inspection permission; a saved original keeps no grant.
pub use worth_query_execution::facade::application_inspection as causal_inspection;
pub mod audit;
pub use worth_query_execution::facade::{application_contribution, application_discovery};
pub use worth_query_execution::facade::{application_installation, application_invariants};
pub mod application_naming;
pub mod local_calendar;
pub mod ordered_index;
/// Shared scope access grants and authenticated single-owner remote transport.
pub mod shared_access;
pub use crate::application_home::{
    discard_home, ApplicationHomeQueryRootDenial, HomeCustodyReadmissionDenial,
    NoHomeProgramTransition, QueryHomeProgramActivationDenial, UnadmittedHomeQueryCustodyRefused,
    UnadmittedHomeQueryService, WorldCompositionRetained, WorthQueryApplicationHome,
    WorthQueryApplicationHomeCheckpoint, WorthQueryApplicationHomeCheckpointDenial,
    WorthQueryApplicationHomeCheckpointInspection,
    WorthQueryApplicationHomeCheckpointJournalObservation,
    WorthQueryApplicationHomeCheckpointLimits, WorthQueryApplicationHomeCloseOutcome,
    WorthQueryApplicationHomeClosePhase, WorthQueryApplicationHomeCloseRefused,
    WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeDiscardDenial,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeOpenRefusedPhase, WorthQueryApplicationHomeOpeningCloseRefused,
    WorthQueryApplicationHomeOpeningClosed, WorthQueryApplicationHomeRequest,
    WorthQueryApplicationHomeTransitionRequired, WorthQueryDirectoryOpenDenial,
    WorthQueryDirectoryOpenRetired, WorthQueryHomeClosePolicyRefused, WorthQueryHomeConfig,
    WorthQueryHomeOpenDisposition,
    WorthQueryHomeProgramTransition, WorthQueryHomeProgramTransitionDenial,
    WorthQueryHomeProgramTransitionOpenPhase, WorthQueryHomeProgramTransitionOpenRefused,
    WorthQueryHomeProgramTransitionPlan, WorthQueryHomeProgramTransitionStep,
    WorthQueryHomeRetirement, WorthQueryHomeRetirementStop, WorthQueryPreviewAdoptDenial,
    WorthQueryPreviewAdoptFailure, WorthQueryPreviewDiscardDenial, WorthQueryPreviewDiscardFailure,
    WorthQueryPreviewFork, WorthQueryPreviewForkDenial, WorthQueryWorkflowApplicationHome,
    WorthQueryWorkflowHomeBindingRefused, WorthQueryWorkflowHomeCloseRefused,
    WorthQueryWorkflowHomeClosed, DEFAULT_HOST_EXECUTION_POLICY,
    DEFAULT_HOST_EXECUTION_WORK_CEILING,
};
/// Non-blocking owned-session calls and push commit notification for a Home.
pub use crate::background::{
    call_worker::DEFAULT_BACKGROUND_QUEUE_CAPACITY,
    commit_watch::{
        WorthQueryCommitWatchDenial, WorthQueryCommitWatchGuard, WorthQueryCommittedBasisVersion,
    },
    pending::{WorthQueryBackgroundDenial, WorthQueryPending},
};
/// Runtime principal onboarding refusals for `worth_query_principal_registry!`.
pub mod principal_registry {
    pub use crate::principal_registry::WorthQueryPrincipalRegistryRefusal;
}
/// Fact-free "who may act?": verdicts from the installed policy decisions, no fact retained.
pub mod eligibility;
/// Reusable authenticated Host session and typed whole-result / managed-index live handles.
pub use crate::session::{
    WorthQueryAutoIdempotencyKey, WorthQueryHostSession, WorthQueryLocalSessionDenial,
    WorthQueryOwnedLiveSubscription, WorthQueryOwnedSession, WorthQueryOwnedSessionDenial,
    WorthQueryOwnedWorkflowSession, WorthQueryPage, WorthQueryPageCursor, WorthQuerySessionCommit,
    WorthQuerySessionIndexedQuery, WorthQuerySessionPrincipalSource, WorthQuerySessionQuery,
    WorthQuerySessionRunDenial, WorthQuerySessionSnapshot, WorthQuerySessionSnapshotDenial,
    WorthQueryTypedLiveCloseDenial, WorthQueryTypedLiveRefreshDenial,
    WorthQueryTypedLiveSubscription, WorthQueryTypedManagedIndexRefreshDenial,
    WorthQueryTypedManagedIndexWatch,
};
pub use crate::{
    worth_query_principal_registry, worth_query_recurring_rule, worth_query_remote_codec,
    worth_query_scope_access, worth_query_shared_access,
};
/// Read-only observations for facade-only certification; these reports grant no authority.
pub use worth_query_execution::facade::{
    certification, convergence_epoch, installed, primary_graph, product, runtime,
    worth_query_application_contribution, worth_query_application_program,
};
pub use worth_query_installation::facade as domain;
pub use worth_query_installation::facade::{
    inspect_installed_graph_obligations, WorthQueryGraphObligationAdoptionDenial,
    WorthQueryGraphObligationAdoptionDenialKind, WorthQueryGraphObligationAdoptionProof,
    WorthQueryGraphObligationAdoptionRow,
};
pub use worth_query_installation::worth_query_conditional_node;
pub use worth_query_macros::*;
pub use worth_query_publication::facade as publication;
pub use worth_query_publication::facade::application_entry;
/// Relational data for a Home transition author, without owner authority.
pub mod home_program_transition;
/// Declarative Directory Home program transitions (`worth_query_program_transition!`),
/// single-call open-with-transition helpers, and package-default build identity resolution.
pub mod program_transition;
/// Existing owner refusal vocabulary for retained Home custody and Directory checkpoints.
pub use crate::application_home::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseDenial};
/// The typed physical causes returned by a Home's `physical_refusal`.
pub use crate::application_home::{
    ApplicationHomePhysicalRefusal, ApplicationHomePhysicalRefusalKind,
    ApplicationHomePhysicalStage,
};
/// End-user refusal summaries: `denial.user_message()` for UIs, `{denial:?}` for logs;
/// an application rule's own structured refusal: `denial.domain_refusal()`.
pub use crate::refusal_summary::{
    WorthQueryDomainRefusal, WorthQueryDomainRefusalCarrier, WorthQueryDomainRefusalDenial,
    WorthQueryRefusalCategory, WorthQueryRefusalSummary, WorthQueryUserFacingRefusal,
    WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES,
    WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES,
};
/// Interactive custody of one source-only command or correction across frames.
pub use crate::source_custody::{
    WorthQueryAnySourceCompletion,
    WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep,
    WorthQueryAnyTerminalSourceEvidence,
    WorthQuerySourceCustody, WorthQuerySourceCustodyPending, WorthQuerySourceCustodyPhase,
    WorthQuerySourceCustodyStep, WorthQuerySourceCustodyWithdrawal,
};
pub mod temporal_contribution;
pub use worth_foundational::facade::{CanonicalDigestId, DurableOwnerFamily};
/// Typed refusal from fresh authored-program installation before Home service.
pub use worth_query_execution::facade::application_home_integration::QueryHomeSignalProgramPreparationDenial;
pub use worth_runtime_world::facade::{
    RuntimeWorldBudgetDenial, RuntimeWorldBudgetResource, RuntimeWorldCloseDenial,
    RuntimeWorldServiceDenial,
};
