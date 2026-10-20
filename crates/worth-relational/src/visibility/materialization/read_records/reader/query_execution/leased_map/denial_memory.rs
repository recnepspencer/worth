//! Charge the original native refusal without mirroring its topology in Query.
use super::{ChargedBytes, MapKernelFailure, MapStop, QueryReadExecutionStop};

// Copy metadata carries no owned backing. Losing that contract makes this
// source owner account the new nested allocation before its callers compile.
pub(super) fn inline<T: Copy>(_: &T) -> u64 {
    0
}

impl ChargedBytes for QueryReadExecutionStop {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::ReadRequest(cause) => inline(cause),
            Self::Request(denial) => match denial {
                worth_execution::WorkCeilingDenial::Admission(denial) => inline(denial),
                worth_execution::WorkCeilingDenial::Stopped(stop) => inline(stop),
                worth_execution::WorkCeilingDenial::Panicked => 0,
            },
            Self::PreparationAdmission(denial) | Self::CompletionAdmission(denial) => {
                inline(denial)
            }
            Self::PreparationLease(denial) => inline(denial),
            Self::PacketAdmission(denial) => inline(denial),
            Self::PreparationStopped { reason, report }
            | Self::CompletionStopped { reason, report } => inline(report) + stop_bytes(reason),
            Self::PacketStopped {
                boundary,
                reason,
                report,
            } => inline(boundary) + inline(report) + stop_bytes(reason),
        }
    }
}
fn stop_bytes<E: ChargedBytes>(stop: &MapStop<E>) -> u64 {
    match stop {
        MapStop::Admission(denial) => inline(denial),
        MapStop::WorkExhausted { identity } => inline(identity),
        MapStop::Failure { identity, cause } => {
            let fixed = inline(identity);
            fixed
                + match cause {
                    MapKernelFailure::Stop(stop) => inline(stop),
                    MapKernelFailure::Domain(error) => error.additional_charged_bytes(),
                    MapKernelFailure::Panic | MapKernelFailure::ResultCapacityExceeded => 0,
                }
        }
    }
}
