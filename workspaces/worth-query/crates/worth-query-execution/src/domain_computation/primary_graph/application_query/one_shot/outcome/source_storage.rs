//! Preserve the original row-roster request or physical allocation refusal.
use super::{denial, WorthQueryApplicationOneShotDenial, WorthQueryApplicationOneShotDenialKind};
use worth_execution::{ExecutionAllocationDenialClass, ExecutionRequestArrayDenial};

pub(super) fn denied(
    cause: ExecutionRequestArrayDenial,
    query: &str,
) -> WorthQueryApplicationOneShotDenial {
    let request = match cause {
        ExecutionRequestArrayDenial::Request(cause) => cause,
        ExecutionRequestArrayDenial::Allocation(cause) => match cause.classify() {
            ExecutionAllocationDenialClass::Request(request) => request,
            ExecutionAllocationDenialClass::Physical(kind) => {
                return denial(
                    WorthQueryApplicationOneShotDenialKind::SourceObservationAllocation {
                        kind,
                        requested_payload_bytes: cause.requested_payload_bytes(),
                    },
                    query,
                    query,
                )
            }
        },
    };
    super::super::map_authorized_read_denial(
        super::super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial::Read(
            super::super::super::read_execution::relational_read_denial(request.into(), query),
        ),
        query,
    )
}
