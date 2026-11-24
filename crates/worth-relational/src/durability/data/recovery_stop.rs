use crate::execution::denial_cause::RelationalExecutionDenialCause;

use crate::durability::data::recovery_errors::DurabilityError;

/// An in-process recovery refusal. Request control is never durable evidence.
///
/// Durable failures retain their existing serialization contract:
/// ```
/// fn durable<T: serde::Serialize>() {}
/// durable::<worth_relational::facade::durability::DurabilityError>();
/// ```
/// A recovery stop cannot be persisted as a recovery failure:
/// ```compile_fail,E0277
/// fn durable<T: serde::Serialize>() {}
/// durable::<worth_relational::facade::durability::RelationalRecoveryStop>();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRecoveryStop {
    AdmissionImage(
        crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
    ),
    Request(RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Durability(DurabilityError),
    Content(crate::content::denial::RelationalContentDenial),
}

impl From<RelationalExecutionDenialCause> for RelationalRecoveryStop {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        match cause {
            RelationalExecutionDenialCause::StorageUnavailable(failure) => failure.into(),
            RelationalExecutionDenialCause::Content(cause) => Self::Content(cause.into()),
            cause => Self::Request(cause),
        }
    }
}

impl From<DurabilityError> for RelationalRecoveryStop {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(error)
    }
}

impl From<crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal>
    for RelationalRecoveryStop
{
    fn from(
        cause: crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal,
    ) -> Self {
        use crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Image;
        match cause {
            Image::Request(cause) => Self::Request(cause),
            Image::Content(cause) => cause.into(),
            Image::Allocation(cause) => Self::Allocation(cause),
            Image::Native(cause) => Self::Durability(cause),
            Image::Buffer(worth_execution::ExecutionRequestArrayDenial::Request(cause)) => {
                Self::Request(cause.into())
            }
            Image::Buffer(worth_execution::ExecutionRequestArrayDenial::Allocation(cause)) => {
                Self::Allocation(cause)
            }
            cause @ (Image::MissingArtifact
            | Image::PredecessorFormat { .. }
            | Image::UnsupportedVersion(_)
            | Image::InvalidVersion
            | Image::Malformed
            | Image::BindingMismatch(_)
            | Image::AuthenticationMismatch(_)
            | Image::MissingRootImage
            | Image::InvalidProjectedMeaning
            | Image::MissingCaptureOrigin
            | Image::OwnerUnavailable
            | Image::CapacityOverflow) => Self::AdmissionImage(cause),
        }
    }
}

impl From<crate::storage::data::storage_unavailable::RelationalStorageUnavailable>
    for RelationalRecoveryStop
{
    fn from(
        failure: crate::storage::data::storage_unavailable::RelationalStorageUnavailable,
    ) -> Self {
        Self::Durability(failure.into())
    }
}

impl std::fmt::Display for RelationalRecoveryStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdmissionImage(cause) => {
                write!(formatter, "checkpoint admission image: {cause:?}")
            }
            Self::Allocation(cause) => write!(formatter, "recovery allocation stopped: {cause}"),
            Self::Request(cause) => write!(formatter, "recovery request stopped: {cause:?}"),
            Self::Content(cause) => write!(formatter, "recovery content refused: {cause:?}"),
            Self::Durability(error) => write!(formatter, "recovery durability: {error:?}"),
        }
    }
}

impl std::error::Error for RelationalRecoveryStop {}

impl From<crate::content::denial::RelationalContentDenial> for RelationalRecoveryStop {
    fn from(cause: crate::content::denial::RelationalContentDenial) -> Self {
        match cause {
            crate::content::denial::RelationalContentDenial::Execution(cause) => Self::from(cause),
            cause => Self::Content(cause),
        }
    }
}

pub(crate) fn map_stop(
    cause: worth_execution::ExecutionOrderedMapDenial,
) -> RelationalRecoveryStop {
    match cause {
        worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
            RelationalRecoveryStop::Request(cause.into())
        }
        worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => {
            RelationalRecoveryStop::Allocation(cause)
        }
        worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => {
            RelationalRecoveryStop::Request(RelationalExecutionDenialCause::MemoryOverflow)
        }
    }
}

pub(crate) fn account_work(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    work: u64,
) -> Result<(), RelationalRecoveryStop> {
    let mut answer = None;
    let scope = request.run_native_with_checkpoint(|context| {
        answer = Some(crate::execution::request_stage::checkpoint(context, work));
    });
    match (scope, answer) {
        (_, Some(Err(cause))) => Err(RelationalRecoveryStop::Request(
            worth_execution::WorkCeilingDenial::Stopped(cause).into(),
        )),
        (Err(cause), _) => Err(RelationalRecoveryStop::Request(cause.into())),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed artifact work executes its callback"),
    }
}
