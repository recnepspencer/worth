use crate::branch::ProductBranchObservation;
use crate::publication::{NoEffectCompositePublication, ReservedCompositePublicationAttempt};

/// Compile-visible stage marker for a publication that never contacts the
/// Signal owner. It is not constructible outside this module.
#[derive(Debug)]
pub struct WithoutSignal {
    _private: (),
}

/// Compile-visible stage marker for a publication that advances the Signal
/// owner. It is not constructible outside this module.
#[derive(Debug)]
pub struct WithSignal {
    _private: (),
}

/// Binds one stage marker to the single prepared type a reservation in that
/// stage may become. The mapping is total and has no inverse: a prepared
/// value cannot move to the other stage.
pub(crate) trait CompositePublicationStage {
    type Prepared;

    fn seal(attempt: ReservedCompositePublicationAttempt) -> Self::Prepared;
}

impl CompositePublicationStage for WithoutSignal {
    type Prepared = PreparedCompositePublicationWithoutSignal;

    fn seal(attempt: ReservedCompositePublicationAttempt) -> Self::Prepared {
        PreparedCompositePublicationWithoutSignal { attempt }
    }
}

impl CompositePublicationStage for WithSignal {
    type Prepared = PreparedCompositePublicationWithSignal;

    fn seal(attempt: ReservedCompositePublicationAttempt) -> Self::Prepared {
        PreparedCompositePublicationWithSignal { attempt }
    }
}

/// Reserved attempt sealed to a no-Signal execution. There is no conversion
/// into the Signal-advancing prepared type, in either direction.
#[derive(Debug)]
#[must_use = "a prepared attempt must be executed or cancelled"]
pub struct PreparedCompositePublicationWithoutSignal {
    attempt: ReservedCompositePublicationAttempt,
}

impl PreparedCompositePublicationWithoutSignal {
    pub fn expected_head(&self) -> &ProductBranchObservation {
        self.attempt.expected_head()
    }

    /// The non-authorizing identity of the recovery slot already reserved for
    /// this attempt. It permits exact lifecycle correlation only; it cannot
    /// execute, publish, inspect, or clean up the prepared attempt.
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.attempt.unpublished_recovery_handle()
    }

    pub fn cancel(self) -> NoEffectCompositePublication {
        self.attempt.cancel()
    }

    pub(crate) fn into_attempt(self) -> ReservedCompositePublicationAttempt {
        self.attempt
    }
}

/// Reserved attempt sealed to a Signal-advancing execution. There is no
/// conversion into the no-Signal prepared type, in either direction.
#[derive(Debug)]
#[must_use = "a prepared attempt must be executed or cancelled"]
pub struct PreparedCompositePublicationWithSignal {
    attempt: ReservedCompositePublicationAttempt,
}

impl PreparedCompositePublicationWithSignal {
    pub(crate) fn reserve_conditional_definition_custody(
        &mut self,
    ) -> std::sync::Arc<crate::publication::ConditionalDefinitionAttemptCustody> {
        self.attempt.reserve_conditional_definition_custody()
    }

    pub fn expected_head(&self) -> &ProductBranchObservation {
        self.attempt.expected_head()
    }

    /// The non-authorizing recovery identity reserved with this attempt.
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.attempt.unpublished_recovery_handle()
    }

    /// Read-only view of the sealed reservation. Reading a plan cannot move
    /// the attempt into the other stage or execute it.
    #[cfg(test)]
    pub(crate) fn attempt(&self) -> &ReservedCompositePublicationAttempt {
        &self.attempt
    }

    pub fn cancel(self) -> NoEffectCompositePublication {
        self.attempt.cancel()
    }

    pub(crate) fn into_attempt(self) -> ReservedCompositePublicationAttempt {
        self.attempt
    }
}
