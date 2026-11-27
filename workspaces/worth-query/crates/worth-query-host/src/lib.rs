//! Query host audience facade.
//!
//! Start with [Build an Application](https://github.com/recnepspencer/worth/blob/master/docs/build-an-application.md)
//! and the [Capability Index](https://github.com/recnepspencer/worth/blob/master/docs/capability-index.md).
//!
//! Admission, lowering, execution, and publication consumers depend on this crate instead of
//! importing Query's internal authority packages directly. The read-only [`facade::domain`]
//! surface exposes an installed schema's `native_contracts()`, an installed operation's typed
//! `graph_reads()` and `touches()`, and aftermath inspection through `authority()`, `recovery()`,
//! `reconciliation()`, and `external_effect().correlation_family()`. These accessors inspect
//! retained meaning; they grant no installation, execution, correction or recovery authority.
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
//! # const fn _receipt_const_readers(
//! #     receipt: &primary_graph::WorthQueryApplicationCommitReceipt,
//! #     approved: &primary_graph::WorthQueryApprovedElevation,
//! # ) {
//! #     let _ = (
//! #         receipt.outcome_identity(), receipt.provider_runtime_instance_id(),
//! #         receipt.commit_reference(), receipt.committed_product_publication(),
//! #         receipt.basis_descriptor(), receipt.changed_record_count(),
//! #         receipt.emitted_effect_count(), receipt.committed_changes(),
//! #         receipt.precondition_comparison(), receipt.canonical_work(),
//! #         receipt.terminal(), receipt.external_dispatch(),
//! #         receipt.external_dispatch_preparation_denial(), receipt.authority_binding(),
//! #         receipt.installed_operation(), receipt.installed_recovery_contract(),
//! #         receipt.published_aftermath_posture(), receipt.principal_scope(),
//! #         receipt.idempotency_binding(), receipt.has_retained_preimage(),
//! #         receipt.aftermath_causality(), approved.approval_product_publication(),
//! #     );
//! # }
//! # fn _open_refused_payloads<S: worth_query_host::facade::declaration::application_schema::ApplicationSchemaComposition, P>(
//! #     refused: worth_query_host::facade::WorthQueryApplicationHomeOpenRefused<S, P>,
//! # ) {
//! #     match *refused.into_phase() {
//! #         worth_query_host::facade::WorthQueryApplicationHomeOpenRefusedPhase::Binding {
//! #             cause, ..
//! #         } => { let _: domain::WorthQueryHomeBindingDenial = cause; }
//! #         _ => {}
//! #     }
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
//!     phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
//! ) {
//!     let _ = runtime.on_branch(branch).select(phase);
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
//!
//! Program publication cannot skip its source-preparation phase. The default
//! idempotent mutation request is intentionally missing `execute_performed`:
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::advanced::WorthQueryApplicationMutationRequestWithIdempotency,
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_operation::ApplicationMutationIntent,
//!     domain::ApplicationSchema,
//! };
//!
//! fn cannot_publish_without_source<'a, Schema, Intent, Program, Root>(
//!     request: WorthQueryApplicationMutationRequestWithIdempotency<
//!         'a, 'a, 'a, 'a, Schema, Intent,
//!     >,
//!     application: &'a WorthQueryProgramApplicationRuntime<Schema, Program>,
//! ) where
//!     Schema: ApplicationSchema,
//!     Intent: ApplicationMutationIntent<Schema>,
//! {
//!     let _ = request.execute_performed::<Program, Root>(application);
//! }
//! ```
//!
//! Its valid type-level twin carries the phase returned by `expect_source`:
//!
//! ```
//! use worth_query_host::facade::{
//!     application_entry::{
//!         advanced::WorthQueryApplicationMutationRequestWithIdempotency,
//!         WorthQueryMutationSourcePrepared,
//!     },
//!     declaration::application_operation::ApplicationMutationIntent,
//!     domain::ApplicationSchema,
//! };
//!
//! fn accepts_prepared_source<'a, Schema, Intent>(
//!     request: WorthQueryApplicationMutationRequestWithIdempotency<
//!         'a, 'a, 'a, 'a, Schema, Intent, WorthQueryMutationSourcePrepared,
//!     >,
//! ) where
//!     Schema: ApplicationSchema,
//!     Intent: ApplicationMutationIntent<Schema>,
//! {
//!     drop(request);
//! }
//! ```
//!
//! Recovery likewise accepts the installed connection's typed root demand;
//! text with the same visible identity cannot call the production handoff:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{
//!     application_contribution::{WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily},
//!     application_entry::{WorthQueryApplicationRequest, WorthQueryOutputDemandControls},
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::{
//!         application_program::{ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition},
//!         application_query::{ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution},
//!         application_schema::ApplicationStructuredValueBinding,
//!     },
//!     domain::ApplicationSchema,
//!     primary_graph::{WorthQueryApplicationCommitReceipt, WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection},
//! };
//! type RootRef<S, R> = <R as ApplicationOutputGraphShape<S>>::RootConnection;
//! type RootConnection<S, R> = <RootRef<S, R> as ApplicationConnectionShape<S>>::Binding;
//! type RootDemand<S, R> = <RootConnection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
//! type RootFamily<S, R> = <RootDemand<S, R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
//! type RootSource<S, R> = <RootFamily<S, R> as WorthQueryProducerOutputFamily<S>>::Source;
//! type RootQuery<S, R> = <RootSource<S, R> as ApplicationQueryBinding<S>>::Query;
//! type RootValue<S, R> = <<RootSource<S, R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
//! fn cannot_recover_by_name<'a, S, P, R>(
//!     request: &WorthQueryApplicationRequest<'a, 'a, 'a, S>,
//!     application: &'a WorthQueryProgramApplicationRuntime<S, P>,
//!     receipt: &WorthQueryApplicationCommitReceipt,
//!     controls: WorthQueryOutputDemandControls,
//! ) where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S>,
//!     RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
//!     RootDemand<S, R>: WorthQueryApplicationOutputDemand<S> + Clone,
//!     <RootSource<S, R> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S, Binding = RootSource<S, R>>,
//!     <RootSource<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S, <RootSource<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
//!     RootValue<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
//! {
//!     let _ = request.advanced().recover_required_outputs::<P, R>(application, receipt, "root-output", controls);
//! }
//! ```
//!
//! The passing twin derives that demand from the explicitly selected program root. The
//! concrete production journeys below call the recovery handoff with this type:
//!
//! ```
//! use worth_query_host::facade::{
//!     declaration::{
//!         application_program::{ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition},
//!     },
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryApplicationRequiredOutputConnection,
//! };
//! type RootRef<S, R> = <R as ApplicationOutputGraphShape<S>>::RootConnection;
//! type RootConnection<S, R> = <RootRef<S, R> as ApplicationConnectionShape<S>>::Binding;
//! type RootDemand<S, R> = <RootConnection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
//! fn accepts_installed_root<S, P, R>(_: RootDemand<S, R>) where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S>,
//!     RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S> {}
//! ```
//!
//! The low owner entry cannot be reached with consumer-assembled rows and a
//! disclosure token. A completed Query lane alone creates their paired source:
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryApplicationOutputDemandDisclosure,
//!     WorthQueryApplicationOutputDemandSource,
//! };
//!
//! fn cannot_pair_rows_with_disclosure<Query, Value>(
//!     disclosure: WorthQueryApplicationOutputDemandDisclosure<Query>,
//! ) -> WorthQueryApplicationOutputDemandSource<Query, Value> {
//!     WorthQueryApplicationOutputDemandSource {
//!         rows: Vec::new(),
//!         disclosure,
//!     }
//! }
//! ```

//! Local scope borrows can execute through a Query-issued dependent read context.
//!
//! ```
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryDependentPairReadContext, WorthQueryAdmittedApplicationQueryPlan,
//!     WorthQueryApplicationProjection, WorthQueryManagedDerivedViewDenial,
//! };
//! use worth_query_host::facade::domain::ApplicationSchema;
//! fn execute_branch<S: ApplicationSchema, Q, P, R, A, I, C>(
//!     read: &mut WorthQueryDependentPairReadContext<'_, '_, '_, '_, '_, S, Q, R>,
//!     plan: WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
//! ) -> Result<(), WorthQueryManagedDerivedViewDenial>
//! where R: WorthQueryApplicationProjection<S, Q> {
//!     let observed = read.execute(plan)?;
//!     let _receipt = observed.receipt();
//!     Ok(())
//! }
//! fn independent_in_one_loan<'a, S, Q, P, R, A, I, C>(
//!     first: WorthQueryAdmittedApplicationQueryPlan<'a, S, Q, P, R, A, I, C>,
//!     second: WorthQueryAdmittedApplicationQueryPlan<'a, S, Q, P, R, A, I, C>,
//!     batch: &worth_query_host::facade::primary_graph::WorthQueryApplicationQueryBatchAdmission,
//! ) -> worth_query_host::facade::primary_graph::WorthQueryDerivedPairReadPlans<'a, S, Q, P, R, Q, P, R, A, I, C> {
//!     worth_query_host::facade::primary_graph::WorthQueryDerivedPairReadPlans::in_batch(first, second, batch)
//! }
//! ```
//!
//! ```compile_fail
//! use worth_query_host::facade::primary_graph::WorthQueryDependentPairReadContext;
//! // Ordinary code must receive this context from Query's reconstruction owner.
//! let forged = WorthQueryDependentPairReadContext::new();
//! ```

mod access_fragment_contract;
mod application_home;
mod audit;
mod background;
mod capability_command_contract;
mod causal_inspection_contract;
mod closed_delivery_contract;
mod collection_contract;
mod correction_facade_contract;
mod cutover_contract;
mod disclosure_and_program_fences_contract;
mod discovered_currentness_contract;
pub mod facade;
mod funded_input_contract;
mod local_calendar;
mod logical_member_contract;
mod no_source_tree_contract;
mod ordered_index;
mod ordinary_contract;
mod principal_registry;
mod producer_stop_contract;
mod program_transition;
mod projection_derive_contract;
mod projection_name_derive_contract;
mod projection_relation_derive_contract;
mod refusal_summary;
mod runtime_and_handler_fences_contract;
mod semantic_refusal_contract;
mod session;
mod session_contract;
mod shared_access;
mod source_custody;
mod source_custody_contract;
mod storage_retention_contract;
mod temporal_contribution;
mod workflow_actor_contract;
mod workflow_owner_contract;
