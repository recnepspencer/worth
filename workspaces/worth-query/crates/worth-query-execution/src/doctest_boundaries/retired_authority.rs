//! Public-boundary doctests for retired authority constructors and accessors.

/// Recovery authority and recovery handles require admission.
///
/// ```
/// use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;
/// use worth_query_execution::facade::domain_computation::WorthQueryRecoveryInspectAuthority;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedIdempotencyRead;
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleBinding;
/// use worth_query_execution_runtime::facade::managed_run::WorthQueryRecoveryHandleRegistry;
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandle;
///
/// let _: Option<WorthQueryRecoveryEffectAuthority> = None;
/// let _: Option<WorthQueryRecoveryInspectAuthority> = None;
/// let _: Option<WorthQueryAdmittedIdempotencyRead> = None;
/// let _: Option<WorthQueryRecoveryHandleBinding> = None;
/// let _: Option<WorthQueryRecoveryHandleRegistry> = None;
/// let _: Option<WorthQueryRecoveryHandle> = None;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;
/// let _ = WorthQueryRecoveryEffectAuthority::mint_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::domain_computation::WorthQueryRecoveryInspectAuthority;
/// let _ = WorthQueryRecoveryInspectAuthority::mint_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedIdempotencyRead;
/// let _ = WorthQueryAdmittedIdempotencyRead::mint_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandleBinding;
/// let _ = WorthQueryRecoveryHandleBinding::principal_scope_mut_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_runtime::facade::managed_run::WorthQueryRecoveryHandleRegistry;
/// let _ = WorthQueryRecoveryHandleRegistry::new;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_runtime::facade::managed_run::WorthQueryRecoveryHandleRegistry;
/// let _ = WorthQueryRecoveryHandleRegistry::new_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_runtime::facade::managed_run::WorthQueryRecoveryHandleRegistry;
/// let _ = WorthQueryRecoveryHandleRegistry::register_axis_probe_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandle;
/// let _ = WorthQueryRecoveryHandle::axis_probe_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryRecoveryHandle;
/// let _ = WorthQueryRecoveryHandle::axis_probe_in_registry_for_test;
/// ```
pub struct RecoveryAuthorityBoundaries;

/// Workflow approval and Bridge access follow the installed authority path.
///
/// ```
/// use worth_query_execution_attempt::facade::authorization::WorthQueryWorkflowApprovalAuthorityBasis;
/// use worth_query_execution_attempt::facade::authorization::WorthQueryInstalledAuthorizationRegistry;
/// use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
///
/// let _: Option<WorthQueryWorkflowApprovalAuthorityBasis> = None;
/// let _: Option<WorthQueryInstalledAuthorizationRegistry> = None;
/// let _: Option<WorthQueryApplicationRuntimeCore> = None;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_attempt::facade::authorization::WorthQueryWorkflowApprovalAuthorityBasis;
/// let _ = WorthQueryWorkflowApprovalAuthorityBasis::from_retained_approval_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_attempt::facade::authorization::WorthQueryInstalledAuthorizationRegistry;
/// let _ = WorthQueryInstalledAuthorizationRegistry::bridge_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::WorthQueryApplicationRuntimeCore;
/// let _ = WorthQueryApplicationRuntimeCore::bridge_for_test;
/// ```
pub struct WorkflowAndBridgeAuthorityBoundaries;

/// Entity identities come from projection, and execution requests come from checked advancement.
///
/// ```
/// use worth_query_execution::facade::primary_graph::WorthQueryInvariantEntityIdentity;
/// use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
/// use worth_query_execution_query::facade::domain_computation::primary_graph::TestHostRequest;
///
/// let _: Option<WorthQueryInvariantEntityIdentity<(), ()>> = None;
/// let _: Option<WorthQueryAdvancementPhase<'static>> = None;
/// let _: Option<TestHostRequest<'static>> = None;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryInvariantEntityIdentity;
/// let _ = WorthQueryInvariantEntityIdentity::<(), ()>::new_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
/// let _ = WorthQueryAdvancementPhase::<'static>::execution_request_for_for_test;
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution_query::facade::domain_computation::primary_graph::TestHostRequest;
/// let _ = TestHostRequest::<'static>::execution_request_for_for_test;
/// ```
pub struct IdentityAndRequestAuthorityBoundaries;
