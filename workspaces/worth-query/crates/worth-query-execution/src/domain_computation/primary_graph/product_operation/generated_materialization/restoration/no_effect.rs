#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryGeneratedOutputPublicationNoEffectCause {
    StaleExpectedProductHead,
    CancelledBeforeEffect,
    DeadlineBeforeEffect,
    OwnerDeniedBeforeEffect,
    CorrespondenceRebindRequired,
    ReferenceGenerationExhausted,
    CapacityExhausted,
    OwnerUnavailable,
    PreEffectFailure,
}

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
