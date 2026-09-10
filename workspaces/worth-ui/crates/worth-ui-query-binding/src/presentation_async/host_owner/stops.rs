#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationAdmissionStop {
    RuntimeAdmission,
    RuntimeObservation,
    QuerySupersession,
    UnexpectedPendingPosture,
    UnexpectedSupersessionPosture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiPresentationCleanupProgress {
    pub cause: WorthUiPresentationAdmissionStop,
    pub stopped_at: WorthUiPresentationRuntimeCleanupStop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationSettlementStop {
    QueryCompletion,
    QuerySupersession,
    QueryObservation,
    UnexpectedQueryPosture,
    QueryClose,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationRuntimeCleanupStop {
    Query,
}
