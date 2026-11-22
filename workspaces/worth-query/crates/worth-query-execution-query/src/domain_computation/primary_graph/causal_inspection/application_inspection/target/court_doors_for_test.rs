//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationInspectionTarget {
    #[doc(hidden)]
    pub fn capture_for_test(
        execution: &ExecutionRequest<'_, '_>,
        original: &WorthQueryApplicationQueryObservation,
        before: Option<&WorthQueryApplicationQueryObservation>,
        contract: WorthQueryInstalledApplicationInspectionContract,
        memory: &mut Option<ExecutionMemoryReservation>,
    ) -> Result<Arc<Self>, InspectionTargetDenial> {
        Self::capture(execution, original, before, contract, memory)
    }
}

impl WorthQueryApplicationInspectionTarget {
    #[doc(hidden)]
    pub fn contract_for_test(&self) -> &WorthQueryInstalledApplicationInspectionContract {
        Self::contract(self)
    }
}

impl WorthQueryApplicationInspectionTarget {
    #[doc(hidden)]
    pub fn original_for_test(&self) -> Option<&WorthQueryApplicationQueryObservation> {
        Self::original(self)
    }
}

impl WorthQueryApplicationInspectionTarget {
    #[doc(hidden)]
    pub fn parameters_for_test(&self) -> worth_foundational::facade::CanonicalDigestId {
        Self::parameters(self)
    }
}

impl WorthQueryApplicationInspectionTarget {
    #[doc(hidden)]
    pub fn original_subject_for_test(&self) -> worth_relational::facade::identity::EntityId {
        Self::original_subject(self)
    }
}
