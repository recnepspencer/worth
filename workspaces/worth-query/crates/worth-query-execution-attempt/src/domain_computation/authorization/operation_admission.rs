//! Move-only installed operation admission authority.
#[path = "admitted_operation_core.rs"]
pub mod admitted_operation_core;

use std::marker::PhantomData;


use worth_query_declaration::facade::application_operation::{
    ApplicationCanonicalWork, ApplicationSharedInput,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

pub use admitted_operation_core::{
    WorthQueryAdmittedOperation, WorthQueryAdmittedOperationData,
};
pub use brand_tag::BrandTag;

#[path = "operation_admission/access.rs"]
pub mod access;
#[path = "operation_admission/authorization_basis.rs"]
pub mod authorization_basis;
#[path = "operation_admission/brand_access.rs"]
pub mod brand_access;
#[path = "operation_admission/brand_custody.rs"]
pub mod brand_custody;
#[path = "operation_admission/read_completion_loan.rs"]
pub mod read_completion_loan;
pub use read_completion_loan::ReadCompletionAdmission;
#[path = "operation_admission/brand_phases.rs"]
pub mod brand_phases;
#[path = "operation_admission/brand_tag.rs"]
pub mod brand_tag;
#[path = "operation_admission/capability_revocation.rs"]
pub mod capability_revocation;
#[path = "operation_admission/delegation_activation.rs"]
pub mod delegation_activation;
#[path = "operation_admission/elevation_approval.rs"]
pub mod elevation_approval;
#[path = "operation_admission/elevation_close.rs"]
pub mod elevation_close;
#[path = "operation_admission/elevation_request.rs"]
pub mod elevation_request;
#[path = "operation_admission/graph_work_inspection.rs"]
pub mod graph_work_inspection;
#[path = "operation_admission/idempotency_binding.rs"]
pub mod idempotency_binding;
#[path = "operation_admission/identity_admitted.rs"]
pub mod identity_admitted;
#[path = "operation_admission/incarnation.rs"]
pub mod incarnation;
#[path = "operation_admission/installed_reentry.rs"]
pub mod installed_reentry;
#[path = "operation_admission/mandatory_review.rs"]
pub mod mandatory_review;
#[path = "operation_admission/original_input.rs"]
pub mod original_input;
#[path = "operation_admission/output_demand.rs"]
pub mod output_demand;

pub use authorization_basis::ErasedAuthorizationBasis;

const RESOURCE_LABEL: &str = "worth-query-application-admission:";
pub use worth_query_execution_contracts::facade::WorthQueryOperationAdmissionIdentity;

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
    pub fn record_request_identity_work_inner(
        &mut self,
        work: worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    ) {
        self.core.record_request_identity_work(work);
    }

    pub fn record_request_identity_work(
        &mut self,
        _access: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
        work: worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    ) {
        self.core.record_request_identity_work(work);
    }

    #[inline]
    pub fn retain_execution_canonical_work(
        &mut self,
        work: WorthQueryCanonicalWorkEvidence,
    ) {
        self.core.retain_execution_canonical_work(work);
    }

}

impl WorthQueryAdmittedOperation {
    fn retain_execution_canonical_work(&mut self, work: WorthQueryCanonicalWorkEvidence) {
        self.data.canonical_work = self.data.canonical_work.with_execution_work(work);
    }

    fn record_request_identity_work(&mut self, work: ApplicationCanonicalWork) {
        self.data.canonical_work = self
            .data
            .canonical_work
            .with_admission_work(WorthQueryCanonicalWorkEvidence::streamed_identities(work));
    }
}



#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::WorthQueryOperationAdmissionIdentity;

    #[test]
    fn operation_admission_identity_exhaustion_cannot_wrap() {
        let counter = AtomicU64::new(u64::MAX);
        assert!(WorthQueryOperationAdmissionIdentity::mint_from(&counter).is_none());
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), u64::MAX);
    }
}

#[path = "operation_admission/forgery_doors.rs"]
pub mod forgery_doors;
#[path = "operation_admission/request_read_stops_tests.rs"]
mod request_read_stops_tests;
#[path = "operation_admission/retained_activation_read_tests.rs"]
mod retained_activation_read_tests;
