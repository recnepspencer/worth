/// Why a generated output publication had no effect. In every case no owner and
/// no product reference moved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputPublicationNoEffectCause {
    /// The caller request refused admission before owner contact.
    ExecutionRequest(worth_execution::WorkCeilingDenial),
    /// The product head moved from the expected head.
    StaleExpectedProductHead,
    /// The publication was cancelled before the first owner effect.
    CancelledBeforeEffect,
    /// The deadline passed before the first owner effect.
    DeadlineBeforeEffect,
    /// An owner refused the publication before any effect.
    OwnerDeniedBeforeEffect,
    /// The bridge correspondence moved from the prepared basis; prepare again.
    CorrespondenceRebindRequired,
    /// The product reference has no generation left to advance to.
    ReferenceGenerationExhausted,
    /// A publication capacity limit was reached.
    CapacityExhausted,
    /// An owner could not be reached.
    OwnerUnavailable,
    /// Relational's exact pre-effect stop, including its recovery/budget reason.
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    /// The publication plan failed a check before any effect.
    PreEffectFailure,
}

/// A generated output publication that had no effect: nothing was published.
/// Read the reason with [`cause`](Self::cause).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryGeneratedOutputPublicationNoEffect {
    cause: WorthQueryGeneratedOutputPublicationNoEffectCause,
}

impl WorthQueryGeneratedOutputPublicationNoEffect {
    pub(super) fn from_world(
        no_effect: worth_runtime_world::facade::NoEffectCompositePublication,
    ) -> Self {
        Self {
            cause: map_cause(no_effect.cause()),
        }
    }

    pub const fn cause(&self) -> WorthQueryGeneratedOutputPublicationNoEffectCause {
        self.cause
    }
}

const fn map_cause(
    cause: worth_runtime_world::facade::NoEffectCause,
) -> WorthQueryGeneratedOutputPublicationNoEffectCause {
    use worth_runtime_world::facade::NoEffectCause as WorldCause;
    match cause {
        WorldCause::ExecutionRequest(cause) => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::ExecutionRequest(cause)
        }
        WorldCause::StaleExpectedProductHead => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::StaleExpectedProductHead
        }
        WorldCause::CancelledBeforeEffect => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::CancelledBeforeEffect
        }
        WorldCause::DeadlineBeforeEffect => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::DeadlineBeforeEffect
        }
        WorldCause::OwnerDeniedBeforeEffect => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::OwnerDeniedBeforeEffect
        }
        WorldCause::CorrespondenceRebindRequired => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::CorrespondenceRebindRequired
        }
        WorldCause::ReferenceGenerationExhausted => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::ReferenceGenerationExhausted
        }
        WorldCause::CapacityExhausted => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::CapacityExhausted
        }
        WorldCause::OwnerUnavailable => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::OwnerUnavailable
        }
        WorldCause::RelationalDeferred(reason) => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::RelationalDeferred(reason)
        }
        WorldCause::PreEffectFailure => {
            WorthQueryGeneratedOutputPublicationNoEffectCause::PreEffectFailure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_no_effect_causes_are_expressed_in_query_vocabulary() {
        assert_eq!(
            map_cause(worth_runtime_world::facade::NoEffectCause::CapacityExhausted),
            WorthQueryGeneratedOutputPublicationNoEffectCause::CapacityExhausted
        );
    }
}
