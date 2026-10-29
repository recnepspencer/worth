//! Exact World recovery for a branch adoption whose Relational effect already exists.

pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryPreparedBranchAdoption,
};
pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
pub use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;
pub use worth_query_host::facade::runtime::NoEffectCause;
pub use worth_relational::facade::mvcc::{CompanionPreflightStop, RelationalPublicationDeferred};

pub use crate::facade::certification::document_retention_model::host::publish_on_first_program;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::programs::RetentionProgramP1;
pub use crate::facade::certification::document_retention_model::readback::read_retention;
pub use crate::facade::certification::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};

pub const P1_VALUE: u64 = 15;

pub const SAME_HEAD_RETRIES: usize = 64;

/// The only deferral a same-head loser may see: the winner holds the
/// companion reservation while the expected head is still unchanged.
pub fn winner_in_flight(cause: NoEffectCause) -> bool {
    cause
        == NoEffectCause::RelationalDeferred(RelationalPublicationDeferred::CompanionPreflight(
            CompanionPreflightStop::TopologyPending,
        ))
}

pub type Settled = (std::sync::Mutex<bool>, std::sync::Condvar);

pub fn mark_settled((done, changed): &Settled) {
    *done.lock().unwrap() = true;
    changed.notify_all();
}

pub fn await_settled((done, changed): &Settled) {
    let (done, wait) = changed
        .wait_timeout_while(
            done.lock().unwrap(),
            std::time::Duration::from_secs(10),
            |done| !*done,
        )
        .unwrap();
    assert!(
        *done && !wait.timed_out(),
        "the winner settles within its bounded wait"
    );
}

/// A retry prepared after the winner settled reads the moved product.
pub fn retry_saw_moved_head(
    denial: &WorthQueryApplicationProgramAdoptionPreparationDenial,
) -> bool {
    matches!(
        denial,
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::RequirementsChanged
        )
    )
}

#[path = "branch_adoption_recovery/two_same_head_adoptions_report_one_performed_world_transition.rs"]
pub mod two_same_head_adoptions_report_one_performed_world_transition;
