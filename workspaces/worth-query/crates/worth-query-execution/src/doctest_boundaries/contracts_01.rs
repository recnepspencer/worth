//! Public-boundary doctests batch 01.

/// Restored boundary contract from `basis/product_branch/read_identity.rs` (#1).
///
/// ```compile_fail,E0277
/// fn ordered<T: Ord>() {}
/// ordered::<worth_query_execution::facade::product::WorthQueryProductBranchReadIdentity>();
/// ```
pub struct BoundaryContract001;

/// Restored boundary contract from `basis/product_branch/read_position.rs` (#1).
///
/// ```
/// mod position {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-query-execution-runtime/src/basis/product_branch/read_position.rs"));
/// }
/// fn observe(o: &worth_runtime_world::facade::ProductBranchObservation) {
///     let _ = position::WorthQueryProductBranchReadPosition::from_observation(o);
/// }
/// ```
pub struct BoundaryContract002;

/// Restored boundary contract from `basis/product_branch/read_position.rs` (#2).
///
/// ```compile_fail,E0451
/// mod position {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-query-execution-runtime/src/basis/product_branch/read_position.rs"));
/// }
/// fn splice(a: &worth_runtime_world::facade::ProductBranchObservation,
///           b: &worth_runtime_world::facade::ProductBranchObservation) {
///     let _ = position::WorthQueryProductBranchReadPosition {
///         branch: a.branch_identity().clone(),
///         incarnation: a.lifecycle_incarnation(),
///         generation: a.reference_generation(),
///         commit: b.selected_commit().clone(),
///     };
/// }
/// ```
pub struct BoundaryContract003;

/// Restored boundary contract from `domain_computation/application_aftermath/recovery_progression/dispose.rs` (#1).
///
/// ```compile_fail,E0599
/// fn cannot_copy(denial:worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalDenial) { let _=denial.clone(); }
/// ```
pub struct BoundaryContract004;

/// Restored boundary contract from `domain_computation/application_aftermath/recovery_progression/dispose.rs` (#2).
///
/// ```
/// fn transfer(denial:worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalDenial)->worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalDenial {denial}
/// ```
pub struct BoundaryContract005;

/// Restored boundary contract from `domain_computation/authorization/operation_admission.rs` (#1).
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// let _: WorthQueryAdmittedApplicationOperation<(), (), (), ()> =
///     serde_json::from_str("{}").unwrap();
/// ```
pub struct BoundaryContract006;

/// Restored boundary contract from `domain_computation/authorization/operation_admission.rs` (#2).
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn cannot_extract_session<Schema, Operation, Input, Scope>(
///     admitted: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
/// ) {
///     let _session = admitted.graph_work_session_identity();
/// }
/// ```
pub struct BoundaryContract007;

/// Restored boundary contract from `domain_computation/authorization/operation_admission.rs` (#3).
///
/// ```compile_fail,E0432
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedOperation;
/// ```
pub struct BoundaryContract008;

/// Restored boundary contract from `domain_computation/authorization/operation_admission.rs` (#4).
///
/// ```compile_fail,E0616
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn cannot_take_core<Schema, Operation, Input, Scope>(
///     admitted: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
/// ) {
///     let _core = admitted.core;
/// }
/// ```
pub struct BoundaryContract009;

/// Restored boundary contract from `domain_computation/authorization/operation_admission.rs` (#5).
///
/// ```compile_fail,E0277
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn send<T: Send>() {}
/// send::<WorthQueryAdmittedApplicationOperation<(), (), std::rc::Rc<u8>, ()>>();
/// ```
pub struct BoundaryContract010;

/// Restored boundary contract from `domain_computation/authorization/operation_admission.rs` (#6).
///
/// ```compile_fail,E0277
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn send<T: Send>() {}
/// send::<WorthQueryAdmittedApplicationOperation<(), (), std::cell::Cell<u8>, ()>>();
/// ```
pub struct BoundaryContract011;

/// Restored boundary contract from `domain_computation/authorization/operation_admission/capability_admission/preparation/observation/admitted_access.rs` (#1).
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAuthenticatedPrincipal,
/// };
///
/// fn cannot_substitute_authentication<Schema, Capability, Operation, Input, Principal, Identity>(
///     principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, Identity>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = principal;
/// }
/// ```
pub struct BoundaryContract012;

/// Restored boundary contract from `domain_computation/authorization/operation_admission/capability_admission/preparation/observation/admitted_access.rs` (#2).
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::{
///     ApplicationCapabilityRef, ApplicationCapabilityRequest,
/// };
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_substitute_description<Schema, Capability, Operation, Input>(
///     capability: ApplicationCapabilityRef<Schema, Capability>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = capability;
/// }
/// ```
pub struct BoundaryContract013;

/// Restored boundary contract from `domain_computation/authorization/operation_admission/capability_admission/preparation/observation/admitted_access.rs` (#3).
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_clone_access<Schema, Capability, Operation, Input>(
///     access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _copied = access.clone();
/// }
/// ```
pub struct BoundaryContract014;

/// Restored boundary contract from `domain_computation/authorization/operation_admission/capability_admission/preparation/observation/admitted_access.rs` (#4).
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_extract_lineage<Schema, Capability, Operation, Input>(
///     access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _lineage = access.delegation_lineage();
/// }
/// ```
pub struct BoundaryContract015;

/// Restored boundary contract from `domain_computation/authorization/operation_admission/capability_admission/preparation/observation/admitted_access.rs` (#5).
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::{
///     ApplicationCapabilityProvenanceRef, ApplicationCapabilityRequest,
/// };
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_promote_provenance<Schema, Capability, Operation, Input, Provenance>(
///     provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = provenance;
/// }
/// ```
pub struct BoundaryContract016;

/// Restored boundary contract from `domain_computation/authorization/operation_admission/capability_admission/preparation/observation/admitted_access.rs` (#6).
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     application_capability::ApplicationCapabilityRequest,
///     application_schema::TypedMutationPreconditions,
/// };
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryPrimaryGraphApplicationRuntime,
/// };
/// use worth_query_installation::facade::{
///     ApplicationSchema, WorthQueryInstalledApplicationOperation,
/// };
///
/// fn cannot_change_operation<Schema, Capability, Granted, Other, Input>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Granted, Input>,
///     other: &WorthQueryInstalledApplicationOperation<Schema, Other, Input>,
/// ) where
///     Schema: ApplicationSchema,
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     runtime.authorize_capability_operation(
///         access,
///         other,
///         TypedMutationPreconditions::new(),
///     );
/// }
/// ```
pub struct BoundaryContract017;

/// Restored boundary contract from `domain_computation/execution_runtime/product_world/relational_publication/delivery.rs` (#1).
///
/// ```compile_fail,E0308
/// # use worth_query_execution::facade::{
/// #     application_contribution::WorthQueryAdvancementPhase,
/// #     integration::WorthQueryProductSharedRoot,
/// #     product::WorthQueryPerformedRelationalProductChange,
/// #     provider_session::ExecutionRequest,
/// # };
/// # use worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding;
/// # fn contract(
/// #     root: &WorthQueryProductSharedRoot,
/// #     phase: &WorthQueryAdvancementPhase<'_>,
/// #     request: ExecutionRequest<'_, '_>,
/// #     lowering: &BridgeConditionalSignalBasisBinding,
/// #     change: WorthQueryPerformedRelationalProductChange,
/// # ) {
/// let _ = root.deliver_performed_relational_change(request, lowering, 0, change);
/// # }
/// ```
pub struct BoundaryContract018;

/// Restored boundary contract from `domain_computation/execution_runtime/product_world/relational_publication/delivery.rs` (#2).
///
/// ```
/// # use worth_query_execution::facade::{
/// #     application_contribution::WorthQueryAdvancementPhase,
/// #     integration::WorthQueryProductSharedRoot,
/// #     product::WorthQueryPerformedRelationalProductChange,
/// #     provider_session::ExecutionRequest,
/// # };
/// # use worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding;
/// # fn contract(
/// #     root: &WorthQueryProductSharedRoot,
/// #     phase: &WorthQueryAdvancementPhase<'_>,
/// #     request: ExecutionRequest<'_, '_>,
/// #     lowering: &BridgeConditionalSignalBasisBinding,
/// #     change: WorthQueryPerformedRelationalProductChange,
/// # ) {
/// let _ = root.deliver_performed_relational_change(phase, lowering, 0, change);
/// # }
/// ```
pub struct BoundaryContract019;

/// Restored boundary contract from `domain_computation/primary_graph/application_attempt/commit_terminal.rs` (#1).
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryApplicationCommitTerminalEvidence,
///     WorthQueryApplicationCommitTerminalKind,
/// };
/// use worth_relational::facade::history::BranchId;
///
/// let forged = WorthQueryApplicationCommitTerminalEvidence {
///     kind: WorthQueryApplicationCommitTerminalKind::Executed,
///     branch: BranchId("forged".to_owned()),
///     attempt_resources_released: None,
/// };
/// ```
pub struct BoundaryContract020;

/// Restored boundary contract from `domain_computation/primary_graph/application_attempt/compare_and_commit/cleanup_required/managed.rs` (#1).
///
/// ```compile_fail,E0308
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup;
/// use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
/// use worth_query_execution::facade::domain_computation::WorthQueryMutationGraphWorkCompletion;
/// fn not_business_completion(owner: WorthQueryApplicationManagedCleanup, phase: &WorthQueryAdvancementPhase<'_>) -> WorthQueryMutationGraphWorkCompletion {
///     owner.retry_in_advancement(phase).unwrap()
/// }
/// ```
pub struct BoundaryContract021;

/// Restored boundary contract from `domain_computation/primary_graph/application_attempt/compare_and_commit/cleanup_required/managed.rs` (#2).
///
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanup;
/// use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationManagedCleanupReceipt;
/// fn cleanup(owner: WorthQueryApplicationManagedCleanup, phase: &WorthQueryAdvancementPhase<'_>) -> Result<WorthQueryApplicationManagedCleanupReceipt, WorthQueryApplicationManagedCleanup> {
///     owner.retry_in_advancement(phase)
/// }
/// ```
pub struct BoundaryContract022;
