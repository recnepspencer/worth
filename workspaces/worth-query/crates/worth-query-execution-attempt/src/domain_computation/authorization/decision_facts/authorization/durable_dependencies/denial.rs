//! Preserve the native wire owner's resource and control refusal at Query doors.
use crate::domain_computation::authorization::{
    WorkflowApprovalRequestDecodingDenial as Decode,
    WorthQueryOperationAuthorizationDenial as Admission,
};
use crate::domain_computation::primary_graph::{
    workflow::actor, WorthQueryApplicationAttemptDenial as Attempt,
};
use worth_relational::facade::authorization::RelationalAuthorizationDependencyDenial as Native;

pub(super) fn decode(cause: Native) -> Decode {
    match cause {
        Native::Execution(cause) => Decode::Actor(Box::new(Attempt::relational_read_stopped(
            cause,
            "authorization durable dependency wire",
        ))),
        Native::Allocation(cause) => Decode::Actor(Box::new(actor::storage_denial(
            cause,
            "authorization durable dependency wire",
        ))),
        _ => Decode::InvalidRequest,
    }
}

pub(super) fn admission(cause: Native) -> Admission {
    match decode(cause) {
        Decode::Actor(cause) => Admission::workflow_actor_native(*cause),
        Decode::InvalidRequest => Admission::inconsistent("authorization durable dependency wire"),
    }
}
