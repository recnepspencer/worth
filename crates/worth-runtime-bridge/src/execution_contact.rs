//! Admission of one Bridge source or delivery contact under the caller's request.
use crate::error::BridgeExecutionDenial;
use worth_execution::{
    ChargedBytes, ExecutionRequest, ExecutionScan, MapKernelFailure, MapStop, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

enum ContactFailure {}
impl ChargedBytes for ContactFailure {
    fn additional_charged_bytes(&self) -> u64 {
        match *self {}
    }
}

/// The dispatch contact is an actual counted operation, rather than a bound
/// reserved for a callback whose work is charged by its own owner.
pub(crate) fn admit(request: ExecutionRequest<'_, '_>) -> Result<(), BridgeExecutionDenial> {
    request
        .in_scope(|lease| {
            let identity = PartitionIdentity::new(1);
            let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
                .expect("one fixed canonical Bridge contact");
            match scan.run(lease, (), 0, 0, 0, 0, |_, _, work| {
                work.checkpoint(1).map_err(MapKernelFailure::Stop)?;
                Ok::<_, MapKernelFailure<ContactFailure>>(((), ()))
            }) {
                ScanOutcome::Complete { .. } => Ok(()),
                ScanOutcome::Stopped { reason, .. } => match reason {
                    MapStop::Admission(cause) => Err(cause.into()),
                    MapStop::WorkExhausted { .. } => Err(BridgeExecutionDenial::WorkCeiling),
                    MapStop::Failure { cause, .. } => match cause {
                        MapKernelFailure::Stop(cause) => Err(cause.into()),
                        MapKernelFailure::Panic => Err(BridgeExecutionDenial::Panicked),
                        MapKernelFailure::Domain(never) => match never {},
                        MapKernelFailure::ResultCapacityExceeded => {
                            unreachable!("unit contact carries no result or error heap")
                        }
                    },
                },
            }
        })
        .map_err(BridgeExecutionDenial::from)?
}
