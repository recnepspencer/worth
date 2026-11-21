//! Pure durable dispatch record denial enum.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDurableDispatchRecordDenial {
    AttemptAlreadyRecorded,
    AttemptAwaitingObservation,
    RequestStopped,
    PublicationUnresolved,
    PublicationNoEffect,
    JournalRefused,
}
