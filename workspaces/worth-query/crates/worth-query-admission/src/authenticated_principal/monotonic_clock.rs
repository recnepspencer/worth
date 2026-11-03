//! Admission retains the execution owner's monotonic mechanism.
pub use worth_execution::{
    MonotonicClock as WorthQueryAdmissionClock,
    MonotonicTimeAdvanceDenial as WorthQueryAdmissionTimeAdvanceDenial,
    MonotonicTimeConstructionDenial as WorthQueryAdmissionTimeConstructionDenial,
    MonotonicTimeController as WorthQueryAdmissionTimeController,
};
