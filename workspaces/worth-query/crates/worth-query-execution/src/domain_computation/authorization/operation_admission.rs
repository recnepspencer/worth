//! Move-only installed operation admission authority.

use std::marker::PhantomData;

#[path = "operation_admission/capability_admission/mod.rs"]
pub(crate) mod capability_admission;
pub use capability_admission::WorthQueryAdmittedApplicationCapabilityAccess;
pub(in crate::domain_computation) use capability_admission::{
    admit_capability_access, admit_encoded_capability_access,
};
pub(in crate::domain_computation::authorization) use capability_admission::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

use worth_query_declaration::facade::application_operation::{
    ApplicationCanonicalWork, ApplicationSharedInput,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

#[path = "admitted_operation_core.rs"]
mod admitted_operation_core;
pub(in crate::domain_computation) use admitted_operation_core::WorthQueryAdmittedOperation;
pub(in crate::domain_computation) use brand_tag::BrandTag;

#[path = "operation_admission/access.rs"]
pub(crate) mod access;
#[path = "operation_admission/authorization_basis.rs"]
mod authorization_basis;
#[path = "operation_admission/brand_access.rs"]
mod brand_access;
#[path = "operation_admission/brand_custody.rs"]
mod brand_custody;
#[path = "operation_admission/read_completion_loan.rs"]
mod read_completion_loan;
pub(in crate::domain_computation) use read_completion_loan::ReadCompletionAdmission;
#[path = "operation_admission/brand_phases.rs"]
mod brand_phases;
#[path = "operation_admission/brand_tag.rs"]
mod brand_tag;
#[path = "operation_admission/capability_revocation.rs"]
mod capability_revocation;
#[path = "operation_admission/conventional.rs"]
mod conventional;
#[path = "operation_admission/delegation_activation.rs"]
mod delegation_activation;
#[path = "operation_admission/elevation_approval.rs"]
mod elevation_approval;
#[path = "operation_admission/elevation_close.rs"]
mod elevation_close;
#[path = "operation_admission/elevation_request.rs"]
mod elevation_request;
#[path = "operation_admission/graph_work_inspection.rs"]
mod graph_work_inspection;
#[path = "operation_admission/idempotency_binding.rs"]
mod idempotency_binding;
#[path = "operation_admission/identity_admitted.rs"]
mod identity_admitted;
#[path = "operation_admission/installed_reentry.rs"]
mod installed_reentry;
#[path = "operation_admission/mandatory_review.rs"]
mod mandatory_review;
#[path = "operation_admission/original_input.rs"]
mod original_input;
#[path = "operation_admission/output_demand.rs"]
mod output_demand;
#[path = "operation_admission/selected_conventional.rs"]
mod selected_conventional;
pub(in crate::domain_computation) use selected_conventional::{
    authorize_current_control_mutation_on_selected, authorize_public_mutation_on_selected,
    SelectedConventionalAdmissionStop,
};

use authorization_basis::ErasedAuthorizationBasis;

use crate::domain_computation::vocabulary::authorization::operation_admission::RESOURCE_LABEL;

/// Query-owned proof that one installed operation was authorized for one exact
/// current principal and typed scope.
///
/// The proof is move-only, has private fields, and is not deserializable.
/// Descriptive identities, token claims, or decision enums cannot mint it.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// let _: WorthQueryAdmittedApplicationOperation<(), (), (), ()> =
///     serde_json::from_str("{}").unwrap();
/// ```
///
/// Operation-session topology remains owner-private and cannot be used as
/// caller authority:
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
///
/// The erased core behind the proof is not nameable outside the engine, and
/// no proof can be rebuilt around another proof's core:
///
/// ```compile_fail,E0432
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedOperation;
/// ```
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
///
/// The proof is `Send` only when its typed input is `Send + Sync`:
///
/// ```compile_fail,E0277
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn send<T: Send>() {}
/// send::<WorthQueryAdmittedApplicationOperation<(), (), std::rc::Rc<u8>, ()>>();
/// ```
///
/// ```compile_fail,E0277
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn send<T: Send>() {}
/// send::<WorthQueryAdmittedApplicationOperation<(), (), std::cell::Cell<u8>, ()>>();
/// ```
pub struct WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
    core: WorthQueryAdmittedOperation,
    tag: brand_tag::BrandTag,
    // The typed inputs stay beside the core; the core records only which
    // authorization mode and original input identity it carries.
    capability_input: Option<ApplicationSharedInput<Input>>,
    original_input: Option<ApplicationSharedInput<Input>>,
    _marker: PhantomData<fn(Input) -> (Schema, Operation, Scope)>,
}

// An admission is `Send` exactly when its typed input is shared-thread safe,
// as before erasure; the core never narrows it.
const _: fn() = || {
    fn send<T: Send>() {}
    send::<WorthQueryAdmittedApplicationOperation<(), (), u8, ()>>();
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[inline]
    pub(in crate::domain_computation) fn retain_execution_canonical_work(
        &mut self,
        work: WorthQueryCanonicalWorkEvidence,
    ) {
        self.core.retain_execution_canonical_work(work);
    }

    /// Reports the derivations that encoded this request's key and input in
    /// the admission phase, so a fresh commit and a replayed retry both carry
    /// them. Only Publication's request entries report them, once per request.
    #[inline]
    pub fn record_request_identity_work(
        &mut self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        work: ApplicationCanonicalWork,
    ) {
        self.core.record_request_identity_work(work);
    }
}

impl WorthQueryAdmittedOperation {
    fn retain_execution_canonical_work(&mut self, work: WorthQueryCanonicalWorkEvidence) {
        self.canonical_work = self.canonical_work.with_execution_work(work);
    }

    fn record_request_identity_work(&mut self, work: ApplicationCanonicalWork) {
        self.canonical_work = self
            .canonical_work
            .with_admission_work(WorthQueryCanonicalWorkEvidence::streamed_identities(work));
    }
}

pub(in crate::domain_computation::authorization::operation_progression) fn transition_conventional_operation<
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bound: super::super::PreconditionBoundConventionalOperation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
) -> Result<
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    conventional::transition(execution, bound)
}

#[path = "operation_admission/retained_activation_read_tests.rs"]
mod retained_activation_read_tests;

#[path = "operation_admission/request_read_stops_tests.rs"]
mod request_read_stops_tests;

#[path = "operation_admission/forgery_doors.rs"]
mod forgery_doors;
pub use capability_admission::BorrowedCapabilityOps;
pub use capability_admission::OperationProgressionOps;
pub(in crate::domain_computation) use capability_admission::OperationProgressionScopedOps;
